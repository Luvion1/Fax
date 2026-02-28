//! Barrier Statistics - Performance Monitoring
//!
//! This module provides statistics for load barrier performance monitoring.
//! Statistics are used for:
//! - Debugging and profiling
//! - Performance tuning
//! - GC health monitoring
//!
//! Metrics tracked:
//! - Total barrier invocations
//! - Fast path success rate
//! - Objects marked via barrier
//! - Pointers healed (relocated objects)

use std::sync::atomic::{AtomicU64, Ordering};

/// LoadBarrierStats - statistics for load barrier operations
///
/// These statistics track all aspects of load barrier performance:
/// - Invocation count
/// - Fast path vs slow path ratio
/// - Marking activity
/// - Pointer healing
///
/// # Thread Safety
/// All fields use atomic operations for thread-safe access.
/// Stats can be updated from multiple GC threads concurrently.
#[derive(Debug, Clone)]
pub struct LoadBarrierStats {
    /// Total barrier invocations
    pub total_invocations: u64,
    /// Objects already marked (fast path success)
    pub already_marked: u64,
    /// Objects needed marking (slow path)
    pub needed_marking: u64,
    /// Pointers healed (relocated objects)
    pub pointers_healed: u64,
    /// Null pointers (skipped)
    pub null_pointers: u64,
    /// Fast path success rate (percentage)
    pub fast_path_rate: f64,
}

impl LoadBarrierStats {
    /// Create new stats with default values
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge stats from another source (for aggregation)
    ///
    /// Used to combine stats from multiple threads
    /// or multiple GC cycles.
    ///
    /// # Arguments
    /// * `other` - Another stats instance to merge
    #[inline]
    pub fn merge(&mut self, other: &LoadBarrierStats) {
        self.total_invocations += other.total_invocations;
        self.already_marked += other.already_marked;
        self.needed_marking += other.needed_marking;
        self.pointers_healed += other.pointers_healed;
        self.null_pointers += other.null_pointers;
        self.recalculate_fast_path_rate();
    }

    /// Recalculate fast path rate
    #[inline]
    fn recalculate_fast_path_rate(&mut self) {
        if self.total_invocations == 0 {
            self.fast_path_rate = 100.0;
        } else {
            self.fast_path_rate =
                (self.already_marked as f64 / self.total_invocations as f64) * 100.0;
        }
    }

    /// Get slow path count
    #[inline]
    pub fn slow_path_count(&self) -> u64 {
        self.needed_marking
    }

    /// Get slow path rate (percentage)
    #[inline]
    pub fn slow_path_rate(&self) -> f64 {
        100.0 - self.fast_path_rate
    }

    /// Get total processed (excluding null pointers)
    #[inline]
    pub fn total_processed(&self) -> u64 {
        self.total_invocations - self.null_pointers
    }

    /// Print stats for debugging
    pub fn print(&self) {
        println!("LoadBarrier Statistics:");
        println!("  Total invocations: {}", self.total_invocations);
        println!(
            "  Already marked (fast path): {} ({:.2}%)",
            self.already_marked, self.fast_path_rate
        );
        println!(
            "  Needed marking (slow path): {} ({:.2}%)",
            self.needed_marking,
            self.slow_path_rate()
        );
        println!("  Pointers healed: {}", self.pointers_healed);
        println!("  Null pointers skipped: {}", self.null_pointers);
    }
}

impl Default for LoadBarrierStats {
    fn default() -> Self {
        Self {
            total_invocations: 0,
            already_marked: 0,
            needed_marking: 0,
            pointers_healed: 0,
            null_pointers: 0,
            fast_path_rate: 100.0, // Default to 100% when no invocations
        }
    }
}

/// AtomicLoadBarrierStats - thread-safe stats collector
///
/// Atomic version for collecting stats from multiple threads
/// without lock overhead.
pub struct AtomicLoadBarrierStats {
    /// Total barrier invocations
    total_invocations: AtomicU64,
    /// Objects already marked (fast path success)
    already_marked: AtomicU64,
    /// Objects needed marking (slow path)
    needed_marking: AtomicU64,
    /// Pointers healed (relocated objects)
    pointers_healed: AtomicU64,
    /// Null pointers (skipped)
    null_pointers: AtomicU64,
}

impl AtomicLoadBarrierStats {
    /// Create new atomic stats
    #[inline]
    pub fn new() -> Self {
        Self {
            total_invocations: AtomicU64::new(0),
            already_marked: AtomicU64::new(0),
            needed_marking: AtomicU64::new(0),
            pointers_healed: AtomicU64::new(0),
            null_pointers: AtomicU64::new(0),
        }
    }

    /// Record barrier invocation
    #[inline]
    pub fn record_invocation(&self) {
        self.total_invocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record fast path success (already marked)
    #[inline]
    pub fn record_fast_path(&self) {
        self.already_marked.fetch_add(1, Ordering::Relaxed);
    }

    /// Record slow path (needed marking)
    #[inline]
    pub fn record_slow_path(&self) {
        self.needed_marking.fetch_add(1, Ordering::Relaxed);
    }

    /// Record pointer healing
    #[inline]
    pub fn record_heal(&self) {
        self.pointers_healed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record null pointer skip
    #[inline]
    pub fn record_null(&self) {
        self.null_pointers.fetch_add(1, Ordering::Relaxed);
    }

    /// Record batch of invocations
    #[inline]
    pub fn record_batch(&self, count: u64) {
        self.total_invocations.fetch_add(count, Ordering::Relaxed);
    }

    /// Get current stats snapshot
    #[inline]
    pub fn snapshot(&self) -> LoadBarrierStats {
        let total = self.total_invocations.load(Ordering::Relaxed);
        let marked = self.already_marked.load(Ordering::Relaxed);

        let fast_path_rate = if total == 0 {
            100.0
        } else {
            (marked as f64 / total as f64) * 100.0
        };

        LoadBarrierStats {
            total_invocations: total,
            already_marked: marked,
            needed_marking: self.needed_marking.load(Ordering::Relaxed),
            pointers_healed: self.pointers_healed.load(Ordering::Relaxed),
            null_pointers: self.null_pointers.load(Ordering::Relaxed),
            fast_path_rate,
        }
    }

    /// Reset all stats to zero
    #[inline]
    pub fn reset(&self) {
        self.total_invocations.store(0, Ordering::Relaxed);
        self.already_marked.store(0, Ordering::Relaxed);
        self.needed_marking.store(0, Ordering::Relaxed);
        self.pointers_healed.store(0, Ordering::Relaxed);
        self.null_pointers.store(0, Ordering::Relaxed);
    }

    /// Get total invocations
    #[inline]
    pub fn total_invocations(&self) -> u64 {
        self.total_invocations.load(Ordering::Relaxed)
    }

    /// Get fast path count
    #[inline]
    pub fn fast_path_count(&self) -> u64 {
        self.already_marked.load(Ordering::Relaxed)
    }

    /// Get slow path count
    #[inline]
    pub fn slow_path_count(&self) -> u64 {
        self.needed_marking.load(Ordering::Relaxed)
    }

    /// Get pointers healed count
    #[inline]
    pub fn pointers_healed_count(&self) -> u64 {
        self.pointers_healed.load(Ordering::Relaxed)
    }

    /// Get null pointer count
    #[inline]
    pub fn null_pointer_count(&self) -> u64 {
        self.null_pointers.load(Ordering::Relaxed)
    }
}

impl Default for AtomicLoadBarrierStats {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Per-thread stats for reducing contention
///
/// Each GC thread has local stats that are merged at the end of GC cycle.
pub struct ThreadLocalStats {
    /// Local stats
    local: LoadBarrierStats,
}

impl ThreadLocalStats {
    /// Create new thread-local stats
    #[inline]
    pub fn new() -> Self {
        Self {
            local: LoadBarrierStats::new(),
        }
    }

    /// Record invocation
    #[inline]
    pub fn record_invocation(&mut self) {
        self.local.total_invocations += 1;
    }

    /// Record fast path
    #[inline]
    pub fn record_fast_path(&mut self) {
        self.local.already_marked += 1;
    }

    /// Record slow path
    #[inline]
    pub fn record_slow_path(&mut self) {
        self.local.needed_marking += 1;
    }

    /// Record heal
    #[inline]
    pub fn record_heal(&mut self) {
        self.local.pointers_healed += 1;
    }

    /// Record null
    #[inline]
    pub fn record_null(&mut self) {
        self.local.null_pointers += 1;
    }

    /// Get stats reference
    #[inline]
    pub fn stats(&self) -> &LoadBarrierStats {
        &self.local
    }

    /// Get mutable stats reference
    #[inline]
    pub fn stats_mut(&mut self) -> &mut LoadBarrierStats {
        &mut self.local
    }

    /// Take stats (reset local)
    #[inline]
    pub fn take(&mut self) -> LoadBarrierStats {
        let stats = self.local.clone();
        self.local = LoadBarrierStats::new();
        stats
    }
}

impl Default for ThreadLocalStats {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === LoadBarrierStats Tests ===

    #[test]
    fn test_stats_new() {
        let stats = LoadBarrierStats::new();
        assert_eq!(stats.total_invocations, 0);
        assert_eq!(stats.already_marked, 0);
        assert_eq!(stats.needed_marking, 0);
        assert_eq!(stats.pointers_healed, 0);
        assert_eq!(stats.null_pointers, 0);
        assert_eq!(stats.fast_path_rate, 100.0);
    }

    #[test]
    fn test_stats_merge() {
        let mut stats1 = LoadBarrierStats {
            total_invocations: 100,
            already_marked: 80,
            needed_marking: 20,
            pointers_healed: 5,
            null_pointers: 10,
            fast_path_rate: 80.0,
        };

        let stats2 = LoadBarrierStats {
            total_invocations: 50,
            already_marked: 40,
            needed_marking: 10,
            pointers_healed: 3,
            null_pointers: 5,
            fast_path_rate: 80.0,
        };

        stats1.merge(&stats2);

        assert_eq!(stats1.total_invocations, 150);
        assert_eq!(stats1.already_marked, 120);
        assert_eq!(stats1.needed_marking, 30);
        assert_eq!(stats1.pointers_healed, 8);
        assert_eq!(stats1.null_pointers, 15);
    }

    #[test]
    fn test_fast_path_rate_calculation() {
        let mut stats = LoadBarrierStats {
            total_invocations: 100,
            already_marked: 75,
            needed_marking: 25,
            pointers_healed: 0,
            null_pointers: 0,
            fast_path_rate: 0.0,
        };
        stats.recalculate_fast_path_rate();

        assert!((stats.fast_path_rate - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_fast_path_rate_zero_invocations() {
        let mut stats = LoadBarrierStats::new();
        stats.recalculate_fast_path_rate();
        assert_eq!(stats.fast_path_rate, 100.0);
    }

    #[test]
    fn test_slow_path_count() {
        let stats = LoadBarrierStats {
            total_invocations: 100,
            already_marked: 80,
            needed_marking: 20,
            pointers_healed: 0,
            null_pointers: 0,
            fast_path_rate: 80.0,
        };

        assert_eq!(stats.slow_path_count(), 20);
    }

    #[test]
    fn test_slow_path_rate() {
        let stats = LoadBarrierStats {
            total_invocations: 100,
            already_marked: 80,
            needed_marking: 20,
            pointers_healed: 0,
            null_pointers: 0,
            fast_path_rate: 80.0,
        };

        assert!((stats.slow_path_rate() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_total_processed() {
        let stats = LoadBarrierStats {
            total_invocations: 100,
            already_marked: 80,
            needed_marking: 20,
            pointers_healed: 0,
            null_pointers: 10,
            fast_path_rate: 80.0,
        };

        assert_eq!(stats.total_processed(), 90);
    }

    #[test]
    fn test_stats_clone() {
        let stats1 = LoadBarrierStats {
            total_invocations: 100,
            already_marked: 80,
            needed_marking: 20,
            pointers_healed: 5,
            null_pointers: 10,
            fast_path_rate: 80.0,
        };

        let stats2 = stats1.clone();

        assert_eq!(stats1.total_invocations, stats2.total_invocations);
        assert_eq!(stats1.already_marked, stats2.already_marked);
    }

    // === AtomicLoadBarrierStats Tests ===

    #[test]
    fn test_atomic_stats_new() {
        let stats = AtomicLoadBarrierStats::new();
        assert_eq!(stats.total_invocations(), 0);
        assert_eq!(stats.fast_path_count(), 0);
        assert_eq!(stats.slow_path_count(), 0);
        assert_eq!(stats.pointers_healed_count(), 0);
        assert_eq!(stats.null_pointer_count(), 0);
    }

    #[test]
    fn test_atomic_stats_record() {
        let stats = AtomicLoadBarrierStats::new();

        stats.record_invocation();
        stats.record_invocation();
        stats.record_invocation();

        stats.record_fast_path();
        stats.record_fast_path();

        stats.record_slow_path();

        stats.record_heal();

        stats.record_null();

        assert_eq!(stats.total_invocations(), 3);
        assert_eq!(stats.fast_path_count(), 2);
        assert_eq!(stats.slow_path_count(), 1);
        assert_eq!(stats.pointers_healed_count(), 1);
        assert_eq!(stats.null_pointer_count(), 1);
    }

    #[test]
    fn test_atomic_stats_snapshot() {
        let stats = AtomicLoadBarrierStats::new();

        for _ in 0..100 {
            stats.record_invocation();
        }
        for _ in 0..80 {
            stats.record_fast_path();
        }
        for _ in 0..20 {
            stats.record_slow_path();
        }

        let snapshot = stats.snapshot();

        assert_eq!(snapshot.total_invocations, 100);
        assert_eq!(snapshot.already_marked, 80);
        assert_eq!(snapshot.needed_marking, 20);
        assert!((snapshot.fast_path_rate - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_atomic_stats_reset() {
        let stats = AtomicLoadBarrierStats::new();

        stats.record_invocation();
        stats.record_fast_path();
        stats.record_slow_path();

        stats.reset();

        assert_eq!(stats.total_invocations(), 0);
        assert_eq!(stats.fast_path_count(), 0);
        assert_eq!(stats.slow_path_count(), 0);
    }

    #[test]
    fn test_atomic_stats_batch() {
        let stats = AtomicLoadBarrierStats::new();

        stats.record_batch(100);

        assert_eq!(stats.total_invocations(), 100);
    }

    #[test]
    fn test_atomic_stats_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let stats = Arc::new(AtomicLoadBarrierStats::new());
        let mut handles = vec![];

        // Spawn multiple threads to record stats concurrently
        for _ in 0..10 {
            let stats_clone = Arc::clone(&stats);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    stats_clone.record_invocation();
                    stats_clone.record_fast_path();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(stats.total_invocations(), 1000);
        assert_eq!(stats.fast_path_count(), 1000);
    }

    // === ThreadLocalStats Tests ===

    #[test]
    fn test_thread_local_stats_new() {
        let stats = ThreadLocalStats::new();
        assert_eq!(stats.stats().total_invocations, 0);
    }

    #[test]
    fn test_thread_local_stats_record() {
        let mut stats = ThreadLocalStats::new();

        stats.record_invocation();
        stats.record_invocation();
        stats.record_fast_path();
        stats.record_slow_path();
        stats.record_heal();
        stats.record_null();

        assert_eq!(stats.stats().total_invocations, 2);
        assert_eq!(stats.stats().already_marked, 1);
        assert_eq!(stats.stats().needed_marking, 1);
        assert_eq!(stats.stats().pointers_healed, 1);
        assert_eq!(stats.stats().null_pointers, 1);
    }

    #[test]
    fn test_thread_local_stats_take() {
        let mut stats = ThreadLocalStats::new();

        stats.record_invocation();
        stats.record_fast_path();

        let taken = stats.take();

        assert_eq!(taken.total_invocations, 1);
        assert_eq!(taken.already_marked, 1);
        assert_eq!(stats.stats().total_invocations, 0);
    }

    #[test]
    fn test_thread_local_stats_mut() {
        let mut stats = ThreadLocalStats::new();

        stats.record_invocation();

        let stats_mut = stats.stats_mut();
        stats_mut.already_marked += 10;

        assert_eq!(stats.stats().already_marked, 10);
    }

    // === Integration Tests ===

    #[test]
    fn test_stats_full_workflow() {
        let atomic_stats = AtomicLoadBarrierStats::new();

        // Simulate barrier operations
        for i in 0..1000 {
            atomic_stats.record_invocation();

            if i % 10 == 0 {
                atomic_stats.record_null();
            } else if i % 5 == 0 {
                atomic_stats.record_slow_path();
            } else {
                atomic_stats.record_fast_path();
            }

            if i % 50 == 0 {
                atomic_stats.record_heal();
            }
        }

        let snapshot = atomic_stats.snapshot();

        assert_eq!(snapshot.total_invocations, 1000);
        assert_eq!(snapshot.null_pointers, 100);
        assert!(snapshot.pointers_healed > 0);

        // Fast path + slow path + null should equal total
        let processed = snapshot.total_invocations - snapshot.null_pointers;
        assert_eq!(processed, snapshot.already_marked + snapshot.needed_marking);
    }

    #[test]
    fn test_stats_merge_multiple() {
        let mut global_stats = LoadBarrierStats::new();

        // Simulate merging from multiple threads
        for thread_id in 0..5 {
            let thread_stats = LoadBarrierStats {
                total_invocations: 100 * (thread_id + 1),
                already_marked: 80 * (thread_id + 1),
                needed_marking: 20 * (thread_id + 1),
                pointers_healed: 5 * (thread_id + 1),
                null_pointers: 10 * (thread_id + 1),
                fast_path_rate: 80.0,
            };
            global_stats.merge(&thread_stats);
        }

        // Sum of 100+200+300+400+500 = 1500
        assert_eq!(global_stats.total_invocations, 1500);
        assert_eq!(global_stats.already_marked, 1200);
        assert_eq!(global_stats.needed_marking, 300);
    }
}
