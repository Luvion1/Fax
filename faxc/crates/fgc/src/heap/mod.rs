//! Heap Management Module - Region-Based Memory Management
//!
//! Module ini mengelola heap berbasis region, teknik yang digunakan oleh ZGC
//! untuk parallel collection dan partial compaction.
//!
//! Heap dibagi menjadi region-region dengan ukuran bervariasi:
//! - Small Region (2MB): Untuk object < 256 bytes
//! - Medium Region (32MB): Untuk object 256 bytes - 4KB
//! - Large Region (variable): Untuk object > 4KB (1 object per region)
//!
//! Region Lifecycle States:
//! 1. Allocating - Region sedang diisi dengan object baru
//! 2. Allocated - Region penuh atau tidak lagi allocating
//! 3. Relocating - Object sedang dipindahkan ke region baru
//! 4. Relocated - Semua object sudah dipindahkan
//! 5. Free - Region kosong dan siap digunakan kembali
//!
//! Virtual Memory Features:
//! - Reserve address space besar di awal
//! - Commit physical memory on-demand
//! - Uncommit memory saat region kosong
//! - Multi-mapping untuk colored pointers

pub mod memory_mapping;
pub mod numa;
pub mod page;
pub mod region;
pub mod virtual_memory;

pub use memory_mapping::MemoryMapping;
pub use numa::NumaManager;
pub use page::Page;
pub use region::{Generation, Region, RegionState, RegionType};
pub use virtual_memory::VirtualMemory;

use crate::config::GcConfig;
use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Default alignment for TLAB allocations (8 bytes)
///
/// This is the minimum alignment for all allocations, ensuring:
/// - CPU efficiency (aligned access is faster)
/// - Atomic operation requirements (some architectures require alignment)
/// - Object header alignment consistency
const DEFAULT_ALIGNMENT: usize = 8;

/// Heap - container untuk semua regions
///
/// Heap mengelola seluruh virtual address space untuk GC.
/// Tidak menyimpan object secara langsung, tapi mengelola region-region.
///
/// Heap Structure:
/// ```
/// ┌─────────────────────────────────────────────────────┐
/// │                      Heap                            │
/// │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐ │
/// │  │ Region 1 │ │ Region 2 │ │ Region 3 │ │   ...   │ │
/// │  │ (Small)  │ │ (Medium) │ │ (Large)  │ │         │ │
/// │  └──────────┘ └──────────┘ └──────────┘ └─────────┘ │
/// └─────────────────────────────────────────────────────┘
/// ```
pub struct Heap {
    /// Base address dari reserved heap space
    base_address: usize,

    /// Ukuran maximum heap
    max_size: usize,

    /// Ukuran committed (physical memory yang digunakan)
    committed_size: AtomicUsize,

    /// Bump pointer untuk TLAB allocation (thread-safe)
    /// Pointer ini menunjuk ke alamat berikutnya yang akan dialokasikan
    alloc_ptr: AtomicUsize,

    /// Region allocation offset (untuk tracking region positions)
    region_offset: AtomicUsize,

    /// Region yang sedang digunakan
    active_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Free list untuk region yang bisa digunakan kembali
    free_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Young generation regions
    nursery_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Old generation regions
    old_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// NUMA manager untuk allocation affinity
    numa_manager: Option<NumaManager>,

    /// Virtual memory manager
    virtual_memory: std::sync::Mutex<VirtualMemory>,

    /// GC configuration
    config: Arc<GcConfig>,
}

impl Heap {
    /// Create new heap dengan konfigurasi tertentu
    ///
    /// # Arguments
    /// * `config` - GC configuration
    pub fn new(config: Arc<GcConfig>) -> Result<Self> {
        let virtual_memory = VirtualMemory::reserve(config.max_heap_size)?;
        let base_address = virtual_memory.base_address();

        let numa_manager = if config.numa_aware {
            Some(NumaManager::new())
        } else {
            None
        };

        let mut heap = Self {
            base_address,
            max_size: config.max_heap_size,
            committed_size: AtomicUsize::new(0),
            alloc_ptr: AtomicUsize::new(base_address),
            region_offset: AtomicUsize::new(0),
            active_regions: std::sync::Mutex::new(Vec::new()),
            free_regions: std::sync::Mutex::new(Vec::new()),
            nursery_regions: std::sync::Mutex::new(Vec::new()),
            old_regions: std::sync::Mutex::new(Vec::new()),
            numa_manager,
            virtual_memory: std::sync::Mutex::new(virtual_memory),
            config,
        };

        heap.initialize_regions()?;

        Ok(heap)
    }

    /// Initialize heap dengan regions awal
    fn initialize_regions(&mut self) -> Result<()> {
        let mut free_regions = self
            .free_regions
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("free_regions in initialize: {}", e)))?;

        for i in 0..4 {
            let offset = i * self.config.small_region_size;
            let start_address = self.base_address.saturating_add(offset);

            let region = Region::with_address(
                start_address,
                RegionType::Small,
                self.config.small_region_size,
                Generation::Young,
            )?;

            let region = Arc::new(region);
            free_regions.push(region);

            self.region_offset
                .fetch_add(self.config.small_region_size, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Allocate region untuk object baru
    ///
    /// # Arguments
    /// * `size` - Size object yang akan dialokasikan
    /// * `generation` - Generation (Young atau Old)
    pub fn allocate_region(&self, size: usize, generation: Generation) -> Result<Arc<Region>> {
        let region_type = if size <= self.config.small_threshold {
            RegionType::Small
        } else if size <= self.config.large_threshold {
            RegionType::Medium
        } else {
            RegionType::Large
        };

        // Try get dari free list
        {
            let mut free_regions = self
                .free_regions
                .lock()
                .map_err(|e| FgcError::LockPoisoned(format!("free_regions in allocate: {}", e)))?;

            for (i, region) in free_regions.iter().enumerate() {
                if region.region_type() == region_type && region.generation() == generation {
                    let region = free_regions.remove(i);
                    drop(free_regions); // Release lock before reset
                    region.reset()?;
                    return Ok(region);
                }
            }
        }

        self.create_new_region(region_type, generation)
    }

    /// Create region baru dari virtual memory
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. It uses a compare-and-swap (CAS) loop to
    /// atomically check and update `committed_size`, preventing race conditions
    /// where multiple threads could allocate regions simultaneously and exceed
    /// the maximum heap size.
    ///
    /// The CAS loop ensures that the check (committed_size + size <= max_size)
    /// and the update (committed_size += size) happen atomically. If another
    /// thread modifies `committed_size` between our load and CAS, we retry with
    /// the new value.
    ///
    /// # Memory Ordering
    ///
    /// - `Acquire` ordering on load: ensures we see all prior modifications to
    ///   committed_size and any associated state
    /// - `AcqRel` ordering on CAS success: acquire semantics for subsequent
    ///   operations, release semantics to make our update visible
    fn create_new_region(
        &self,
        region_type: RegionType,
        generation: Generation,
    ) -> Result<Arc<Region>> {
        let size = match region_type {
            RegionType::Small => self.config.small_region_size,
            RegionType::Medium => self.config.medium_region_size,
            RegionType::Large => self.config.large_threshold * 2,
        };

        let region_offset = loop {
            let current = self.committed_size.load(Ordering::Acquire);

            if current.saturating_add(size) > self.max_size {
                return Err(FgcError::OutOfMemory {
                    requested: size,
                    available: self.max_size.saturating_sub(current),
                });
            }

            match self.committed_size.compare_exchange_weak(
                current,
                current.saturating_add(size),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break current,
                Err(_) => continue,
            }
        };

        let start_address = self.base_address.saturating_add(region_offset);

        if let Ok(mut vm) = self.virtual_memory.lock() {
            let _ = vm.commit(region_offset, size);
        }

        let region = Region::with_address(start_address, region_type, size, generation)?;
        let region = Arc::new(region);

        Ok(region)
    }

    /// Return region ke free list
    pub fn return_region(&self, region: Arc<Region>) {
        region.reset().ok();

        if let Ok(mut free_regions) = self.free_regions.lock() {
            free_regions.push(region);
        }
        // If lock fails, region will be dropped - this is acceptable for error recovery
    }

    /// Flip mark bits untuk semua regions
    ///
    /// Dipanggil saat GC cycle baru dimulai.
    pub fn flip_mark_bits(&self) {
        if let Ok(active_regions) = self.active_regions.lock() {
            for region in active_regions.iter() {
                region.flip_mark_bits();
            }
        }
    }

    /// Get semua active regions
    pub fn get_active_regions(&self) -> Vec<Arc<Region>> {
        self.active_regions
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Get nursery regions (young generation)
    pub fn get_nursery_regions(&self) -> Vec<Arc<Region>> {
        self.nursery_regions
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Get old generation regions
    pub fn get_old_regions(&self) -> Vec<Arc<Region>> {
        self.old_regions
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Get heap statistics
    pub fn get_stats(&self) -> HeapStats {
        let (active_regions, free_regions) =
            match (self.active_regions.lock(), self.free_regions.lock()) {
                (Ok(a), Ok(f)) => (a.clone(), f.len()),
                _ => return HeapStats::default(),
            };

        let mut young_size: usize = 0;
        let mut old_size: usize = 0;
        let mut used_bytes: usize = 0;

        for region in active_regions.iter() {
            used_bytes = used_bytes.saturating_add(region.used());
            if region.generation() == Generation::Young {
                young_size = young_size.saturating_add(region.size());
            } else {
                old_size = old_size.saturating_add(region.size());
            }
        }

        HeapStats {
            used: used_bytes,
            committed: self.committed_size.load(Ordering::Relaxed),
            max: self.max_size,
            young_size,
            old_size,
            region_count: active_regions.len(),
            free_region_count: free_regions,
        }
    }

    /// Update heap statistics (setelah GC)
    pub fn update_stats(&self) {
        // Cleanup dan update statistics
        // Bisa trigger uncommit memory jika heap terlalu besar
    }

    /// Allocate TLAB (Thread-Local Allocation Buffer) memory using bump pointer.
    ///
    /// This function implements a thread-safe bump pointer allocator for fast
    /// memory allocation. The bump pointer is atomically incremented to ensure
    /// each allocation receives a unique, non-overlapping memory region.
    ///
    /// # Bump Pointer Mechanism
    ///
    /// The allocator maintains an atomic pointer (`alloc_ptr`) that points to the
    /// next available address. When allocating:
    ///
    /// 1. Read the current pointer value (start of allocation)
    /// 2. Calculate the new pointer value (current + aligned_size)
    /// 3. Atomically swap: if pointer hasn't changed, update it to new value
    /// 4. Return the old pointer value (start of allocated region)
    ///
    /// This ensures O(1) allocation time with proper synchronization.
    ///
    /// # Alignment
    ///
    /// All allocations are aligned to 8-byte boundaries minimum for:
    /// - CPU efficiency (aligned access is faster)
    /// - Atomic operation requirements (some architectures require alignment)
    /// - Object header alignment consistency
    ///
    /// # Zero-Size Allocations
    ///
    /// Zero-size allocations are treated as 1-byte allocations (aligned to 8 bytes).
    /// This ensures the bump pointer always advances, guaranteeing unique addresses
    /// for each allocation, even when size is 0.
    ///
    /// # Arguments
    /// * `size` - Requested allocation size in bytes
    ///
    /// # Returns
    /// * `Ok(usize)` - The starting address of the allocated memory region
    /// * `Err(FgcError::OutOfMemory)` - If heap is exhausted
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. Multiple threads can allocate simultaneously
    /// without contention beyond the atomic operation.
    pub fn allocate_tlab_memory(&self, size: usize) -> Result<usize> {
        self.allocate_tlab_memory_aligned(size, DEFAULT_ALIGNMENT)
    }

    /// Allocate TLAB memory with custom alignment.
    ///
    /// This function implements a thread-safe bump pointer allocator with support
    /// for custom alignment requirements. The bump pointer is atomically incremented
    /// to ensure each allocation receives a unique, properly aligned memory region.
    ///
    /// # Alignment Requirements
    ///
    /// - `alignment` must be a power of 2
    /// - Minimum alignment is 8 bytes (enforced automatically)
    /// - Both the returned address and allocation size are aligned to the specified boundary
    ///
    /// # Bump Pointer Mechanism
    ///
    /// The allocator maintains an atomic pointer (`alloc_ptr`) that points to the
    /// next available address. When allocating:
    ///
    /// 1. Read the current pointer value
    /// 2. Align the current pointer to the alignment boundary
    /// 3. Calculate the new pointer value (aligned_current + aligned_size)
    /// 4. Atomically swap: if pointer hasn't changed, update it to new value
    /// 5. Return the aligned pointer value (start of allocated region)
    ///
    /// This ensures O(1) allocation time with proper synchronization and alignment.
    ///
    /// # Arguments
    /// * `size` - Requested allocation size in bytes
    /// * `alignment` - Required alignment in bytes (must be power of 2, minimum 8)
    ///
    /// # Returns
    /// * `Ok(usize)` - The starting address of the allocated memory region (properly aligned)
    /// * `Err(FgcError::TlabError)` - If alignment is not a power of 2
    /// * `Err(FgcError::OutOfMemory)` - If heap is exhausted
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. Multiple threads can allocate simultaneously
    /// without contention beyond the atomic operation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Allocate with default 8-byte alignment
    /// let addr = heap.allocate_tlab_memory_aligned(64, 8)?;
    ///
    /// // Allocate with 32-byte alignment for SIMD operations
    /// let addr = heap.allocate_tlab_memory_aligned(128, 32)?;
    ///
    /// // Allocate with 64-byte alignment for cache line alignment
    /// let addr = heap.allocate_tlab_memory_aligned(256, 64)?;
    /// ```
    pub fn allocate_tlab_memory_aligned(&self, size: usize, alignment: usize) -> Result<usize> {
        // Validate alignment is a power of 2
        if !alignment.is_power_of_two() {
            return Err(FgcError::TlabError(format!(
                "Alignment must be a power of 2, got {}",
                alignment
            )));
        }

        // Enforce minimum alignment of 8 bytes
        let effective_alignment = alignment.max(DEFAULT_ALIGNMENT);

        // Validate size BEFORE any arithmetic to prevent overflow
        if size > self.max_size {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: self.max_size,
            });
        }

        // Ensure minimum allocation size of 1 byte to advance bump pointer.
        // This prevents zero-size allocations from returning the same address.
        let effective_size = size.max(1);

        // Align size to alignment boundary
        // Formula: (size + alignment - 1) & !(alignment - 1) rounds up to nearest multiple
        // Use saturating arithmetic to prevent overflow
        let aligned_size = effective_size
            .saturating_add(effective_alignment)
            .wrapping_sub(1)
            & !(effective_alignment - 1);

        // Calculate heap limit (base + max_size) using saturating arithmetic
        let limit = self.base_address.saturating_add(self.max_size);

        // Loop for CAS-based atomic allocation
        loop {
            // Load current bump pointer value
            let current = self.alloc_ptr.load(Ordering::SeqCst);

            // Align current pointer to alignment boundary
            // Formula: (ptr + alignment - 1) & !(alignment - 1)
            let aligned_current = current.saturating_add(effective_alignment).wrapping_sub(1)
                & !(effective_alignment - 1);

            // Calculate the next allocation position using saturating arithmetic
            let next = aligned_current.saturating_add(aligned_size);

            // Check for Out Of Memory before attempting allocation
            if next > limit || next == usize::MAX {
                let available = if current > limit {
                    0
                } else {
                    limit.saturating_sub(current)
                };
                return Err(FgcError::OutOfMemory {
                    requested: size,
                    available,
                });
            }

            // Atomically try to increment the bump pointer
            // compare_exchange: if current == alloc_ptr, set to next and return Ok(current)
            // otherwise, return Err(actual_value) which we'll retry with
            match self.alloc_ptr.compare_exchange_weak(
                current,
                next,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    // Success! Return the ALIGNED pointer value (start of allocation)
                    return Ok(aligned_current);
                }
                Err(_) => {
                    // Another thread modified alloc_ptr, retry with new value
                    continue;
                }
            }
        }
    }

    /// Get base address
    pub fn base_address(&self) -> usize {
        self.base_address
    }

    /// Get max heap size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get committed size
    pub fn committed_size(&self) -> usize {
        self.committed_size.load(Ordering::Relaxed)
    }
}

/// Heap statistics
#[derive(Debug, Default)]
pub struct HeapStats {
    /// Memory yang sedang digunakan (bytes)
    pub used: usize,
    /// Memory yang sudah committed (bytes)
    pub committed: usize,
    /// Memory maksimum (bytes)
    pub max: usize,
    /// Young generation size
    pub young_size: usize,
    /// Old generation size
    pub old_size: usize,
    /// Jumlah active regions
    pub region_count: usize,
    /// Jumlah free regions
    pub free_region_count: usize,
}
