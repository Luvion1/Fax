//! Generational Allocator - Young/Old Generation Management
//!
//! Manages generational allocation (young/old generation).
//! Based on the observation that:
//! - Most objects die young
//! - Objects that survive tend to live long

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

/// AgeTracker - tracks object age for promotion
///
/// Tracks how many GC cycles an object has survived.
pub struct AgeTracker {
    /// Current age of object
    age: AtomicUsize,
    /// Promotion threshold
    threshold: usize,
}

impl AgeTracker {
    /// Create new age tracker
    pub fn new(threshold: usize) -> Self {
        Self {
            age: AtomicUsize::new(0),
            threshold,
        }
    }

    /// Increment age
    pub fn increment(&self) -> usize {
        self.age.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Get current age
    pub fn age(&self) -> usize {
        self.age.load(Ordering::Relaxed)
    }

    /// Check if should be promoted
    pub fn should_promote(&self) -> bool {
        self.age.load(Ordering::Relaxed) >= self.threshold
    }

    /// Reset age
    pub fn reset(&self) {
        self.age.store(0, Ordering::Relaxed);
    }
}

/// GenerationalStats - statistics for generational allocator
#[derive(Debug, Clone, Default)]
pub struct GenerationalStats {
    /// Young generation size
    pub young_size: usize,
    /// Old generation size
    pub old_size: usize,
    /// Objects promoted
    pub promoted_count: usize,
    /// Young gen collections
    pub young_collections: usize,
    /// Old gen collections
    pub old_collections: usize,
}

/// GenerationalAllocator - manages young and old generation
///
/// Objects are allocated in young generation first.
/// Objects that survive GC are promoted to old generation.
pub struct GenerationalAllocator {
    /// Young generation capacity
    young_capacity: usize,
    /// Old generation capacity
    old_capacity: usize,
    /// Current young generation usage
    young_used: AtomicUsize,
    /// Current old generation usage
    old_used: AtomicUsize,
    /// Promotion threshold (GC cycles before promotion)
    promotion_threshold: usize,
    /// Statistics
    stats: std::sync::Mutex<GenerationalStats>,
}

impl GenerationalAllocator {
    /// Create new generational allocator
    ///
    /// # Arguments
    /// * `young_capacity` - Young generation capacity
    /// * `old_capacity` - Old generation capacity
    /// * `promotion_threshold` - GC cycles before promotion
    pub fn new(young_capacity: usize, old_capacity: usize, promotion_threshold: usize) -> Self {
        Self {
            young_capacity,
            old_capacity,
            young_used: AtomicUsize::new(0),
            old_used: AtomicUsize::new(0),
            promotion_threshold,
            stats: std::sync::Mutex::new(GenerationalStats {
                young_size: young_capacity,
                old_size: old_capacity,
                ..Default::default()
            }),
        }
    }

    /// Allocate in young generation
    pub fn allocate_young(&self, size: usize) -> Result<usize> {
        let current = self.young_used.load(Ordering::Relaxed);
        let new_used = current.saturating_add(size);

        if new_used > self.young_capacity {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: self.young_capacity.saturating_sub(current),
            });
        }

        self.young_used.store(new_used, Ordering::Relaxed);

        // Return dummy address for now
        Ok(0x1000 + current)
    }

    /// Allocate in old generation
    pub fn allocate_old(&self, size: usize) -> Result<usize> {
        let current = self.old_used.load(Ordering::Relaxed);
        let new_used = current.saturating_add(size);

        if new_used > self.old_capacity {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: self.old_capacity.saturating_sub(current),
            });
        }

        self.old_used.store(new_used, Ordering::Relaxed);

        // Return dummy address for now
        Ok(0x10000 + current)
    }

    /// Promote object from young to old
    pub fn promote(&self, size: usize) -> Result<()> {
        self.allocate_old(size)?;

        match self.stats.lock() {
            Ok(mut stats) => {
                stats.promoted_count += 1;
            },
            Err(e) => {
                log::error!("GenerationalAllocator stats lock poisoned: {}", e);
            },
        }

        Ok(())
    }

    /// Record young generation collection
    pub fn record_young_collection(&self) {
        match self.stats.lock() {
            Ok(mut stats) => {
                stats.young_collections += 1;
            },
            Err(e) => {
                log::error!("GenerationalAllocator stats lock poisoned: {}", e);
            },
        }
    }

    /// Record old generation collection
    pub fn record_old_collection(&self) {
        match self.stats.lock() {
            Ok(mut stats) => {
                stats.old_collections += 1;
            },
            Err(e) => {
                log::error!("GenerationalAllocator stats lock poisoned: {}", e);
            },
        }
    }

    /// Get promotion threshold
    pub fn promotion_threshold(&self) -> usize {
        self.promotion_threshold
    }

    /// Get statistics
    pub fn stats(&self) -> GenerationalStats {
        match self.stats.lock() {
            Ok(stats) => stats.clone(),
            Err(e) => {
                log::error!("GenerationalAllocator stats lock poisoned: {}", e);
                GenerationalStats::default()
            },
        }
    }

    /// Get young generation usage
    pub fn young_used(&self) -> usize {
        self.young_used.load(Ordering::Relaxed)
    }

    /// Get old generation usage
    pub fn old_used(&self) -> usize {
        self.old_used.load(Ordering::Relaxed)
    }

    /// Reset young generation
    pub fn reset_young(&self) {
        self.young_used.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_tracker_creation() {
        let tracker = AgeTracker::new(3);
        assert_eq!(tracker.age(), 0);
        assert!(!tracker.should_promote());
    }

    #[test]
    fn test_age_tracker_increment() {
        let tracker = AgeTracker::new(3);

        assert_eq!(tracker.increment(), 1);
        assert_eq!(tracker.increment(), 2);
        assert_eq!(tracker.increment(), 3);

        assert!(tracker.should_promote());
    }

    #[test]
    fn test_age_tracker_reset() {
        let tracker = AgeTracker::new(3);

        tracker.increment();
        tracker.increment();
        tracker.reset();

        assert_eq!(tracker.age(), 0);
        assert!(!tracker.should_promote());
    }

    #[test]
    fn test_generational_allocator_creation() {
        let alloc = GenerationalAllocator::new(1024, 4096, 3);

        let stats = alloc.stats();
        assert_eq!(stats.young_size, 1024);
        assert_eq!(stats.old_size, 4096);
    }

    #[test]
    fn test_generational_allocator_young_allocation() {
        let alloc = GenerationalAllocator::new(1024, 4096, 3);

        let addr = alloc.allocate_young(100).unwrap();
        assert!(addr > 0);
        assert_eq!(alloc.young_used(), 100);
    }

    #[test]
    fn test_generational_allocator_old_allocation() {
        let alloc = GenerationalAllocator::new(1024, 4096, 3);

        let addr = alloc.allocate_old(200).unwrap();
        assert!(addr > 0);
        assert_eq!(alloc.old_used(), 200);
    }

    #[test]
    fn test_generational_allocator_young_oom() {
        let alloc = GenerationalAllocator::new(100, 4096, 3);

        let _ = alloc.allocate_young(80).unwrap();

        let result = alloc.allocate_young(50);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_generational_allocator_promotion() {
        let alloc = GenerationalAllocator::new(1024, 4096, 3);

        alloc.promote(100).unwrap();

        let stats = alloc.stats();
        assert_eq!(stats.promoted_count, 1);
    }

    #[test]
    fn test_generational_allocator_collection_recording() {
        let alloc = GenerationalAllocator::new(1024, 4096, 3);

        alloc.record_young_collection();
        alloc.record_young_collection();
        alloc.record_old_collection();

        let stats = alloc.stats();
        assert_eq!(stats.young_collections, 2);
        assert_eq!(stats.old_collections, 1);
    }

    #[test]
    fn test_generational_allocator_reset_young() {
        let alloc = GenerationalAllocator::new(1024, 4096, 3);

        let _ = alloc.allocate_young(100).unwrap();
        assert_eq!(alloc.young_used(), 100);

        alloc.reset_young();
        assert_eq!(alloc.young_used(), 0);
    }

    #[test]
    fn test_generational_allocator_concurrent_allocation() {
        use std::sync::Arc;
        use std::thread;

        let alloc = Arc::new(GenerationalAllocator::new(1024 * 1024, 4 * 1024 * 1024, 3));
        let mut handles = Vec::new();

        for _ in 0..4 {
            let alloc = Arc::clone(&alloc);
            let handle = thread::spawn(move || {
                let mut count = 0;
                for _ in 0..100 {
                    if alloc.allocate_young(64).is_ok() {
                        count += 1;
                    }
                }
                count
            });
            handles.push(handle);
        }

        let total: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

        assert!(total > 0);
        assert_eq!(alloc.young_used(), total * 64);
    }
}
