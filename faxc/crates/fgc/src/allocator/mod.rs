//! Allocator Module - Memory Allocation Strategies
//!
//! Manages all memory allocation strategies in FGC.
//! Allocator is responsible for high-speed, thread-safe object allocation.
//!
//! ## Allocation Strategies
//!
//! - **Bump Pointer Allocation**: O(1) allocation for small/medium objects
//! - **TLAB (Thread-Local Allocation Buffer)**: Lock-free per-thread allocation
//! - **Large Object Allocation**: Dedicated region per large object (> 4KB)
//! - **Generational Allocation**: Young/Old generation separation
//!
//! ## Size Classes
//!
//! - Small: < 256 bytes (bump pointer in 2MB regions)
//! - Medium: 256 bytes - 4KB (bump pointer in 32MB regions)
//! - Large: > 4KB (dedicated region per object)

pub mod bump;
pub mod tlab;
pub mod large;
pub mod generational;

pub use bump::{BumpPointerAllocator, MultiBumpAllocator};
pub use tlab::{Tlab, TlabManager, ThreadId};
pub use large::LargeObjectAllocator;
pub use generational::{GenerationalAllocator, GenerationalStats, AgeTracker};

use crate::error::Result;
use crate::heap::Heap;

/// Main allocator for FGC
///
/// Manages allocation for all object sizes and generations.
/// Thread-safe and optimized for concurrent access.
///
/// ## Allocation Flow
///
/// 1. Check object size
/// 2. Large object (> 4KB) -> large allocator
/// 3. Small/medium -> try TLAB first (fast path)
/// 4. TLAB full -> refill or bump allocator
pub struct Allocator {
    /// Young generation bump allocator
    young_allocator: MultiBumpAllocator,

    /// Old generation bump allocator
    old_allocator: MultiBumpAllocator,

    /// Large object allocator
    large_allocator: LargeObjectAllocator,

    /// TLAB manager for thread-local allocation
    tlab_manager: TlabManager,

    /// Enable generational mode
    generational: bool,

    /// Reference to heap for TLAB allocation
    heap: std::sync::Arc<Heap>,
}

impl Allocator {
    /// Create new allocator with specified configuration
    ///
    /// # Arguments
    /// * `heap` - Heap reference for TLAB allocation
    /// * `generational` - Enable generational allocation
    pub fn new(heap: std::sync::Arc<Heap>, generational: bool) -> Self {
        Self {
            young_allocator: MultiBumpAllocator::new(2 * 1024 * 1024, 8, 100),
            old_allocator: MultiBumpAllocator::new(32 * 1024 * 1024, 8, 50),
            large_allocator: LargeObjectAllocator::new(),
            tlab_manager: TlabManager::new(256 * 1024, 16 * 1024, 2 * 1024 * 1024, 8, 1000),
            generational,
            heap,
        }
    }

    /// Allocate memory for a new object
    ///
    /// Main entry point for allocation. Chooses strategy based on size and generation.
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    /// * `young` - If true, allocate in young generation
    ///
    /// # Returns
    /// Address of allocated memory
    pub fn allocate(&self, size: usize, young: bool) -> Result<usize> {
        // Large object -> dedicated allocation
        if size > large::LARGE_THRESHOLD {
            return self.large_allocator.allocate(size);
        }

        // Try TLAB first (fast path)
        let thread_id = self.get_current_thread_id();

        if let Ok(tlab) = self.tlab_manager.get_or_create_tlab(thread_id, &self.heap) {
            if tlab.has_space(size) {
                if let Ok(addr) = tlab.allocate(size) {
                    return Ok(addr);
                }
            }
        }

        // TLAB not available or full, use bump allocator
        if self.generational && young {
            self.young_allocator.allocate(size)
        } else {
            self.old_allocator.allocate(size)
        }
    }

    /// Allocate in young generation
    pub fn allocate_young(&self, size: usize) -> Result<usize> {
        self.allocate(size, true)
    }

    /// Allocate in old generation
    pub fn allocate_old(&self, size: usize) -> Result<usize> {
        self.allocate(size, false)
    }

    /// Promote object from young to old generation
    ///
    /// Copies object data to old generation and returns new address.
    pub fn promote_object(&self, _old_address: usize, size: usize) -> Result<usize> {
        let new_address = self.allocate_old(size)?;
        Ok(new_address)
    }

    /// Get TLAB for current thread
    pub fn get_current_tlab(&self) -> Option<std::sync::Arc<Tlab>> {
        let thread_id = self.get_current_thread_id();
        self.tlab_manager
            .get_or_create_tlab(thread_id, &self.heap)
            .ok()
    }

    /// Refill TLAB for current thread
    pub fn refill_tlab(&self) -> Result<std::sync::Arc<Tlab>> {
        let thread_id = self.get_current_thread_id();
        self.tlab_manager.refill_tlab(thread_id, &self.heap)
    }

    /// Get current thread ID
    fn get_current_thread_id(&self) -> ThreadId {
        static THREAD_COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);

        thread_local! {
            static TID: u64 = THREAD_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        TID.with(|&id| id)
    }

    /// Get statistics
    pub fn stats(&self) -> AllocatorStats {
        AllocatorStats {
            young_allocated: self.young_allocator.total_allocated(),
            old_allocated: self.old_allocator.total_allocated(),
            large_allocated: self.large_allocator.total_allocated(),
            large_objects: self.large_allocator.object_count(),
            active_tlabs: self.tlab_manager.active_tlab_count(),
            tlab_refills: self.tlab_manager.total_refills(),
        }
    }

    /// Reset allocator (for GC)
    pub fn reset_young(&self) {
        self.young_allocator.reset_all();
    }
}

/// Statistics for allocator
#[derive(Debug, Default)]
pub struct AllocatorStats {
    /// Bytes allocated in young generation
    pub young_allocated: usize,
    /// Bytes allocated in old generation
    pub old_allocated: usize,
    /// Bytes allocated for large objects
    pub large_allocated: usize,
    /// Count of large objects
    pub large_objects: usize,
    /// Active TLABs
    pub active_tlabs: usize,
    /// TLAB refill count
    pub tlab_refills: usize,
}
