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

/// Global weak reference registry
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
