//! GC Cycle Statistics - ZGC-like Metrics
//!
//! Comprehensive GC cycle statistics similar to ZGC's gc metrics.
//! Tracks all phases: pause marks, concurrent mark, pause reloc, concurrent reloc.

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// GC Cycle Statistics - comprehensive metrics for each GC cycle
#[derive(Debug, Clone, Default)]
pub struct GcCycleStats {
    /// Cycle ID
    pub cycle_id: u64,

    /// Phase timings (nanoseconds)
    pub pause_mark_start_ns: u64,
    pub concurrent_mark_ns: u64,
    pub pause_mark_end_ns: u64,
    pub pause_relocate_start_ns: u64,
    pub concurrent_relocate_ns: u64,
    pub pause_relocate_end_ns: u64,

    /// Memory statistics
    pub heap_used_before: usize,
    pub heap_used_after: usize,
    pub heap_committed: usize,
    pub memory_reclaimed: usize,

    /// Object statistics
    pub objects_scanned: u64,
    pub objects_marked: u64,
    pub objects_relocated: u64,
    pub objects_garbage: u64,

    /// Reference processing
    pub weak_refs_cleared: u64,
    pub soft_refs_cleared: u64,
    pub phantom_refs_cleared: u64,

    /// Thread statistics
    pub gc_threads_used: usize,
    pub worker_time_total_ns: u64,

    /// Success flag
    pub completed: bool,
    pub failed: bool,
    pub failure_reason: Option<String>,
}

impl GcCycleStats {
    pub fn new(cycle_id: u64) -> Self {
        Self {
            cycle_id,
            ..Default::default()
        }
    }

    /// Get total pause time (sum of all STW phases)
    pub fn total_pause_time_ns(&self) -> u64 {
        self.pause_mark_start_ns
            + self.pause_mark_end_ns
            + self.pause_relocate_start_ns
            + self.pause_relocate_end_ns
    }

    /// Get total concurrent time
    pub fn total_concurrent_time_ns(&self) -> u64 {
        self.concurrent_mark_ns + self.concurrent_relocate_ns
    }

    /// Get total cycle time
    pub fn total_cycle_time_ns(&self) -> u64 {
        self.total_pause_time_ns() + self.total_concurrent_time_ns()
    }

    /// Get pause time percentage
    pub fn pause_time_percent(&self) -> f64 {
        let total = self.total_cycle_time_ns() as f64;
        if total == 0.0 {
            0.0
        } else {
            (self.total_pause_time_ns() as f64 / total) * 100.0
        }
    }
}

/// GC Statistics Collector - aggregates statistics across all cycles
pub struct GcStatsCollector {
    /// Current cycle stats
    current_cycle: RwLock<Option<GcCycleStats>>,

    /// Historical cycles (last N cycles)
    history: RwLock<Vec<GcCycleStats>>,
    max_history: usize,

    /// Aggregated statistics
    total_cycles: AtomicU64,
    total_pause_time_ns: AtomicU64,
    total_concurrent_time_ns: AtomicU64,
    total_objects_marked: AtomicU64,
    total_objects_relocated: AtomicU64,
    total_memory_reclaimed: AtomicUsize,
    total_garbage_collected: AtomicU64,

    /// Reference processing totals
    total_weak_cleared: AtomicU64,
    total_soft_cleared: AtomicU64,
    total_phantom_cleared: AtomicU64,

    /// Peak memory usage
    peak_heap_used: AtomicUsize,
    peak_gc_pause_ns: AtomicU64,
}

impl GcStatsCollector {
    pub fn new(max_history: usize) -> Self {
        Self {
            current_cycle: RwLock::new(None),
            history: RwLock::new(Vec::with_capacity(max_history)),
            max_history,
            total_cycles: AtomicU64::new(0),
            total_pause_time_ns: AtomicU64::new(0),
            total_concurrent_time_ns: AtomicU64::new(0),
            total_objects_marked: AtomicU64::new(0),
            total_objects_relocated: AtomicU64::new(0),
            total_memory_reclaimed: AtomicUsize::new(0),
            total_garbage_collected: AtomicU64::new(0),
            total_weak_cleared: AtomicU64::new(0),
            total_soft_cleared: AtomicU64::new(0),
            total_phantom_cleared: AtomicU64::new(0),
            peak_heap_used: AtomicUsize::new(0),
            peak_gc_pause_ns: AtomicU64::new(0),
        }
    }

    /// Start a new GC cycle
    pub fn start_cycle(&self, cycle_id: u64) {
        let mut current = self.current_cycle.write();
        *current = Some(GcCycleStats::new(cycle_id));
    }

    /// End current GC cycle
    pub fn end_cycle(&self, stats: GcCycleStats) {
        let _cycle_id = stats.cycle_id;

        // Update aggregated stats
        self.total_cycles.fetch_add(1, Ordering::Relaxed);
        self.total_pause_time_ns
            .fetch_add(stats.total_pause_time_ns(), Ordering::Relaxed);
        self.total_concurrent_time_ns
            .fetch_add(stats.total_concurrent_time_ns(), Ordering::Relaxed);
        self.total_objects_marked
            .fetch_add(stats.objects_marked, Ordering::Relaxed);
        self.total_objects_relocated
            .fetch_add(stats.objects_relocated, Ordering::Relaxed);
        self.total_memory_reclaimed
            .fetch_add(stats.memory_reclaimed, Ordering::Relaxed);
        self.total_garbage_collected
            .fetch_add(stats.objects_garbage, Ordering::Relaxed);
        self.total_weak_cleared
            .fetch_add(stats.weak_refs_cleared, Ordering::Relaxed);
        self.total_soft_cleared
            .fetch_add(stats.soft_refs_cleared, Ordering::Relaxed);
        self.total_phantom_cleared
            .fetch_add(stats.phantom_refs_cleared, Ordering::Relaxed);

        // Update peaks
        let heap_used = stats.heap_used_after;
        let current_peak = self.peak_heap_used.load(Ordering::Relaxed);
        if heap_used > current_peak {
            self.peak_heap_used.store(heap_used, Ordering::Relaxed);
        }

        let pause_time = stats.total_pause_time_ns();
        let current_peak_pause = self.peak_gc_pause_ns.load(Ordering::Relaxed);
        if pause_time > current_peak_pause {
            self.peak_gc_pause_ns.store(pause_time, Ordering::Relaxed);
        }

        // Add to history
        let mut history = self.history.write();
        if history.len() >= self.max_history {
            history.remove(0);
        }
        history.push(stats);

        // Clear current cycle
        let mut current = self.current_cycle.write();
        *current = None;
    }

    /// Get current cycle stats (if active)
    pub fn current_cycle(&self) -> Option<GcCycleStats> {
        self.current_cycle.read().clone()
    }

    /// Get aggregated statistics
    pub fn get_aggregated(&self) -> AggregatedStats {
        let cycles = self.total_cycles.load(Ordering::Relaxed);
        AggregatedStats {
            total_cycles: cycles,
            total_pause_time_ns: self.total_pause_time_ns.load(Ordering::Relaxed),
            total_concurrent_time_ns: self.total_concurrent_time_ns.load(Ordering::Relaxed),
            total_objects_marked: self.total_objects_marked.load(Ordering::Relaxed),
            total_objects_relocated: self.total_objects_relocated.load(Ordering::Relaxed),
            total_memory_reclaimed: self.total_memory_reclaimed.load(Ordering::Relaxed),
            total_garbage_collected: self.total_garbage_collected.load(Ordering::Relaxed),
            avg_pause_time_ns: if cycles > 0 {
                self.total_pause_time_ns.load(Ordering::Relaxed) / cycles
            } else {
                0
            },
            avg_concurrent_time_ns: if cycles > 0 {
                self.total_concurrent_time_ns.load(Ordering::Relaxed) / cycles
            } else {
                0
            },
            peak_heap_used: self.peak_heap_used.load(Ordering::Relaxed),
            peak_gc_pause_ns: self.peak_gc_pause_ns.load(Ordering::Relaxed),
        }
    }

    /// Get history of recent cycles
    pub fn get_history(&self) -> Vec<GcCycleStats> {
        self.history.read().clone()
    }

    /// Reset all statistics
    pub fn reset(&self) {
        *self.current_cycle.write() = None;
        self.history.write().clear();
        self.total_cycles.store(0, Ordering::Relaxed);
        self.total_pause_time_ns.store(0, Ordering::Relaxed);
        self.total_concurrent_time_ns.store(0, Ordering::Relaxed);
        self.total_objects_marked.store(0, Ordering::Relaxed);
        self.total_objects_relocated.store(0, Ordering::Relaxed);
        self.total_memory_reclaimed.store(0, Ordering::Relaxed);
        self.total_garbage_collected.store(0, Ordering::Relaxed);
        self.total_weak_cleared.store(0, Ordering::Relaxed);
        self.total_soft_cleared.store(0, Ordering::Relaxed);
        self.total_phantom_cleared.store(0, Ordering::Relaxed);
        self.peak_heap_used.store(0, Ordering::Relaxed);
        self.peak_gc_pause_ns.store(0, Ordering::Relaxed);
    }
}

/// Aggregated statistics across all cycles
#[derive(Debug, Clone, Default)]
pub struct AggregatedStats {
    pub total_cycles: u64,
    pub total_pause_time_ns: u64,
    pub total_concurrent_time_ns: u64,
    pub total_objects_marked: u64,
    pub total_objects_relocated: u64,
    pub total_memory_reclaimed: usize,
    pub total_garbage_collected: u64,
    pub avg_pause_time_ns: u64,
    pub avg_concurrent_time_ns: u64,
    pub peak_heap_used: usize,
    pub peak_gc_pause_ns: u64,
}

impl AggregatedStats {
    /// Get average pause time in milliseconds
    pub fn avg_pause_time_ms(&self) -> f64 {
        self.avg_pause_time_ns as f64 / 1_000_000.0
    }

    /// Get peak pause time in milliseconds
    pub fn peak_pause_time_ms(&self) -> f64 {
        self.peak_gc_pause_ns as f64 / 1_000_000.0
    }

    /// Get peak heap usage in MB
    pub fn peak_heap_mb(&self) -> f64 {
        self.peak_heap_used as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_stats() {
        let mut stats = GcCycleStats::new(1);
        stats.pause_mark_start_ns = 100_000;
        stats.pause_mark_end_ns = 200_000;
        stats.concurrent_mark_ns = 1_000_000;

        assert_eq!(stats.total_pause_time_ns(), 300_000);
        assert_eq!(stats.total_concurrent_time_ns(), 1_000_000);
        assert_eq!(stats.total_cycle_time_ns(), 1_300_000);
    }

    #[test]
    fn test_stats_collector() {
        let collector = GcStatsCollector::new(10);

        collector.start_cycle(1);
        {
            let mut current = collector.current_cycle.write();
            if let Some(ref mut stats) = *current {
                stats.objects_marked = 1000;
                stats.objects_garbage = 500;
                stats.memory_reclaimed = 1_000_000;
                stats.pause_mark_start_ns = 100_000;
            }
        }

        let stats = collector.current_cycle().unwrap();
        collector.end_cycle(stats);

        let agg = collector.get_aggregated();
        assert_eq!(agg.total_cycles, 1);
        assert_eq!(agg.total_objects_marked, 1000);
    }
}
