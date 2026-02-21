//! Generational Allocator - Young/Old Generation Management
//!
//! Manages generational allocation (young/old generation).
//! Based on the observation that:
//! - Most objects die young
//! - Objects that survive tend to live long

use crate::allocator::bump::MultiBumpAllocator;
use crate::allocator::tlab::{Tlab, TlabManager, ThreadId};
use crate::error::Result;
use crate::heap::Heap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use indexmap::IndexMap;

/// GenerationalAllocator - allocator with young/old separation
///
/// Manages allocation for both generations:
/// - Young generation: Bump pointer + TLAB for fast allocation
/// - Old generation: Bump pointer for promoted objects
pub struct GenerationalAllocator {
    /// Young generation bump allocator
    young_allocator: MultiBumpAllocator,

    /// Old generation bump allocator
    old_allocator: MultiBumpAllocator,

    /// TLAB manager for young generation
    tlab_manager: TlabManager,

    /// Heap reference for TLAB allocation
    heap: Arc<Heap>,

    /// Young generation size (bytes)
    young_size: AtomicUsize,

    /// Old generation size (bytes)
    old_size: AtomicUsize,

    /// Promotion count (objects promoted to old)
    promotion_count: AtomicUsize,

    /// Tenure threshold (survive N minor GCs before promote)
    tenure_threshold: u8,
}

impl GenerationalAllocator {
    /// Create new generational allocator
    ///
    /// # Arguments
    /// * `heap` - Heap reference
    /// * `young_ratio` - Ratio of heap for young generation (0.0-1.0)
    /// * `tenure_threshold` - Survives before promotion
    pub fn new(heap: Arc<Heap>, young_ratio: f32, tenure_threshold: u8) -> Self {
        let heap_size = heap.max_size();
        let young_size = (heap_size as f32 * young_ratio) as usize;
        let old_size = heap_size - young_size;

        Self {
            young_allocator: MultiBumpAllocator::new(2 * 1024 * 1024, 8, 100),
            old_allocator: MultiBumpAllocator::new(32 * 1024 * 1024, 8, 50),
            tlab_manager: TlabManager::new(256 * 1024, 16 * 1024, 2 * 1024 * 1024, 8, 1000),
            heap,
            young_size: AtomicUsize::new(young_size),
            old_size: AtomicUsize::new(old_size),
            promotion_count: AtomicUsize::new(0),
            tenure_threshold,
        }
    }

    /// Allocate in young generation (fast path)
    ///
    /// Default allocation for new objects.
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    pub fn allocate_young(&self, size: usize) -> Result<usize> {
        let thread_id = self.get_current_thread_id();

        if let Ok(tlab) = self.tlab_manager.get_or_create_tlab(thread_id, &self.heap) {
            if tlab.has_space(size) {
                if let Ok(addr) = tlab.allocate(size) {
                    return Ok(addr);
                }
            }
        }

        self.young_allocator.allocate(size)
    }

    /// Allocate in old generation
    ///
    /// Used for promoted objects and large objects.
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    pub fn allocate_old(&self, size: usize) -> Result<usize> {
        self.old_allocator.allocate(size)
    }

    /// Allocate with generational heuristic
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    /// * `prefer_young` - If true, try young generation first
    pub fn allocate(&self, size: usize, prefer_young: bool) -> Result<usize> {
        if prefer_young {
            self.allocate_young(size)
        } else {
            self.allocate_old(size)
        }
    }

    /// Promote object from young to old generation
    ///
    /// Called when object survives minor GC.
    ///
    /// # Arguments
    /// * `_old_address` - Object address in young generation
    /// * `size` - Object size
    ///
    /// # Returns
    /// New address in old generation
    pub fn promote_object(&self, _old_address: usize, size: usize) -> Result<usize> {
        let new_address = self.allocate_old(size)?;
        self.promotion_count.fetch_add(1, Ordering::Relaxed);
        Ok(new_address)
    }

    /// Minor GC - collect young generation
    ///
    /// Called when young generation is full.
    ///
    /// # Returns
    /// Estimated bytes reclaimed
    pub fn minor_gc(&self) -> usize {
        self.young_allocator.reset_all();
        self.young_size.load(Ordering::Relaxed)
    }

    /// Major GC - collect old generation
    ///
    /// Called when old generation is nearly full.
    ///
    /// # Returns
    /// Estimated bytes reclaimed
    pub fn major_gc(&self) -> usize {
        self.old_allocator.reset_all();
        self.old_size.load(Ordering::Relaxed) / 2
    }

    /// Full GC - collect both generations
    ///
    /// Called when heap is nearly full.
    pub fn full_gc(&self) -> usize {
        let young_reclaimed = self.minor_gc();
        let old_reclaimed = self.major_gc();
        young_reclaimed + old_reclaimed
    }

    /// Get TLAB for current thread
    pub fn get_current_tlab(&self) -> Option<Arc<Tlab>> {
        let thread_id = self.get_current_thread_id();
        self.tlab_manager
            .get_or_create_tlab(thread_id, &self.heap)
            .ok()
    }

    /// Refill TLAB for current thread
    pub fn refill_tlab(&self) -> Result<Arc<Tlab>> {
        let thread_id = self.get_current_thread_id();
        self.tlab_manager.refill_tlab(thread_id, &self.heap)
    }

    /// Get young generation size
    pub fn young_size(&self) -> usize {
        self.young_size.load(Ordering::Relaxed)
    }

    /// Get old generation size
    pub fn old_size(&self) -> usize {
        self.old_size.load(Ordering::Relaxed)
    }

    /// Get young generation usage
    pub fn young_usage(&self) -> usize {
        self.young_allocator.total_allocated()
    }

    /// Get old generation usage
    pub fn old_usage(&self) -> usize {
        self.old_allocator.total_allocated()
    }

    /// Get promotion count
    pub fn promotion_count(&self) -> usize {
        self.promotion_count.load(Ordering::Relaxed)
    }

    /// Get tenure threshold
    pub fn tenure_threshold(&self) -> u8 {
        self.tenure_threshold
    }

    /// Set tenure threshold
    pub fn set_tenure_threshold(&mut self, threshold: u8) {
        self.tenure_threshold = threshold;
    }

    /// Get statistics
    pub fn stats(&self) -> GenerationalStats {
        GenerationalStats {
            young_size: self.young_size(),
            old_size: self.old_size(),
            young_used: self.young_usage(),
            old_used: self.old_usage(),
            promotion_count: self.promotion_count(),
            tlab_count: self.tlab_manager.active_tlab_count(),
            tlab_refills: self.tlab_manager.total_refills(),
        }
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
}

/// Statistics for generational allocator
#[derive(Debug, Default)]
pub struct GenerationalStats {
    /// Young generation size (bytes)
    pub young_size: usize,
    /// Old generation size (bytes)
    pub old_size: usize,
    /// Young generation used (bytes)
    pub young_used: usize,
    /// Old generation used (bytes)
    pub old_used: usize,
    /// Objects promoted to old generation
    pub promotion_count: usize,
    /// Active TLABs
    pub tlab_count: usize,
    /// TLAB refills
    pub tlab_refills: usize,
}

/// Object age tracker for tenure decision
///
/// Tracks how many times an object survives minor GC.
pub struct AgeTracker {
    ages: std::sync::Mutex<IndexMap<usize, u8>>,
}

impl AgeTracker {
    /// Create new age tracker
    pub fn new() -> Self {
        Self {
            ages: std::sync::Mutex::new(IndexMap::new()),
        }
    }

    /// Increment age for object
    pub fn increment_age(&self, address: usize) -> u8 {
        let mut ages = self.ages.lock().unwrap();
        let age = ages.entry(address).or_insert(0);
        *age += 1;
        *age
    }

    /// Get age for object
    pub fn get_age(&self, address: usize) -> u8 {
        let ages = self.ages.lock().unwrap();
        *ages.get(&address).unwrap_or(&0)
    }

    /// Remove object from tracker
    pub fn remove(&self, address: usize) {
        let mut ages = self.ages.lock().unwrap();
        ages.swap_remove(&address);
    }

    /// Clear all ages (after major GC)
    pub fn clear(&self) {
        let mut ages = self.ages.lock().unwrap();
        ages.clear();
    }
}

impl Default for AgeTracker {
    fn default() -> Self {
        Self::new()
    }
}
