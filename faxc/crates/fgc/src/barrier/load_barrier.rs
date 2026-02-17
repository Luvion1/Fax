//! Load Barrier Implementation
//!
//! Load barrier adalah fungsi yang dipanggil saat membaca pointer dari heap.
//! Berbeda dengan write barrier (CMS, G1) yang aktif saat menulis,
//! load barrier aktif saat membaca pointer.
//!
//! Keuntungan Load Barrier:
//! 1. Concurrent Relocation - Object bisa dipindahkan saat thread lain masih
//!    membaca pointer lama. Load barrier handle forwarding on-the-fly.
//!
//! 2. Lazy Marking - Marking tidak perlu traverse seluruh heap sekaligus.
//!    Object ditandai saat pertama kali diakses setelah GC start.
//!
//! 3. Self-Healing Pointers - Pointer diupdate saat digunakan, tidak perlu
//!    pause untuk remap semua pointers sekaligus.
//!
//! 4. No Write Barrier Overhead - Writes tetap cepat, tidak perlu barrier
//!    code injection di setiap write operation.
//!
//! Load Barrier Pseudocode:
//! ```
//! function LOAD_BARRIER(pointer):
//!     if NOT_NEEDS_PROCESSING(pointer):
//!         return pointer  // Fast path, no overhead
//!
//!     color = GET_COLOR(pointer)
//!
//!     if GC_PHASE == MARKING:
//!         if color == UNMARKED:
//!             MARK_OBJECT(pointer)
//!             SET_COLOR(pointer, CURRENT_MARK_BIT)
//!         return pointer
//!
//!     if GC_PHASE == RELOCATING:
//!         if color == MARKED:
//!             new_address = CHECK_FORWARDING_TABLE(pointer)
//!             if new_address != NULL:
//!                 CAS(pointer, new_address | REMAPPED)
//!                 return new_address
//!         return pointer
//! ```

use crate::barrier::colored_ptr::{ColoredPointer, GcPhase};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// LoadBarrier - handler untuk load barrier operations
///
/// Load barrier adalah fungsi yang dipanggil saat membaca pointer dari heap.
/// Fungsi ini memproses pointer berdasarkan color dan GC phase.
///
/// Thread Safety:
/// LoadBarrier dirancang untuk diakses dari multiple threads secara concurrent.
/// Semua operations lock-free untuk performance.
pub struct LoadBarrier {
    /// Current GC phase
    /// Menentukan behavior barrier (marking, relocating, idle)
    phase: std::sync::Mutex<GcPhase>,

    /// Reference ke mark queue untuk enqueue object yang perlu di-mark
    mark_queue: std::sync::Arc<crate::marker::MarkQueue>,

    /// Reference ke forwarding table untuk lookup alamat baru
    forwarding_table: std::sync::Arc<crate::relocate::ForwardingTable>,

    /// Current mark bit (Marked0 atau Marked1)
    /// Bergantian setiap GC cycle
    current_mark_bit: AtomicBool,

    /// Enable load barrier (bisa disable untuk debugging)
    enabled: AtomicBool,
}

impl LoadBarrier {
    /// Create new load barrier
    ///
    /// # Arguments
    /// * `mark_queue` - Queue untuk marking
    /// * `forwarding_table` - Table untuk relocation forwarding
    pub fn new(
        mark_queue: std::sync::Arc<crate::marker::MarkQueue>,
        forwarding_table: std::sync::Arc<crate::relocate::ForwardingTable>,
    ) -> Self {
        Self {
            phase: std::sync::Mutex::new(GcPhase::Idle),
            mark_queue,
            forwarding_table,
            current_mark_bit: AtomicBool::new(false), // false = Marked0, true = Marked1
            enabled: AtomicBool::new(true),
        }
    }

    /// Main load barrier entry point
    ///
    /// Dipanggil setiap kali membaca pointer dari heap.
    /// Fast path: inline check, slow path: this function.
    ///
    /// # Arguments
    /// * `pointer` - Colored pointer yang akan dibaca
    ///
    /// # Returns
    /// Pointer yang sudah di-process (mungkin sudah healed)
    pub fn on_pointer_load(&self, pointer: ColoredPointer) -> ColoredPointer {
        // Fast path check - jika barrier disabled atau tidak perlu processing
        if !self.enabled.load(Ordering::Relaxed) {
            return pointer;
        }

        let phase = *self.phase.lock().unwrap();

        // Fast path: check if pointer needs processing
        if !pointer.needs_processing(phase) {
            return pointer;
        }

        // Slow path: process based on phase
        match phase {
            GcPhase::Marking => self.handle_marking(pointer),
            GcPhase::Relocating => self.handle_relocating(pointer),
            _ => pointer,
        }
    }

    /// Atomic load barrier with pointer healing
    ///
    /// This is the CRITICAL fix for pointer healing. Unlike `on_pointer_load`,
    /// this method atomically updates the source memory location when a pointer
    /// is healed (relocated).
    ///
    /// # How it works:
    /// 1. Atomically load current pointer value
    /// 2. Process through barrier (check forwarding table, etc.)
    /// 3. If pointer was healed (address changed), update source with CAS
    /// 4. Return the healed pointer
    ///
    /// # Thread Safety
    /// Uses CAS (compare-and-swap) to ensure atomic update even when
    /// multiple threads access the same pointer location concurrently.
    ///
    /// # Arguments
    /// * `ptr_location` - Atomic pointer to the memory location containing the pointer
    ///
    /// # Returns
    /// The processed (possibly healed) pointer value
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use fgc::barrier::load_barrier::LoadBarrier;
    ///
    /// let ptr_location = AtomicUsize::new(0x1000);
    /// let healed = barrier.on_pointer_load_atomic(&ptr_location);
    /// // ptr_location is now updated atomically if pointer was healed
    /// ```
    pub fn on_pointer_load_atomic(&self, ptr_location: &AtomicUsize) -> ColoredPointer {
        // Fast path check - if barrier disabled, just load and return
        if !self.enabled.load(Ordering::Relaxed) {
            let raw = ptr_location.load(Ordering::Acquire);
            return ColoredPointer::from_raw(raw);
        }

        let phase = *self.phase.lock().unwrap();

        // Load current pointer value atomically
        let current_raw = ptr_location.load(Ordering::Acquire);
        let pointer = ColoredPointer::from_raw(current_raw);

        // Fast path: check if pointer needs processing
        if !pointer.needs_processing(phase) {
            return pointer;
        }

        // Slow path: process based on phase
        let result = match phase {
            GcPhase::Marking => self.handle_marking_atomic(ptr_location, pointer),
            GcPhase::Relocating => self.handle_relocating_atomic(ptr_location, pointer),
            _ => pointer,
        };

        result
    }

    /// Handle marking phase with atomic update
    ///
    /// Atomically sets mark bit if object is unmarked.
    fn handle_marking_atomic(
        &self,
        ptr_location: &AtomicUsize,
        pointer: ColoredPointer,
    ) -> ColoredPointer {
        if !pointer.is_marked() {
            // Object belum marked, enqueue untuk marking
            self.mark_queue.push(pointer.address());

            // Atomically set mark bit using test-and-set pattern
            // This ensures only one thread "wins" the marking
            let mark_mask = if self.current_mark_bit.load(Ordering::Relaxed) {
                ColoredPointer::MARKED1_MASK
            } else {
                ColoredPointer::MARKED0_MASK
            };

            // Use fetch_or to atomically set the mark bit
            ptr_location.fetch_or(mark_mask, Ordering::AcqRel);

            // Return pointer with mark bit set
            ColoredPointer::from_raw(ptr_location.load(Ordering::Acquire))
        } else {
            pointer
        }
    }

    /// Handle relocating phase with atomic pointer healing
    ///
    /// This is the CORE of pointer healing. When an object has been relocated:
    /// 1. Lookup new address in forwarding table
    /// 2. Create healed pointer with new address and remapped bit
    /// 3. Atomically update source location using CAS
    /// 4. Return healed pointer
    ///
    /// # CAS Loop
    /// Uses compare-and-swap loop to handle concurrent updates:
    /// - If another thread updated the pointer first, we see their update
    /// - If CAS fails, we retry with the new value
    fn handle_relocating_atomic(
        &self,
        ptr_location: &AtomicUsize,
        pointer: ColoredPointer,
    ) -> ColoredPointer {
        let address = pointer.address();

        // Check forwarding table
        if let Some(new_address) = self.forwarding_table.lookup(address) {
            // Object sudah dipindahkan, create healed pointer
            let mut healed = ColoredPointer::new(new_address);
            healed.set_remapped();
            let healed_raw = healed.raw();

            // Atomic update with CAS loop
            // This ensures pointer healing happens exactly once even with
            // concurrent access from multiple threads
            let mut current_raw = pointer.raw();
            loop {
                match ptr_location.compare_exchange(
                    current_raw,
                    healed_raw,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        // CAS succeeded - we healed the pointer
                        return healed;
                    }
                    Err(actual) => {
                        // CAS failed - another thread updated the pointer
                        // Check if it was already healed
                        let actual_ptr = ColoredPointer::from_raw(actual);
                        if actual_ptr.is_remapped() || actual_ptr.address() == new_address {
                            // Already healed by another thread
                            return actual_ptr;
                        }
                        // Different update - retry with new current value
                        current_raw = actual;
                    }
                }
            }
        }

        pointer
    }

    /// Handle marking phase
    ///
    /// Jika pointer belum marked:
    /// 1. Enqueue object ke mark queue
    /// 2. Set color bit (Marked0 atau Marked1)
    fn handle_marking(&self, mut pointer: ColoredPointer) -> ColoredPointer {
        if !pointer.is_marked() {
            // Object belum marked, enqueue untuk marking
            self.mark_queue.push(pointer.address());

            // Set mark bit sesuai current cycle
            if self.current_mark_bit.load(Ordering::Relaxed) {
                pointer.set_marked1();
            } else {
                pointer.set_marked0();
            }
        }

        pointer
    }

    /// Handle relocating phase
    ///
    /// Jika pointer ke object di relocation set:
    /// 1. Check forwarding table
    /// 2. Update pointer (self-healing)
    /// 3. Return new address
    fn handle_relocating(&self, mut pointer: ColoredPointer) -> ColoredPointer {
        let address = pointer.address();

        // Check forwarding table
        if let Some(new_address) = self.forwarding_table.lookup(address) {
            // Object sudah dipindahkan, heal pointer
            pointer = ColoredPointer::new(new_address);
            pointer.set_remapped();

            return pointer;
        }

        pointer
    }

    /// Set current GC phase
    ///
    /// Dipanggil oleh GC coordinator saat transition phases.
    pub fn set_phase(&self, phase: GcPhase) {
        *self.phase.lock().unwrap() = phase;
    }

    /// Get current GC phase
    pub fn phase(&self) -> GcPhase {
        *self.phase.lock().unwrap()
    }

    /// Flip mark bit untuk GC cycle baru
    ///
    /// Dipanggil saat GC cycle berganti (even <-> odd).
    pub fn flip_mark_bit(&self) {
        let current = self.current_mark_bit.load(Ordering::Relaxed);
        self.current_mark_bit.store(!current, Ordering::Relaxed);
    }

    /// Enable atau disable load barrier
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check jika barrier enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Get current mark bit
    pub fn current_mark_bit(&self) -> bool {
        self.current_mark_bit.load(Ordering::Relaxed)
    }
}

/// Inline load barrier untuk code generation
///
/// Function ini bisa di-inline oleh code generator untuk
/// fast path load barrier.
///
/// # Arguments
/// * `ptr` - Raw pointer value
/// * `barrier` - LoadBarrier reference
///
/// # Returns
/// Processed pointer value
#[inline]
pub fn inline_load_barrier(ptr: usize, barrier: &LoadBarrier) -> usize {
    let colored = ColoredPointer::from_raw(ptr);
    let processed = barrier.on_pointer_load(colored);
    processed.raw()
}

/// Load barrier fast path check
///
/// Check cepat apakah pointer perlu processing.
/// Bisa di-inline di generated code.
#[inline]
pub fn load_barrier_fast_path(ptr: usize, phase: GcPhase) -> bool {
    let colored = ColoredPointer::from_raw(ptr);
    !colored.needs_processing(phase)
}
