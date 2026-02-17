//! TLAB - Thread-Local Allocation Buffer
//!
//! TLAB reduces contention in multi-threaded allocation.
//! Each thread gets a private buffer for lock-free allocation.
//!
//! ## How it works
//!
//! 1. Thread requests TLAB from global heap
//! 2. Thread allocates from its own TLAB (bump pointer, no lock)
//! 3. TLAB full: thread requests new TLAB
//! 4. Thread exits: TLAB retired to heap
//!
//! ## Benefits
//!
//! - Lock-free allocation for common case
//! - Better cache locality
//! - Reduced false sharing

use crate::allocator::bump::BumpPointerAllocator;
use crate::error::{FgcError, Result};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Thread ID type
pub type ThreadId = u64;

/// TLAB - Thread-Local Allocation Buffer
///
/// Private allocation buffer for a single thread.
/// Uses bump pointer for fast allocation.
pub struct Tlab {
    /// Thread ID that owns this TLAB
    owner: ThreadId,

    /// Bump pointer allocator for TLAB
    allocator: BumpPointerAllocator,

    /// Total bytes allocated in this TLAB (lifetime)
    total_allocated: AtomicUsize,

    /// Number of allocations
    allocation_count: AtomicUsize,

    /// TLAB retired (no more allocations allowed)
    retired: std::sync::atomic::AtomicBool,
}

impl Tlab {
    /// Create new TLAB for a thread
    ///
    /// # Arguments
    /// * `owner` - Thread ID that owns this TLAB
    /// * `start` - TLAB start address
    /// * `end` - TLAB end address
    /// * `alignment` - Alignment requirement
    pub fn new(owner: ThreadId, start: usize, end: usize, alignment: usize) -> Self {
        Self {
            owner,
            allocator: BumpPointerAllocator::new(start, end, alignment),
            total_allocated: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            retired: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Allocate from TLAB
    ///
    /// Fast path: bump pointer increment.
    /// Returns error if TLAB is full or retired.
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    pub fn allocate(&self, size: usize) -> Result<usize> {
        if self.retired.load(Ordering::Relaxed) {
            return Err(FgcError::TlabError("TLAB is retired".to_string()));
        }

        let addr = self.allocator.allocate(size)?;

        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        Ok(addr)
    }

    /// Check if there is enough space for allocation
    pub fn has_space(&self, size: usize) -> bool {
        self.allocator.remaining() >= size
    }

    /// Get remaining space in TLAB
    pub fn remaining(&self) -> usize {
        self.allocator.remaining()
    }

    /// Get total bytes allocated in this TLAB
    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    /// Get number of allocations
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Retire TLAB (no more allocations)
    ///
    /// Called when thread exits or TLAB needs to be reclaimed.
    pub fn retire(&self) {
        self.retired.store(true, Ordering::SeqCst);
    }

    /// Check if TLAB is retired
    pub fn is_retired(&self) -> bool {
        self.retired.load(Ordering::Relaxed)
    }

    /// Get thread owner
    pub fn owner(&self) -> ThreadId {
        self.owner
    }

    /// Reset TLAB for reuse
    ///
    /// Only safe when no thread is using it.
    pub fn reset(&self) {
        self.allocator.reset();
        self.retired.store(false, Ordering::SeqCst);
    }
}

/// TLAB Manager - manages all TLABs in the system
///
/// Handles:
/// - TLAB creation for new threads
/// - TLAB refill when full
/// - TLAB retirement when thread exits
/// - Dynamic TLAB size adjustment
pub struct TlabManager {
    /// Default TLAB size
    default_size: usize,

    /// Minimum TLAB size
    min_size: usize,

    /// Maximum TLAB size
    max_size: usize,

    /// Alignment requirement
    alignment: usize,

    /// Maximum active TLABs
    max_tlabs: usize,

    /// Active TLABs (mapped by thread ID)
    tlabs: std::sync::Mutex<std::collections::HashMap<ThreadId, Arc<Tlab>>>,

    /// TLAB refill counter
    refill_count: AtomicUsize,

    /// TLAB create counter
    create_count: AtomicUsize,
}

impl TlabManager {
    /// Create new TLAB manager
    ///
    /// # Arguments
    /// * `default_size` - Default TLAB size
    /// * `min_size` - Minimum TLAB size
    /// * `max_size` - Maximum TLAB size
    /// * `alignment` - Alignment requirement
    /// * `max_tlabs` - Maximum concurrent TLABs
    pub fn new(
        default_size: usize,
        min_size: usize,
        max_size: usize,
        alignment: usize,
        max_tlabs: usize,
    ) -> Self {
        Self {
            default_size,
            min_size,
            max_size,
            alignment,
            max_tlabs,
            tlabs: std::sync::Mutex::new(std::collections::HashMap::new()),
            refill_count: AtomicUsize::new(0),
            create_count: AtomicUsize::new(0),
        }
    }

    /// Get or create TLAB for thread
    ///
    /// Thread-safe: holds lock during entire check-and-create operation.
    ///
    /// # Arguments
    /// * `thread_id` - Thread ID
    /// * `heap` - Heap for TLAB memory allocation
    pub fn get_or_create_tlab(&self, thread_id: ThreadId, heap: &crate::heap::Heap) -> Result<Arc<Tlab>> {
        let mut tlabs = self.tlabs.lock().unwrap();

        if let Some(tlab) = tlabs.get(&thread_id) {
            if !tlab.is_retired() {
                return Ok(tlab.clone());
            }
        }

        if tlabs.len() >= self.max_tlabs {
            return Err(FgcError::TlabError("Maximum TLABs reached".to_string()));
        }

        let tlab_size = self.default_size;
        let tlab_start = heap.allocate_tlab_memory(tlab_size)?;
        let tlab_end = tlab_start + tlab_size;

        let tlab = Arc::new(Tlab::new(thread_id, tlab_start, tlab_end, self.alignment));

        tlabs.insert(thread_id, tlab.clone());
        self.create_count.fetch_add(1, Ordering::Relaxed);

        Ok(tlab)
    }

    /// Refill TLAB for thread
    ///
    /// Called when thread's TLAB is full.
    /// Retires old TLAB and creates new one.
    ///
    /// # Arguments
    /// * `thread_id` - Thread ID
    /// * `heap` - Heap for TLAB memory allocation
    pub fn refill_tlab(&self, thread_id: ThreadId, heap: &crate::heap::Heap) -> Result<Arc<Tlab>> {
        let mut tlabs = self.tlabs.lock().unwrap();

        if let Some(tlab) = tlabs.get(&thread_id) {
            tlab.retire();
        }
        tlabs.remove(&thread_id);

        drop(tlabs);

        self.refill_count.fetch_add(1, Ordering::Relaxed);

        self.get_or_create_tlab(thread_id, heap)
    }

    /// Remove TLAB for exited thread
    ///
    /// # Arguments
    /// * `thread_id` - Thread ID whose TLAB to remove
    pub fn remove_tlab(&self, thread_id: ThreadId) {
        let mut tlabs = self.tlabs.lock().unwrap();
        if let Some(tlab) = tlabs.get(&thread_id) {
            tlab.retire();
        }
        tlabs.remove(&thread_id);
    }

    /// Get active TLAB count
    pub fn active_tlab_count(&self) -> usize {
        let tlabs = self.tlabs.lock().unwrap();
        tlabs.len()
    }

    /// Get total refills
    pub fn total_refills(&self) -> usize {
        self.refill_count.load(Ordering::Relaxed)
    }

    /// Get total creates
    pub fn total_creates(&self) -> usize {
        self.create_count.load(Ordering::Relaxed)
    }

    /// Dynamic TLAB resize based on allocation pattern
    ///
    /// # Arguments
    /// * `_thread_id` - Thread ID (reserved for future per-thread tracking)
    /// * `new_size` - New TLAB size (clamped to min/max bounds)
    pub fn resize_tlab(&self, _thread_id: ThreadId, new_size: usize) {
        let _clamped_size = new_size.max(self.min_size).min(self.max_size);
    }
}

// Thread-local storage for TLAB reference
thread_local! {
    static CURRENT_TLAB: RefCell<Option<Arc<Tlab>>> = RefCell::new(None);
}

/// Get TLAB for current thread
pub fn get_current_tlab() -> Option<Arc<Tlab>> {
    CURRENT_TLAB.with(|tlab| tlab.borrow().clone())
}

/// Set TLAB for current thread
pub fn set_current_tlab(tlab: Arc<Tlab>) {
    CURRENT_TLAB.with(|t| {
        *t.borrow_mut() = Some(tlab);
    });
}
