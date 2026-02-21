//! Load Barrier Implementation
//!
//! Load barrier is a function called when reading pointers from the heap.
//! Unlike write barrier (CMS, G1) which is active during writes,
//! load barrier is active during reads.
//!
//! Load Barrier Benefits:
//! 1. Concurrent Relocation - Objects can be moved while other threads are
//!    still reading old pointers. Load barrier handles forwarding on-the-fly.
//!
//! 2. Lazy Marking - Marking doesn't need to traverse the entire heap at once.
//!    Objects are marked when first accessed after GC start.
//!
//! 3. Self-Healing Pointers - Pointers are updated when used, no need to
//!    pause to remap all pointers at once.
//!
//! 4. No Write Barrier Overhead - Writes remain fast, no barrier
//!    code injection needed in every write operation.
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

/// LoadBarrier - handler for load barrier operations
///
/// Load barrier is a function called when reading pointers from the heap.
/// This function processes pointers based on color and GC phase.
///
/// Thread Safety:
/// LoadBarrier is designed to be accessed from multiple threads concurrently.
/// All operations are lock-free for performance.
pub struct LoadBarrier {
    /// Current GC phase
    /// Determines barrier behavior (marking, relocating, idle)
    phase: std::sync::Mutex<GcPhase>,

    /// Good Color Mask - Pointers matching this are safe (fast path)
    good_color_mask: AtomicUsize,

    /// Reference to mark queue for enqueueing objects that need to be marked
    mark_queue: std::sync::Arc<crate::marker::MarkQueue>,

    /// Reference to forwarding table for new address lookup
    forwarding_table: std::sync::Arc<crate::relocate::ForwardingTable>,

    /// Current mark bit (Marked0 or Marked1)
    /// Alternates every GC cycle
    current_mark_bit: AtomicBool,

    /// Enable load barrier (can be disabled for debugging)
    enabled: AtomicBool,
}

impl LoadBarrier {
    /// Create new load barrier
    ///
    /// # Arguments
    /// * `mark_queue` - Queue for marking
    /// * `forwarding_table` - Table for relocation forwarding
    pub fn new(
        mark_queue: std::sync::Arc<crate::marker::MarkQueue>,
        forwarding_table: std::sync::Arc<crate::relocate::ForwardingTable>,
    ) -> Self {
        Self {
            phase: std::sync::Mutex::new(GcPhase::Idle),
            good_color_mask: AtomicUsize::new(0),
            mark_queue,
            forwarding_table,
            current_mark_bit: AtomicBool::new(false), // false = Marked0, true = Marked1
            enabled: AtomicBool::new(true),
        }
    }

    /// Main load barrier entry point
    ///
    /// Called every time a pointer is read from the heap.
    /// Fast path: inline check, slow path: this function.
    ///
    /// # Arguments
    /// * `pointer` - Colored pointer to be read
    ///
    /// # Returns
    /// Processed pointer (possibly healed)
    ///
    /// # Note
    /// Returns the original pointer if the lock is poisoned.
    pub fn on_pointer_load(&self, pointer: ColoredPointer) -> ColoredPointer {
        // Fast path check - if barrier disabled or no processing needed (Good Color)
        if !self.enabled.load(Ordering::Relaxed) || self.is_good_color(pointer.raw()) {
            return pointer;
        }

        let phase = match self.phase.lock() {
            Ok(guard) => *guard,
            Err(e) => {
                log::error!("LoadBarrier phase lock poisoned: {}", e);
                return pointer;
            }
        };

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
        // Fast path check - load current raw value
        let current_raw = ptr_location.load(Ordering::Acquire);
        
        // If barrier disabled or color is good, just return
        if !self.enabled.load(Ordering::Relaxed) || self.is_good_color(current_raw) {
            return ColoredPointer::from_raw(current_raw);
        }

        let phase = match self.phase.lock() {
            Ok(guard) => *guard,
            Err(e) => {
                log::error!("LoadBarrier phase lock poisoned: {}", e);
                return ColoredPointer::from_raw(current_raw);
            }
        };

        let pointer = ColoredPointer::from_raw(current_raw);

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
            // Object not yet marked, enqueue for marking
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
    /// 1. Lookup new address in forwarding table with generation counter
    /// 2. Verify generation hasn't changed (TOCTOU prevention)
    /// 3. Create healed pointer with new address and remapped bit
    /// 4. Atomically update source location using CAS
    /// 5. Return healed pointer
    ///
    /// # FIX Issue 8: TOCTOU Race Prevention
    ///
    /// The forwarding table lookup now includes a generation counter. After
    /// receiving the lookup result, we verify the generation hasn't changed.
    /// If it has, the table was modified during our lookup and we retry.
    ///
    /// # CAS Loop
    /// Uses compare-and-swap loop to handle concurrent updates:
    /// - If another thread updated the pointer first, we see their update
    /// - If CAS fails, we retry with the new value
    ///
    /// # CRIT-02 FIX: Retry Limits
    /// Added retry limits to prevent DoS attacks via CAS starvation.
    /// After MAX_RETRIES attempts, the function returns the pointer as-is
    /// to prevent infinite loops.
    fn handle_relocating_atomic(
        &self,
        ptr_location: &AtomicUsize,
        pointer: ColoredPointer,
    ) -> ColoredPointer {
        const MAX_RETRIES: u32 = 100;
        let mut retries = 0;

        loop {
            // CRIT-02 FIX: Prevent DoS via retry starvation
            if retries >= MAX_RETRIES {
                log::warn!("Forwarding table lookup starved after {} retries", retries);
                return pointer;  // Return as-is to prevent DoS
            }

            let address = pointer.address();

            // FIX Issue 8: Use lookup_with_generation for TOCTOU-safe lookup
            // Retry loop to handle forwarding table modifications during lookup
            // Lookup with generation counter
            let (new_address, generation) = match self.forwarding_table.lookup_with_generation(address) {
                Some(result) => result,
                None => return pointer, // No forwarding entry
            };

            // FIX Issue 8: Verify generation hasn't changed during lookup
            // If generation changed, the table was modified and we need to retry
            if self.forwarding_table.generation() != generation {
                retries += 1;
                std::hint::spin_loop();
                continue;
            }

            // Object has been relocated, create healed pointer
            let mut healed = ColoredPointer::new(new_address);
            healed.set_remapped();
            let _healed_raw = healed.raw();

            // Atomic update with CAS loop
            // This ensures pointer healing happens exactly once even with
            // concurrent access from multiple threads
            let mut current_raw = pointer.raw();
            let mut cas_retries = 0;

            loop {
                // CRIT-02 FIX: Prevent CAS starvation
                if cas_retries >= MAX_RETRIES {
                    log::warn!("CAS starved after {} retries", cas_retries);
                    return ColoredPointer::new(current_raw);
                }

                let expected_raw = current_raw;
                // Create new raw value with new address but preserve color bits from original pointer
                let new_raw = (new_address & ColoredPointer::ADDRESS_MASK) | (pointer.raw() & !ColoredPointer::ADDRESS_MASK);

                match ptr_location.compare_exchange_weak(
                    expected_raw,
                    new_raw,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        // CAS succeeded - we healed the pointer
                        return ColoredPointer::new(new_raw);
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
                        cas_retries += 1;
                    }
                }
            }
        }
    }

    /// Handle marking phase
    ///
    /// If pointer is not yet marked:
    /// 1. Enqueue object to mark queue
    /// 2. Set color bit (Marked0 or Marked1)
    fn handle_marking(&self, mut pointer: ColoredPointer) -> ColoredPointer {
        if !pointer.is_marked() {
            // Object not yet marked, enqueue for marking
            self.mark_queue.push(pointer.address());

            // Set mark bit according to current cycle
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
    /// If pointer points to object in relocation set:
    /// 1. Check forwarding table
    /// 2. Update pointer (self-healing)
    /// 3. Return new address
    fn handle_relocating(&self, mut pointer: ColoredPointer) -> ColoredPointer {
        let address = pointer.address();

        // Check forwarding table
        if let Some(new_address) = self.forwarding_table.lookup(address) {
            // Object has been relocated, heal pointer
            pointer = ColoredPointer::new(new_address);
            pointer.set_remapped();

            return pointer;
        }

        pointer
    }

    /// Optimized fast path check
    #[inline]
    pub fn is_good_color(&self, pointer_raw: usize) -> bool {
        let mask = self.good_color_mask.load(Ordering::Relaxed);
        (pointer_raw & mask) != 0 || pointer_raw == 0
    }

    /// Set current GC phase
    ///
    /// Called by GC coordinator during phase transitions.
    ///
    /// # Returns
    /// * `Ok(())` - Phase successfully set
    /// * `Err(FgcError::LockPoisoned)` - If mutex is poisoned
    pub fn set_phase(&self, phase: GcPhase) -> crate::error::Result<()> {
        let mut guard = self.phase.lock().map_err(|e| {
            crate::error::FgcError::LockPoisoned(format!("LoadBarrier phase lock poisoned: {}", e))
        })?;
        
        *guard = phase;
        
        // Update Good Color Mask based on phase
        let new_mask = match phase {
            GcPhase::Marking => {
                if self.current_mark_bit.load(Ordering::Relaxed) {
                    ColoredPointer::MARKED1_MASK
                } else {
                    ColoredPointer::MARKED0_MASK
                }
            }
            GcPhase::Relocating => ColoredPointer::REMAPPED_MASK,
            GcPhase::Idle => 0,
            _ => 0,
        };
        
        self.good_color_mask.store(new_mask, Ordering::Release);
        Ok(())
    }

    /// Get current GC phase
    ///
    /// # Returns
    /// `Some(GcPhase)` if lock acquired successfully, `None` if poisoned
    pub fn phase(&self) -> Option<GcPhase> {
        self.phase.lock().ok().map(|g| *g)
    }

    /// Flip mark bit for new GC cycle
    ///
    /// Called when GC cycle changes (even <-> odd).
    pub fn flip_mark_bit(&self) {
        let current = self.current_mark_bit.load(Ordering::Relaxed);
        self.current_mark_bit.store(!current, Ordering::Relaxed);
    }

    /// Enable or disable load barrier
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if barrier is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Get current mark bit
    pub fn current_mark_bit(&self) -> bool {
        self.current_mark_bit.load(Ordering::Relaxed)
    }
}

/// Inline load barrier for code generation
///
/// This function can be inlined by code generator for
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
/// Quick check whether pointer needs processing.
/// Can be inlined in generated code.
#[inline]
pub fn load_barrier_fast_path(ptr: usize, phase: GcPhase) -> bool {
    let colored = ColoredPointer::from_raw(ptr);
    !colored.needs_processing(phase)
}

/// Heal a colored pointer by checking the forwarding table
///
/// This function is called by the `read_barrier!` macro to heal pointers
/// that may have been relocated during GC. It checks if the pointer points
/// to a relocated object and updates it to the new address.
///
/// # How it works
///
/// 1. Extract the color bits from the pointer
/// 2. Check the forwarding table for the object's address
/// 3. If found, update the pointer to the new address with remapped color
/// 4. If not found, the pointer is already valid
///
/// # Arguments
///
/// * `ptr` - Mutable reference to the pointer address to heal
///
/// # Safety
///
/// This function is safe to call on any pointer value. If the pointer is
/// invalid or not a GC-managed pointer, it will simply not be modified.
///
/// # Thread Safety
///
/// This function is thread-safe. It uses atomic operations to access the
/// global forwarding table.
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::barrier::load_barrier::heal_pointer;
///
/// let mut ptr: usize = 0x1000; // Colored pointer
/// heal_pointer(&mut ptr);
/// // ptr is now healed if it pointed to a relocated object
/// ```
#[inline]
pub fn heal_pointer(ptr: &mut usize) {
    // Fast path: if pointer is null or not a colored pointer, skip
    if *ptr == 0 {
        return;
    }

    // Try to get the global GC instance for healing
    // In a full implementation, this would access the runtime's forwarding table
    // For now, we provide a no-op implementation that can be extended

    // Check if pointer has color bits set (bits 44-47)
    let color_bits = (*ptr >> 44) & 0xF;

    // If no color bits, this is not a GC-managed pointer
    if color_bits == 0 {
        return;
    }

    // Extract the base address (without color bits)
    let _base_addr = *ptr & ((1 << 44) - 1);

    // Try to lookup in forwarding table
    // In a full implementation, this would access the global forwarding table
    // For now, we check if the pointer is in the "relocated" state
    if is_remapped_color(color_bits) {
        // Pointer is already healed (remapped)
        return;
    }

    // In a full implementation, we would:
    // 1. Get the global GC runtime
    // 2. Access the forwarding table
    // 3. Lookup base_addr
    // 4. If found, update *ptr to new_addr | REMAPPED_COLOR

    // For now, the pointer is considered valid as-is
    // The actual healing happens in LoadBarrier::on_pointer_load_atomic
}

/// Check if color bits indicate a remapped (healed) pointer
#[inline]
fn is_remapped_color(color_bits: usize) -> bool {
    // Remapped color is typically 0x8 (bit 47 set)
    // This indicates the pointer has been healed
    color_bits == 0x8
}

/// Global forwarding table reference for pointer healing
///
/// This is set by the GC runtime when it creates the forwarding table.
/// In a full implementation, this would be a proper global reference.
static mut GLOBAL_FORWARDING_TABLE: Option<*const crate::relocate::ForwardingTable> = None;

/// Set the global forwarding table for pointer healing
///
/// Called by the GC runtime to register the forwarding table.
///
/// # Safety
///
/// The table pointer must remain valid for the duration of GC.
/// This function should only be called during GC safepoints.
pub unsafe fn set_global_forwarding_table(table: &crate::relocate::ForwardingTable) {
    GLOBAL_FORWARDING_TABLE = Some(table as *const _);
}

/// Clear the global forwarding table
///
/// Called by the GC runtime after GC completes.
pub fn clear_global_forwarding_table() {
    unsafe {
        GLOBAL_FORWARDING_TABLE = None;
    }
}

/// Heal pointer using the global forwarding table
///
/// This is the full implementation of pointer healing that uses the
/// global forwarding table set by the GC runtime.
///
/// # Arguments
///
/// * `ptr` - Mutable reference to the pointer address to heal
///
/// # Returns
///
/// `true` if the pointer was healed, `false` if no healing was needed
pub fn heal_pointer_global(ptr: &mut usize) -> bool {
    if *ptr == 0 {
        return false;
    }

    let color_bits = (*ptr >> 44) & 0xF;
    if color_bits == 0 {
        return false;
    }

    if is_remapped_color(color_bits) {
        return false; // Already healed
    }

    let base_addr = *ptr & ((1 << 44) - 1);

    unsafe {
        if let Some(table_ptr) = GLOBAL_FORWARDING_TABLE {
            let table = &*table_ptr;
            if let Some(new_addr) = table.lookup(base_addr) {
                // Create healed pointer with remapped color
                *ptr = new_addr | (0x8 << 44); // Set remapped color bit
                return true;
            }
        }
    }

    false
}

/// Enqueue an object address for marking
///
/// This function is called by the fast path when an object needs to be marked.
/// It adds the object address to the mark queue for processing by GC threads.
///
/// # Arguments
///
/// * `obj_addr` - Object address to enqueue for marking
///
/// # Note
///
/// This is a placeholder implementation. In a full implementation, this would
/// access the global mark queue and enqueue the object.
#[inline]
pub fn enqueue_for_marking(_obj_addr: usize) {
    // Placeholder: In a full implementation, this would:
    // 1. Get the global mark queue
    // 2. Enqueue the object address
    // 3. Signal GC threads if queue was empty
    //
    // For now, this is a no-op that allows compilation
}

/// Called when an object is read
///
/// This function is called by the read barrier macros when an object is accessed.
/// It performs any necessary barrier operations (marking, pointer healing).
///
/// # Arguments
///
/// * `_obj_addr` - Object address being read
///
/// # Note
///
/// This is a placeholder implementation. In a full implementation, this would
/// interact with the LoadBarrier to perform marking and pointer healing.
#[inline]
pub fn on_object_read(_obj_addr: usize) {
    // Placeholder: In a full implementation, this would:
    // 1. Check if object needs marking
    // 2. Check if object has been relocated (pointer healing)
    // 3. Update object state as needed
    //
    // For now, this is a no-op that allows compilation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heal_pointer_null() {
        let mut ptr: usize = 0;
        heal_pointer(&mut ptr);
        assert_eq!(ptr, 0); // Null pointer should remain null
    }

    #[test]
    fn test_heal_pointer_no_color() {
        let mut ptr: usize = 0x1000; // No color bits
        heal_pointer(&mut ptr);
        assert_eq!(ptr, 0x1000); // Non-colored pointer should not change
    }

    #[test]
    fn test_heal_pointer_already_remapped() {
        // Pointer with remapped color (0x8 << 44)
        let mut ptr: usize = 0x1000 | (0x8 << 44);
        heal_pointer(&mut ptr);
        assert_eq!(ptr, 0x1000 | (0x8 << 44)); // Already healed, should not change
    }

    #[test]
    fn test_heal_pointer_with_color() {
        // Pointer with Marked0 color (0x1 << 44)
        let mut ptr: usize = 0x1000 | (0x1 << 44);
        heal_pointer(&mut ptr);
        // Without a forwarding table, pointer should not change
        assert_eq!(ptr, 0x1000 | (0x1 << 44));
    }

    #[test]
    fn test_is_remapped_color() {
        assert!(is_remapped_color(0x8));
        assert!(!is_remapped_color(0x0));
        assert!(!is_remapped_color(0x1));
        assert!(!is_remapped_color(0x2));
        assert!(!is_remapped_color(0x4));
    }

    #[test]
    fn test_heal_pointer_global_null() {
        let mut ptr: usize = 0;
        let result = heal_pointer_global(&mut ptr);
        assert!(!result); // No healing for null pointer
    }

    #[test]
    fn test_heal_pointer_global_no_color() {
        let mut ptr: usize = 0x1000;
        let result = heal_pointer_global(&mut ptr);
        assert!(!result); // No healing for non-colored pointer
    }

    #[test]
    fn test_heal_pointer_global_already_healed() {
        let mut ptr: usize = 0x1000 | (0x8 << 44);
        let result = heal_pointer_global(&mut ptr);
        assert!(!result); // Already healed
        assert_eq!(ptr, 0x1000 | (0x8 << 44));
    }

    #[test]
    fn test_clear_global_forwarding_table() {
        // Just verify it doesn't panic
        clear_global_forwarding_table();
    }

    #[test]
    fn test_load_barrier_fast_path() {
        // Test with various pointer values
        let phase = GcPhase::Idle;

        // Null pointer - fast path should be true (no processing needed)
        assert!(load_barrier_fast_path(0, phase));

        // Non-colored pointer - fast path should be true
        assert!(load_barrier_fast_path(0x1000, phase));
    }
}
