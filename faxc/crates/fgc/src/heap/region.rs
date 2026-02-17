//! Region Implementation - Unit Dasar Heap Management
//!
//! Region adalah contiguous block of memory dengan ukuran tetap.
//! Setiap region memiliki state lifecycle yang terdefinisi.
//!
//! Region Types:
//! - Small (2MB): Untuk object < 256 bytes, 90%+ alokasi masuk sini
//! - Medium (32MB): Untuk object 256 bytes - 4KB
//! - Large (variable): Untuk object > 4KB, 1 object per region
//!
//! Region Lifecycle:
//! ```
//! Allocating ──▶ Allocated ──▶ Relocating ──▶ Relocated ──▶ Free
//!     │              │              │              │          │
//!     └──────────────┴──────────────┴──────────────┴──────────┘
//! ```

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Region - unit dasar heap management
///
/// Region adalah contiguous block of memory dengan ukuran tetap.
/// Setiap region memiliki state lifecycle yang terdefinisi.
///
/// Thread Safety:
/// Region dirancang untuk diakses dari multiple threads.
/// State changes menggunakan atomic operations.
pub struct Region {
    /// Alamat awal region di virtual memory
    start: AtomicUsize,

    /// Ukuran region (2MB, 32MB, atau variable)
    size: usize,

    /// Tipe region (Small, Medium, Large)
    region_type: RegionType,

    /// State saat ini
    state: std::sync::Mutex<RegionState>,

    /// Bump pointer untuk allocation
    top: AtomicUsize,

    /// End address (limit) dari region
    end: usize,

    /// Marking bitmap untuk tracking live objects
    mark_bitmap: Vec<AtomicU64>,

    /// Forwarding table untuk relocation
    forwarding_table: std::sync::Mutex<Option<Arc<crate::relocate::ForwardingTable>>>,

    /// NUMA node tempat region dialokasikan
    numa_node: usize,

    /// Generation (Young atau Old)
    generation: Generation,

    /// Allocation count - berapa kali region ini digunakan
    allocation_count: AtomicUsize,

    /// Mark bit current (false = Marked0, true = Marked1)
    mark_bit: AtomicBool,

    /// Is memory from VirtualMemory (needs commit)
    needs_commit: bool,
}

impl Region {
    /// Create new region dengan specific start address
    ///
    /// # Arguments
    /// * `start_address` - Start address dari region (dari VirtualMemory)
    /// * `region_type` - Type region (Small, Medium, Large)
    /// * `size` - Size region dalam bytes
    /// * `generation` - Generation (Young atau Old)
    pub fn with_address(
        start_address: usize,
        region_type: RegionType,
        size: usize,
        generation: Generation,
    ) -> Result<Self> {
        let bitmap_size = (size / 64 + 63) / 64;
        let mark_bitmap = (0..bitmap_size).map(|_| AtomicU64::new(0)).collect();

        Ok(Self {
            start: AtomicUsize::new(start_address),
            size,
            region_type,
            state: std::sync::Mutex::new(RegionState::Free),
            top: AtomicUsize::new(start_address),
            end: start_address + size,
            mark_bitmap,
            forwarding_table: std::sync::Mutex::new(None),
            numa_node: 0,
            generation,
            allocation_count: AtomicUsize::new(0),
            mark_bit: AtomicBool::new(false),
            needs_commit: false,
        })
    }

    /// Create new region (legacy - uses default address)
    ///
    /// # Arguments
    /// * `region_type` - Type region (Small, Medium, Large)
    /// * `size` - Size region dalam bytes
    /// * `generation` - Generation (Young atau Old)
    pub fn new(region_type: RegionType, size: usize, generation: Generation) -> Result<Self> {
        Self::with_address(0x1000_0000, region_type, size, generation)
    }

    /// Create region yang memerlukan memory commit
    pub fn new_sparse(
        start_address: usize,
        region_type: RegionType,
        size: usize,
        generation: Generation,
    ) -> Result<Self> {
        let mut region = Self::with_address(start_address, region_type, size, generation)?;
        region.needs_commit = true;
        Ok(region)
    }

    /// Allocate memory di region ini
    ///
    /// Menggunakan bump pointer allocation (O(1)).
    ///
    /// # Arguments
    /// * `size` - Size dalam bytes
    /// * `alignment` - Alignment requirement
    pub fn allocate(&self, size: usize, alignment: usize) -> Result<usize> {
        // Check state
        let state = *self.state.lock().unwrap();
        if state != RegionState::Allocating && state != RegionState::Allocated {
            return Err(FgcError::RegionAllocationFailed {
                reason: format!("Region in {:?} state", state),
            });
        }

        // Align size
        let aligned_size = (size + alignment - 1) & !(alignment - 1);

        // Bump pointer allocation
        let mut current_top = self.top.load(Ordering::Relaxed);

        loop {
            let new_top = current_top + aligned_size;

            // Check jika region penuh
            if new_top > self.end {
                return Err(FgcError::OutOfMemory {
                    requested: size,
                    available: self.end - current_top,
                });
            }

            // Try CAS untuk update top
            match self.top.compare_exchange_weak(
                current_top,
                new_top,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Success! Mark object di bitmap
                    self.mark_object(current_top, size);
                    return Ok(current_top);
                }
                Err(actual) => {
                    // CAS gagal, retry
                    current_top = actual;
                }
            }
        }
    }

    /// Mark object di bitmap
    pub fn mark_object(&self, address: usize, _size: usize) {
        // Calculate bitmap index
        let offset = address - self.start.load(Ordering::Relaxed);
        let bit_index = offset / 64;
        let bit_offset = offset % 64;

        if bit_index < self.mark_bitmap.len() {
            // Use fetch_or untuk atomic update
            self.mark_bitmap[bit_index].fetch_or(1u64 << bit_offset, Ordering::Relaxed);
        }
    }

    /// Check jika object marked
    pub fn is_marked(&self, address: usize) -> bool {
        let offset = address - self.start.load(Ordering::Relaxed);
        let bit_index = offset / 64;
        let bit_offset = offset % 64;

        if bit_index >= self.mark_bitmap.len() {
            return false;
        }

        (self.mark_bitmap[bit_index].load(Ordering::Relaxed) & (1u64 << bit_offset)) != 0
    }

    /// Reset region untuk reuse
    pub fn reset(&self) -> Result<()> {
        *self.state.lock().unwrap() = RegionState::Allocating;
        self.top
            .store(self.start.load(Ordering::Relaxed), Ordering::SeqCst);

        // Clear bitmap
        for word in self.mark_bitmap.iter() {
            word.store(0, Ordering::Relaxed);
        }

        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Flip mark bits untuk GC cycle baru
    pub fn flip_mark_bits(&self) {
        let current = self.mark_bit.load(Ordering::Relaxed);
        self.mark_bit.store(!current, Ordering::Relaxed);
    }

    /// Get current mark bit
    pub fn current_mark_bit(&self) -> bool {
        self.mark_bit.load(Ordering::Relaxed)
    }

    /// Set region state
    pub fn set_state(&self, state: RegionState) {
        *self.state.lock().unwrap() = state;
    }

    /// Get region state
    pub fn state(&self) -> RegionState {
        *self.state.lock().unwrap()
    }

    /// Get region type
    pub fn region_type(&self) -> RegionType {
        self.region_type
    }

    /// Get region size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Get bytes used di region
    pub fn used(&self) -> usize {
        self.top.load(Ordering::Relaxed) - self.start.load(Ordering::Relaxed)
    }

    /// Get bytes remaining
    pub fn remaining(&self) -> usize {
        self.end - self.top.load(Ordering::Relaxed)
    }

    /// Check jika region penuh
    pub fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    /// Get start address
    pub fn start(&self) -> usize {
        self.start.load(Ordering::Relaxed)
    }

    /// Get end address
    pub fn end(&self) -> usize {
        self.end
    }

    /// Get allocation count
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Get garbage ratio (estimasi)
    pub fn garbage_ratio(&self) -> f32 {
        // Simple estimation: ratio of unmarked objects
        let total_objects = self.allocation_count.load(Ordering::Relaxed);
        if total_objects == 0 {
            return 0.0;
        }

        let marked_count = self.count_marked();
        1.0 - (marked_count as f32 / total_objects as f32)
    }

    /// Count marked objects
    fn count_marked(&self) -> usize {
        self.mark_bitmap
            .iter()
            .map(|word| word.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }

    /// Setup forwarding table untuk relocation
    pub fn setup_forwarding(&self) {
        let mut ft = self.forwarding_table.lock().unwrap();
        *ft = Some(Arc::new(crate::relocate::ForwardingTable::new(
            self.start.load(Ordering::Relaxed),
            self.size,
        )));
    }

    /// Get forwarding table
    pub fn forwarding_table(&self) -> Option<Arc<crate::relocate::ForwardingTable>> {
        self.forwarding_table.lock().unwrap().clone()
    }
}

/// Tipe region berdasarkan ukuran object yang ditampung
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    /// 2MB region untuk object kecil (< 256 bytes)
    Small,
    /// 32MB region untuk object medium (256 bytes - 4KB)
    Medium,
    /// Variable size region untuk large object (> 4KB)
    Large,
}

/// State lifecycle region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionState {
    /// Region sedang diisi dengan object baru
    Allocating,
    /// Region penuh atau tidak lagi allocating
    Allocated,
    /// Region dipilih untuk compaction
    Relocating,
    /// Semua object sudah dipindahkan
    Relocated,
    /// Region kosong dan siap digunakan kembali
    Free,
}

/// Young vs Old generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    /// Young generation (nursery) untuk object baru
    Young,
    /// Old generation untuk object yang survive minor GC
    Old,
}
