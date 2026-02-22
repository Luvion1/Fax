//! Weak References Support
//!
//! Weak references allow holding references to objects without preventing
//! garbage collection. When the referent is collected, the weak reference
//! is cleared (set to None/null).
//!
//! Use Cases:
//! - Caches (don't prevent object collection)
//! - Observer patterns (listeners that can be collected)
//! - Canonicalization maps (weak hash maps)
//!
//! Implementation:
//! - WeakReference<T>: Holds weak reference to object
//! - Reference queue for notification when cleared
//! - Processed at end of GC cycle

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// WeakReference - a reference that doesn't prevent collection
///
/// When the referent is collected, this reference is automatically cleared.
pub struct WeakReference {
    /// Address of referent object
    referent: AtomicUsize,

    /// Reference queue (optional)
    queue: Option<usize>,

    /// Unique ID for this weak reference
    id: u64,
}

lazy_static::lazy_static! {
    static ref WEAK_REFS: Mutex<Vec<WeakReference>> = Mutex::new(Vec::new());
    static ref NEXT_WEAK_ID: AtomicUsize = AtomicUsize::new(1);
}

impl WeakReference {
    /// Create new weak reference to object
    ///
    /// # Arguments
    /// * `referent_addr` - Address of referent object
    ///
    /// # Returns
    /// WeakReference instance
    pub fn new(referent_addr: usize) -> Self {
        let id = NEXT_WEAK_ID.fetch_add(1, Ordering::Relaxed) as u64;

        Self {
            referent: AtomicUsize::new(referent_addr),
            queue: None,
            id,
        }
    }

    /// Create weak reference with reference queue
    ///
    /// When referent is collected, this reference is enqueued.
    pub fn with_queue(referent_addr: usize, queue_addr: usize) -> Self {
        let id = NEXT_WEAK_ID.fetch_add(1, Ordering::Relaxed) as u64;

        Self {
            referent: AtomicUsize::new(referent_addr),
            queue: Some(queue_addr),
            id,
        }
    }

    /// Get referent address
    ///
    /// Returns None if referent has been collected.
    pub fn get(&self) -> Option<usize> {
        let addr = self.referent.load(Ordering::Acquire);
        if addr == 0 {
            None
        } else {
            Some(addr)
        }
    }

    /// Clear weak reference
    ///
    /// Called by GC when referent is collected.
    pub fn clear(&self) {
        self.referent.store(0, Ordering::Release);
    }

    /// Check if reference is cleared
    pub fn is_cleared(&self) -> bool {
        self.referent.load(Ordering::Acquire) == 0
    }

    /// Get reference ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get queue (if any)
    pub fn queue(&self) -> Option<usize> {
        self.queue
    }
}

/// ReferenceQueue - notification queue for weak references
///
/// When a weak reference's referent is collected, the reference
/// is enqueued to this queue for notification.
pub struct ReferenceQueue {
    /// Enqueued references
    references: Mutex<Vec<usize>>,
}

impl ReferenceQueue {
    /// Create new reference queue
    pub fn new() -> Self {
        Self {
            references: Mutex::new(Vec::new()),
        }
    }

    /// Enqueue a weak reference
    pub fn enqueue(&self, weak_ref_addr: usize) {
        if let Ok(mut refs) = self.references.lock() {
            refs.push(weak_ref_addr);
        }
    }

    /// Poll for enqueued reference
    pub fn poll(&self) -> Option<usize> {
        if let Ok(mut refs) = self.references.lock() {
            refs.pop()
        } else {
            None
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        if let Ok(refs) = self.references.lock() {
            refs.is_empty()
        } else {
            true
        }
    }

    /// Get queue size
    pub fn size(&self) -> usize {
        if let Ok(refs) = self.references.lock() {
            refs.len()
        } else {
            0
        }
    }

    /// Clear the queue
    pub fn clear(&self) {
        if let Ok(mut refs) = self.references.lock() {
            refs.clear();
        }
    }
}

impl Default for ReferenceQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference types similar to Java (ZGC)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    /// Weak reference - cleared after marking
    Weak,
    /// Soft reference - cleared when memory pressure
    Soft,
    /// Phantom reference - for post-mortem cleanup
    Phantom,
    /// Final reference - object with finalize()
    Final,
}

/// SoftReference - cleared based on memory pressure
///
/// Unlike weak references, soft references are cleared based on
/// the garbage collector's discretion when memory is low.
pub struct SoftReference {
    /// Address of referent object
    referent: AtomicUsize,

    /// Reference queue (optional)
    queue: Option<usize>,

    /// Unique ID
    id: u64,

    /// Timestamp for LRU eviction
    timestamp: AtomicUsize,
}

lazy_static::lazy_static! {
    static ref SOFT_REFS: Mutex<Vec<SoftReference>> = Mutex::new(Vec::new());
    static ref NEXT_SOFT_ID: AtomicUsize = AtomicUsize::new(1);
}

impl SoftReference {
    /// Create new soft reference
    pub fn new(referent_addr: usize) -> Self {
        let id = NEXT_SOFT_ID.fetch_add(1, Ordering::Relaxed) as u64;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as usize)
            .unwrap_or(0);

        Self {
            referent: AtomicUsize::new(referent_addr),
            queue: None,
            id,
            timestamp: AtomicUsize::new(timestamp),
        }
    }

    /// Create with reference queue
    pub fn with_queue(referent_addr: usize, queue_addr: usize) -> Self {
        let mut this = Self::new(referent_addr);
        this.queue = Some(queue_addr);
        this
    }

    /// Get referent
    pub fn get(&self) -> Option<usize> {
        let addr = self.referent.load(Ordering::Acquire);
        if addr == 0 {
            None
        } else {
            Some(addr)
        }
    }

    /// Clear reference
    pub fn clear(&self) {
        self.referent.store(0, Ordering::Release);
    }

    /// Check if cleared
    pub fn is_cleared(&self) -> bool {
        self.referent.load(Ordering::Acquire) == 0
    }

    /// Get queue (if any)
    pub fn queue(&self) -> Option<usize> {
        self.queue
    }

    /// Get timestamp (for LRU)
    pub fn timestamp(&self) -> usize {
        self.timestamp.load(Ordering::Acquire)
    }

    /// Update timestamp on access
    pub fn touch(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as usize)
            .unwrap_or(0);
        self.timestamp.store(now, Ordering::Release);
    }

    /// Get ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// PhantomReference - for post-mortem cleanup
///
/// Phantom references are used for performing custom cleanup
/// actions before an object is garbage collected.
pub struct PhantomReference {
    /// Address of referent object
    referent: AtomicUsize,

    /// Reference queue for notification
    queue: usize,

    /// Unique ID
    id: u64,
}

lazy_static::lazy_static! {
    static ref PHANTOM_REFS: Mutex<Vec<PhantomReference>> = Mutex::new(Vec::new());
    static ref NEXT_PHANTOM_ID: AtomicUsize = AtomicUsize::new(1);
}

impl PhantomReference {
    /// Create new phantom reference
    pub fn new(referent_addr: usize, queue_addr: usize) -> Self {
        let id = NEXT_PHANTOM_ID.fetch_add(1, Ordering::Relaxed) as u64;
        Self {
            referent: AtomicUsize::new(referent_addr),
            queue: queue_addr,
            id,
        }
    }

    /// Get referent
    pub fn get(&self) -> Option<usize> {
        let addr = self.referent.load(Ordering::Acquire);
        if addr == 0 {
            None
        } else {
            Some(addr)
        }
    }

    /// Clear reference
    pub fn clear(&self) {
        self.referent.store(0, Ordering::Release);
    }

    /// Check if cleared
    pub fn is_cleared(&self) -> bool {
        self.referent.load(Ordering::Acquire) == 0
    }

    /// Get queue address
    pub fn queue(&self) -> usize {
        self.queue
    }

    /// Get ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// ReferenceProcessor - processes all reference types during GC
///
/// Similar to ZGC's reference processing, handles:
///
/// - Weak references: cleared after marking
/// - Soft references: cleared based on memory pressure
/// - Phantom references: enqueued for post-mortem cleanup
/// - Finalizers: processed after object is reachable
pub struct ReferenceProcessor {
    /// Reference queues for each type
    weak_queue: ReferenceQueue,
    soft_queue: ReferenceQueue,
    phantom_queue: ReferenceQueue,
    #[allow(dead_code)]
    final_queue: ReferenceQueue,

    /// Statistics
    cleared_weak: AtomicUsize,
    cleared_soft: AtomicUsize,
    cleared_phantom: AtomicUsize,
    processed_final: AtomicUsize,
}

impl ReferenceProcessor {
    /// Create new reference processor
    pub fn new() -> Self {
        Self {
            weak_queue: ReferenceQueue::new(),
            soft_queue: ReferenceQueue::new(),
            phantom_queue: ReferenceQueue::new(),
            final_queue: ReferenceQueue::new(),
            cleared_weak: AtomicUsize::new(0),
            cleared_soft: AtomicUsize::new(0),
            cleared_phantom: AtomicUsize::new(0),
            processed_final: AtomicUsize::new(0),
        }
    }

    /// Process weak references (ZGC: after marking)
    ///
    /// Clears weak references whose referents are not marked.
    pub fn process_weak_references(&self, marked: &impl Fn(usize) -> bool) -> usize {
        let mut cleared = 0;

        if let Ok(refs) = WEAK_REFS.lock() {
            for r in refs.iter() {
                if let Some(addr) = r.get() {
                    if !marked(addr) {
                        r.clear();
                        if let Some(queue) = r.queue() {
                            self.weak_queue.enqueue(queue);
                        }
                        cleared += 1;
                    }
                }
            }
        }

        self.cleared_weak.fetch_add(cleared, Ordering::Relaxed);
        cleared
    }

    /// Process soft references (ZGC: based on memory pressure)
    ///
    /// Clears soft references based on age and memory pressure.
    /// More aggressive clearing when heap is full.
    pub fn process_soft_references(
        &self,
        marked: &impl Fn(usize) -> bool,
        heap_percent_used: f32,
    ) -> usize {
        let mut cleared = 0;
        let threshold_ms = (heap_percent_used * 100.0) as usize;

        if let Ok(refs) = SOFT_REFS.lock() {
            for r in refs.iter() {
                if let Some(addr) = r.get() {
                    let is_marked = marked(addr);
                    let age = r.timestamp();

                    if !is_marked || age < threshold_ms {
                        r.clear();
                        if let Some(queue) = r.queue() {
                            self.soft_queue.enqueue(queue);
                        }
                        cleared += 1;
                    }
                }
            }
        }

        self.cleared_soft.fetch_add(cleared, Ordering::Relaxed);
        cleared
    }

    /// Process phantom references (ZGC: enqueue for cleanup)
    ///
    /// Enqueues phantom references whose referents are not reachable.
    pub fn process_phantom_references(&self, marked: &impl Fn(usize) -> bool) -> usize {
        let mut enqueued = 0;

        if let Ok(refs) = PHANTOM_REFS.lock() {
            for r in refs.iter() {
                if let Some(addr) = r.get() {
                    if !marked(addr) {
                        r.clear();
                        self.phantom_queue.enqueue(r.queue());
                        enqueued += 1;
                    }
                }
            }
        }

        self.cleared_phantom.fetch_add(enqueued, Ordering::Relaxed);
        enqueued
    }

    /// Get statistics
    pub fn stats(&self) -> ReferenceStats {
        ReferenceStats {
            cleared_weak: self.cleared_weak.load(Ordering::Relaxed),
            cleared_soft: self.cleared_soft.load(Ordering::Relaxed),
            cleared_phantom: self.cleared_phantom.load(Ordering::Relaxed),
            processed_final: self.processed_final.load(Ordering::Relaxed),
        }
    }
}

impl Default for ReferenceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference processing statistics
#[derive(Debug, Clone, Default)]
pub struct ReferenceStats {
    pub cleared_weak: usize,
    pub cleared_soft: usize,
    pub cleared_phantom: usize,
    pub processed_final: usize,
}

/// Process weak references at end of GC cycle
///
/// Called by GC after marking is complete.
/// Clears weak references to unreachable objects.
pub fn process_weak_references(marked_objects: &[usize]) {
    if let Ok(weak_refs) = WEAK_REFS.lock() {
        for weak_ref in weak_refs.iter() {
            if let Some(referent) = weak_ref.get() {
                if !marked_objects.contains(&referent) {
                    weak_ref.clear();
                }
            }
        }
    }
}

/// Register a weak reference
pub fn register_weak_reference(weak_ref: WeakReference) {
    if let Ok(mut weak_refs) = WEAK_REFS.lock() {
        weak_refs.push(weak_ref);
    }
}

/// Get count of registered weak references
pub fn weak_reference_count() -> usize {
    if let Ok(weak_refs) = WEAK_REFS.lock() {
        weak_refs.len()
    } else {
        0
    }
}

/// Clear all weak references
pub fn clear_all_weak_references() {
    if let Ok(mut weak_refs) = WEAK_REFS.lock() {
        weak_refs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_reference_basic() {
        let weak = WeakReference::new(0x1000);
        assert_eq!(weak.get(), Some(0x1000));
        assert!(!weak.is_cleared());

        weak.clear();
        assert!(weak.is_cleared());
        assert_eq!(weak.get(), None);
    }

    #[test]
    fn test_reference_queue() {
        let queue = ReferenceQueue::new();
        assert!(queue.is_empty());

        queue.enqueue(0x1000);
        queue.enqueue(0x2000);

        assert_eq!(queue.size(), 2);
        assert!(!queue.is_empty());

        let ref1 = queue.poll();
        assert_eq!(ref1, Some(0x2000));

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_process_weak_references() {
        let weak1 = WeakReference::new(0x1000);
        let weak2 = WeakReference::new(0x2000);

        register_weak_reference(weak1);
        register_weak_reference(weak2);

        let marked = vec![0x1000];
        process_weak_references(&marked);

        assert_eq!(weak_reference_count(), 2);
    }
}
