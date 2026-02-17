//! Stats Module - GC Performance Monitoring
//!
//! Module ini mengumpulkan statistik performa GC untuk:
//! - Performance tuning
//! - Production monitoring
//! - Debugging & profiling
//!
//! Metrics:
//! - Pause time (min, max, avg, percentiles)
//! - Memory usage (used, committed, max)
//! - GC frequency
//! - Allocation rates

pub mod timer;
pub mod histogram;
pub mod metrics;

pub use timer::GcTimer;
pub use histogram::Histogram;
pub use metrics::GcMetrics;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// GcStats - statistics collector untuk GC
///
/// Central repository untuk semua GC metrics.
pub struct GcStats {
    /// Total GC cycles
    total_cycles: AtomicU64,
    /// Minor GC count
    minor_cycles: AtomicU64,
    /// Major GC count
    major_cycles: AtomicU64,
    /// Pause time statistics
    pause_stats: Arc<histogram::Histogram>,
    /// Memory usage
    memory_usage: AtomicUsize,
    /// Start time
    start_time: std::time::Instant,
}

impl GcStats {
    /// Create new stats collector
    pub fn new() -> Self {
        Self {
            total_cycles: AtomicU64::new(0),
            minor_cycles: AtomicU64::new(0),
            major_cycles: AtomicU64::new(0),
            pause_stats: Arc::new(histogram::Histogram::new()),
            memory_usage: AtomicUsize::new(0),
            start_time: std::time::Instant::now(),
        }
    }

    /// Get pause_stats reference
    pub fn pause_stats(&self) -> Arc<histogram::Histogram> {
        self.pause_stats.clone()
    }

    /// Clone Arc for returning from stats()
    ///
    /// This method requires `self` to be an `&Arc<Self>` so it can clone
    /// the Arc rather than creating new atomic counters. This ensures
    /// all clones share the same underlying statistics.
    pub fn clone_arc(self: &Arc<Self>) -> Arc<GcStats> {
        self.clone()
    }

    /// Record GC collection
    pub fn record_collection(
        &self,
        cycle: u64,
        generation: crate::gc::GcGeneration,
        duration: std::time::Duration,
    ) {
        self.total_cycles.fetch_add(1, Ordering::Relaxed);

        match generation {
            crate::gc::GcGeneration::Young => {
                self.minor_cycles.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.major_cycles.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Record pause time
        self.pause_stats.record(duration.as_nanos() as u64);
    }

    /// Record memory usage
    pub fn record_memory_usage(&self, bytes: usize) {
        self.memory_usage.store(bytes, Ordering::Relaxed);
    }

    /// Get summary statistics
    pub fn summary(&self) -> GcSummary {
        GcSummary {
            total_cycles: self.total_cycles.load(Ordering::Relaxed),
            minor_cycles: self.minor_cycles.load(Ordering::Relaxed),
            major_cycles: self.major_cycles.load(Ordering::Relaxed),
            avg_pause_ms: self.pause_stats.mean() as f64 / 1_000_000.0,
            max_pause_ms: self.pause_stats.max() as f64 / 1_000_000.0,
            heap_used_mb: self.memory_usage.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0),
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }

    /// Get pause time histogram
    pub fn pause_histogram(&self) -> Arc<histogram::Histogram> {
        self.pause_stats.clone()
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.total_cycles.store(0, Ordering::Relaxed);
        self.minor_cycles.store(0, Ordering::Relaxed);
        self.major_cycles.store(0, Ordering::Relaxed);
        self.pause_stats.clear();
    }
}

impl Default for GcStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics
#[derive(Debug, Default)]
pub struct GcSummary {
    /// Total GC cycles
    pub total_cycles: u64,
    /// Minor GC count
    pub minor_cycles: u64,
    /// Major GC count
    pub major_cycles: u64,
    /// Average pause time (ms)
    pub avg_pause_ms: f64,
    /// Max pause time (ms)
    pub max_pause_ms: f64,
    /// Heap used (MB)
    pub heap_used_mb: f64,
    /// Uptime (seconds)
    pub uptime_secs: u64,
}
