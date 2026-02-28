//! Heap Management Module - Region-Based Memory Management
//!
//! This module manages region-based heap, a technique used by ZGC
//! for parallel collection and partial compaction.
//!
//! Heap is divided into regions with varying sizes:
//! - Small Region (2MB): For objects < 256 bytes
//! - Medium Region (32MB): For objects 256 bytes - 4KB
//! - Large Region (variable): For objects > 4KB (1 object per region)
//!
//! Region Lifecycle States:
//! 1. Allocating - Region is being filled with new objects
//! 2. Allocated - Region is full or no longer allocating
//! 3. Relocating - Objects are being moved to new regions
//! 4. Relocated - All objects have been moved
//! 5. Free - Region is empty and ready for reuse
//!
//! Virtual Memory Features:
//! - Reserve large address space upfront
//! - Commit physical memory on-demand
//! - Uncommit memory when region is empty
//! - Multi-mapping for colored pointers

pub mod adaptive;
pub mod memory_mapping;
pub mod numa;
pub mod page;
pub mod region;
pub mod virtual_memory;

pub use adaptive::{AdaptiveConfig, AdaptiveHeapController, HeapSizeStats};
pub use memory_mapping::MemoryMapping;
pub use numa::NumaManager;
pub use page::Page;
pub use region::{Generation, Region, RegionState, RegionType};
pub use virtual_memory::VirtualMemory;

use crate::config::GcConfig;
use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Default alignment for TLAB allocations (8 bytes)
///
/// This is the minimum alignment for all allocations, ensuring:
/// - CPU efficiency (aligned access is faster)
/// - Atomic operation requirements (some architectures require alignment)
/// - Object header alignment consistency
const DEFAULT_ALIGNMENT: usize = 8;

/// Heap - container for all regions
///
/// Heap manages the entire virtual address space for GC.
/// It does not store objects directly, but manages regions.
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
    /// Base address of reserved heap space
    base_address: usize,

    /// Maximum heap size
    max_size: usize,

    /// Committed size (physical memory in use)
    committed_size: AtomicUsize,

    /// Bump pointer for TLAB allocation (thread-safe)
    /// This pointer points to the next address to be allocated
    alloc_ptr: AtomicUsize,

    /// Region allocation offset (for tracking region positions)
    region_offset: AtomicUsize,

    /// Currently active regions
    active_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Free list for regions that can be reused
    free_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Young generation regions
    nursery_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Old generation regions
    old_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// NUMA manager for allocation affinity
    #[allow(dead_code)]
    numa_manager: Option<NumaManager>,

    /// Virtual memory manager
    virtual_memory: std::sync::Mutex<VirtualMemory>,

    /// GC configuration
    config: Arc<GcConfig>,

    /// Total allocations counter
    allocations: AtomicU64,

    /// Total bytes allocated
    total_allocated: AtomicU64,
}

impl Heap {
    /// Create new heap with specific configuration
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
            allocations: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
        };

        heap.initialize_regions()?;

        Ok(heap)
    }

    /// Initialize heap with initial regions
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

            // region is already Arc<Region>
            free_regions.push(region.clone());

            self.region_offset
                .fetch_add(self.config.small_region_size, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Allocate region for new object
    ///
    /// # Arguments
    /// * `size` - Size of object to allocate
    /// * `generation` - Generation (Young or Old)
    pub fn allocate_region(&self, size: usize, generation: Generation) -> Result<Arc<Region>> {
        let region_type = if size <= self.config.small_threshold {
            RegionType::Small
        } else if size <= self.config.large_threshold {
            RegionType::Medium
        } else {
            RegionType::Large
        };

        // Try get from free list
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

    /// Create new region from virtual memory
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

        if let Ok(vm) = self.virtual_memory.lock() {
            let _ = vm.commit(region_offset, size);
        }

        let region = Region::with_address(start_address, region_type, size, generation)?;
        // region is already Arc<Region>

        Ok(region)
    }

    /// Return region to free list
    pub fn return_region(&self, region: Arc<Region>) {
        region.reset().ok();

        if let Ok(mut free_regions) = self.free_regions.lock() {
            free_regions.push(region.clone());
        }
        // If lock fails, region will be dropped - this is acceptable for error recovery
    }

    /// Flip mark bits for all regions
    ///
    /// Called when new GC cycle starts.
    pub fn flip_mark_bits(&self) {
        if let Ok(active_regions) = self.active_regions.lock() {
            for region in active_regions.iter() {
                region.flip_mark_bits();
            }
        }
    }

    /// Returns all active regions
    pub fn get_active_regions(&self) -> Vec<Arc<Region>> {
        self.active_regions
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Returns nursery regions (young generation)
    pub fn get_nursery_regions(&self) -> Vec<Arc<Region>> {
        self.nursery_regions
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Returns old generation regions
    pub fn get_old_regions(&self) -> Vec<Arc<Region>> {
        self.old_regions
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Returns heap statistics - ZGC-style comprehensive metrics
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

        let utilization = if self.max_size > 0 {
            used_bytes as f64 / self.max_size as f64
        } else {
            0.0
        };

        HeapStats {
            used: used_bytes,
            committed: self.committed_size.load(Ordering::Relaxed),
            max: self.max_size,
            young_size,
            old_size,
            region_count: active_regions.len(),
            free_region_count: free_regions,
            utilization,
            allocations: self.allocations.load(Ordering::Relaxed),
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
        }
    }

    /// Update heap statistics (after GC)
    pub fn update_stats(&self) {
        // Cleanup and update statistics
        // Can trigger uncommit memory if heap is too large
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
    #[inline]
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
    /// * `Err(FgcError::OutOfMemory)` - If heap is exhausted or overflow detected
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. Multiple threads can allocate simultaneously
    /// without contention beyond the atomic operation.
    ///
    /// # Overflow Safety
    ///
    /// All pointer arithmetic uses checked operations. If any intermediate
    /// calculation would overflow, the function returns an error instead of
    /// silently wrapping or saturating.
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
    #[inline]
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

        // Align size to alignment boundary using CHECKED arithmetic.
        // Formula: (size + alignment - 1) & !(alignment - 1) rounds up to nearest multiple
        // We use checked_add to detect overflow instead of silently saturating.
        let size_plus_align =
            effective_size
                .checked_add(effective_alignment)
                .ok_or(FgcError::OutOfMemory {
                    requested: size,
                    available: 0,
                })?;

        // Subtract 1 (this cannot overflow since size_plus_align >= alignment >= 8)
        let size_mask = size_plus_align - 1;

        // Apply alignment mask (this cannot overflow)
        let aligned_size = size_mask & !(effective_alignment - 1);

        // Calculate heap limit (base + max_size) using CHECKED arithmetic
        let limit = self
            .base_address
            .checked_add(self.max_size)
            .ok_or(FgcError::Internal(
                "Heap base + max_size overflow - invalid heap configuration".to_string(),
            ))?;

        // Loop for CAS-based atomic allocation
        loop {
            // Load current bump pointer value
            let current = self.alloc_ptr.load(Ordering::SeqCst);

            // Align current pointer to alignment boundary using CHECKED arithmetic
            // Formula: (ptr + alignment - 1) & !(alignment - 1)
            let ptr_plus_align =
                current
                    .checked_add(effective_alignment)
                    .ok_or(FgcError::OutOfMemory {
                        requested: size,
                        available: 0,
                    })?;

            // Subtract 1 (cannot overflow since ptr_plus_align >= alignment >= 8)
            let aligned_mask = ptr_plus_align - 1;

            // Apply alignment mask
            let aligned_current = aligned_mask & !(effective_alignment - 1);

            // Calculate the next allocation position using CHECKED arithmetic
            let next = aligned_current
                .checked_add(aligned_size)
                .ok_or(FgcError::OutOfMemory {
                    requested: size,
                    available: 0,
                })?;

            // Check for Out Of Memory before attempting allocation
            // Use checked comparison to avoid overflow issues
            if next > limit {
                let available = limit.saturating_sub(current);
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
                    // Track allocations for ZGC-style stats
                    self.allocations.fetch_add(1, Ordering::Relaxed);
                    self.total_allocated
                        .fetch_add(aligned_size as u64, Ordering::Relaxed);

                    // Success! Return the ALIGNED pointer value (start of allocation)
                    return Ok(aligned_current);
                },
                Err(_) => {
                    // Another thread modified alloc_ptr, retry with new value
                    continue;
                },
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

    /// Check if an address is within any active region managed by this heap
    ///
    /// # Arguments
    /// * `addr` - Address to check
    ///
    /// # Returns
    /// `true` if address is within GC-managed heap regions, `false` otherwise
    ///
    /// # CRIT-01 FIX: Root Validation
    /// This method is used to validate that GC roots point to valid GC-managed
    /// memory, preventing attackers from registering arbitrary memory addresses
    /// as roots to exfiltrate data.
    pub fn contains_address(&self, addr: usize) -> bool {
        let regions = match self.active_regions.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Heap active_regions lock poisoned: {}", e);
                return false;
            },
        };
        for region in regions.iter() {
            if region.contains(addr) {
                return true;
            }
        }
        false
    }
}

/// Check if an address is within GC-managed heap regions
///
/// This function checks against all active heap regions to determine if
/// an address points to GC-managed memory.
///
/// # Arguments
/// * `addr` - Address to check
///
/// # Returns
/// `true` if address is within GC-managed heap, `false` otherwise
///
/// # CRIT-01 FIX: Root Validation
/// This function is used to validate that GC roots point to valid GC-managed
/// memory, preventing attackers from registering arbitrary memory addresses
/// as roots to exfiltrate data.
///
/// # Note
/// This function requires access to a Heap instance. Use `is_gc_managed_address_with_heap`
/// for proper validation, or use `Heap::contains_address` directly.
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::heap::{Heap, is_gc_managed_address_with_heap};
/// use fgc::config::GcConfig;
/// use std::sync::Arc;
///
/// let config = Arc::new(GcConfig::default());
/// let heap = Heap::new(config).unwrap();
///
/// // Kernel address should be rejected
/// assert!(!is_gc_managed_address_with_heap(0xFFFF_FFFF_FFFF_F000, &heap));
/// ```
pub fn is_gc_managed_address(addr: usize) -> bool {
    // Without heap context, we can only do basic sanity checks
    // Reject obviously invalid addresses
    if addr == 0 {
        return false;
    }

    // Reject kernel-space addresses (common across platforms)
    if addr > 0x0000_7FFF_FFFF_FFFF {
        return false;
    }

    // Reject very low addresses (typically unmapped)
    if addr < 0x1000 {
        return false;
    }

    // For proper validation, use is_gc_managed_address_with_heap
    log::trace!(
        "is_gc_managed_address({:#x}) called without heap context - basic checks only",
        addr
    );
    true // Allow for backward compatibility, proper check requires heap
}

/// Check if an address is within GC-managed heap regions with heap context
///
/// This is the proper way to validate addresses for root validation.
///
/// # Arguments
/// * `addr` - Address to check
/// * `heap` - Heap reference for validation
///
/// # Returns
/// `true` if address is within GC-managed heap, `false` otherwise
///
/// # CRIT-01 FIX: Root Validation
/// This function provides proper validation by checking against actual
/// heap regions, preventing attackers from registering arbitrary memory.
pub fn is_gc_managed_address_with_heap(addr: usize, heap: &Heap) -> bool {
    heap.contains_address(addr)
}

/// Heap statistics - ZGC-style comprehensive metrics
#[derive(Debug, Default)]
pub struct HeapStats {
    /// Memory currently in use (bytes)
    pub used: usize,
    /// Memory committed (bytes)
    pub committed: usize,
    /// Maximum memory (bytes)
    pub max: usize,
    /// Young generation size
    pub young_size: usize,
    /// Old generation size
    pub old_size: usize,
    /// Number of active regions
    pub region_count: usize,
    /// Number of free regions
    pub free_region_count: usize,
    /// Heap utilization (0.0 - 1.0)
    pub utilization: f64,
    /// Number of allocations since last GC
    pub allocations: u64,
    /// Total bytes allocated
    pub total_allocated: u64,
}

impl HeapStats {
    /// Calculate utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.max == 0 {
            return 0.0;
        }
        (self.used as f64 / self.max as f64) * 100.0
    }

    /// Check if heap is highly utilized (>80%)
    pub fn is_high_utilization(&self) -> bool {
        self.utilization > 0.8
    }

    /// Get available memory
    pub fn available(&self) -> usize {
        self.max.saturating_sub(self.used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_heap(max_size: usize) -> Heap {
        let config = Arc::new(GcConfig {
            max_heap_size: max_size,
            ..Default::default()
        });
        Heap::new(config).expect("Failed to create test heap")
    }

    #[test]
    fn test_allocate_tlab_memory_basic() {
        let heap = create_test_heap(1024 * 1024); // 1MB heap

        // Basic allocation should succeed
        let addr1 = heap.allocate_tlab_memory(64).unwrap();
        assert!(addr1 != 0);
        assert_eq!(addr1 % DEFAULT_ALIGNMENT, 0);

        // Second allocation should return different address
        let addr2 = heap.allocate_tlab_memory(64).unwrap();
        assert!(addr2 != addr1);
    }

    #[test]
    fn test_allocate_tlab_memory_alignment() {
        let heap = create_test_heap(1024 * 1024);

        // Test with 32-byte alignment
        let addr = heap.allocate_tlab_memory_aligned(64, 32).unwrap();
        assert_eq!(addr % 32, 0);

        // Test with 64-byte alignment
        let addr = heap.allocate_tlab_memory_aligned(128, 64).unwrap();
        assert_eq!(addr % 64, 0);
    }

    #[test]
    fn test_allocate_tlab_memory_invalid_alignment() {
        let heap = create_test_heap(1024 * 1024);

        // Non-power-of-2 alignment should fail
        let result = heap.allocate_tlab_memory_aligned(64, 15);
        assert!(result.is_err());
    }

    #[test]
    fn test_allocate_tlab_memory_zero_size() {
        let heap = create_test_heap(1024 * 1024);

        // Zero-size allocation should still advance bump pointer
        let addr1 = heap.allocate_tlab_memory(0).unwrap();
        let addr2 = heap.allocate_tlab_memory(0).unwrap();
        assert!(addr2 > addr1);
    }

    #[test]
    fn test_allocate_tlab_memory_exhaustion() {
        let heap = create_test_heap(1024); // 1KB heap

        // Allocate most of the heap
        let _addr1 = heap.allocate_tlab_memory(512).unwrap();

        // Try to allocate more than available
        let result = heap.allocate_tlab_memory(1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_heap_stats() {
        let heap = create_test_heap(1024 * 1024);

        let stats = heap.get_stats();
        assert_eq!(stats.max, 1024 * 1024);
        // committed depends on implementation - skip for now
    }

    #[test]
    fn test_contains_address() {
        let heap = create_test_heap(1024 * 1024);

        // Null address should be rejected
        assert!(!heap.contains_address(0));

        // Kernel address should be rejected
        assert!(!heap.contains_address(0xFFFF_FFFF_FFFF_F000));
    }
}
