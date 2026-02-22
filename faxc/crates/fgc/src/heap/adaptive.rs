//! Adaptive Heap Sizing - ZGC-like Dynamic Heap Management
//!
//! Automatically adjusts heap size based on allocation patterns and GC behavior.
//! Similar to ZGC's adaptive heap sizing.

use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Adaptive Heap Controller - ZGC-like dynamic heap sizing
pub struct AdaptiveHeapController {
    /// Current heap minimum
    min_heap: AtomicUsize,
    /// Current heap maximum
    max_heap: AtomicUsize,
    /// Soft maximum (target)
    soft_max_heap: AtomicUsize,

    /// Allocation rate tracking
    alloc_rate: RwLock<AllocationRateTracker>,

    /// Last GC stats
    last_gc: RwLock<GcStatsForSizing>,

    /// Configuration
    config: AdaptiveConfig,

    /// State
    state: RwLock<AdaptiveState>,
}

#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    /// Enable adaptive heap sizing
    pub enabled: bool,

    /// Growth factor when heap usage > threshold
    pub growth_factor: f64,

    /// Shrink factor when heap usage < threshold  
    pub shrink_factor: f64,

    /// Heap usage threshold for growth (0.0 - 1.0)
    pub growth_threshold: f64,

    /// Heap usage threshold for shrinking (0.0 - 1.0)
    pub shrink_threshold: f64,

    /// Minimum heap size
    pub min_size: usize,

    /// Maximum heap size
    pub max_size: usize,

    /// Number of samples for rate calculation
    pub sample_count: usize,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            growth_factor: 1.2,
            shrink_factor: 0.8,
            growth_threshold: 0.75,
            shrink_threshold: 0.4,
            min_size: 16 * 1024 * 1024,       // 16MB
            max_size: 4 * 1024 * 1024 * 1024, // 4GB
            sample_count: 10,
        }
    }
}

#[derive(Debug, Default)]
struct AllocationRateTracker {
    samples: Vec<AllocationSample>,
    current_index: usize,
}

#[derive(Debug, Clone)]
struct AllocationSample {
    bytes_allocated: usize,
    time_ms: u64,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct GcStatsForSizing {
    gc_count: u64,
    total_pause_ms: u64,
    heap_used_before: usize,
    heap_used_after: usize,
    reclaimed_bytes: usize,
    collection_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum AdaptiveState {
    Warmup,
    Stable,
    Growing,
    Shrinking,
}

impl AdaptiveHeapController {
    pub fn new(initial_size: usize, config: AdaptiveConfig) -> Self {
        let min_size = config.min_size;
        let max_size = config.max_size;

        Self {
            min_heap: AtomicUsize::new(min_size),
            max_heap: AtomicUsize::new(max_size),
            soft_max_heap: AtomicUsize::new(initial_size),
            alloc_rate: RwLock::new(AllocationRateTracker::new(config.sample_count)),
            last_gc: RwLock::new(GcStatsForSizing::default()),
            config,
            state: RwLock::new(AdaptiveState::Warmup),
        }
    }

    pub fn record_allocation(&self, bytes: usize, time_ms: u64) {
        let mut tracker = self.alloc_rate.write();
        tracker.add_sample(bytes, time_ms);
    }

    pub fn record_gc(&self, used_before: usize, used_after: usize, reclaimed: usize, reason: &str) {
        let mut stats = self.last_gc.write();
        stats.gc_count += 1;
        stats.heap_used_before = used_before;
        stats.heap_used_after = used_after;
        stats.reclaimed_bytes = reclaimed;
        stats.collection_reason = reason.to_string();
    }

    pub fn calculate_new_heap_size(&self) -> usize {
        let config = &self.config;
        if !config.enabled {
            return self.soft_max_heap.load(Ordering::Relaxed);
        }

        let current_soft_max = self.soft_max_heap.load(Ordering::Relaxed);
        let stats = self.last_gc.read();
        let alloc_rate = self.alloc_rate.read();

        if stats.gc_count < 3 {
            *self.state.write() = AdaptiveState::Warmup;
            return current_soft_max;
        }

        let usage_ratio = if current_soft_max > 0 {
            stats.heap_used_after as f64 / current_soft_max as f64
        } else {
            0.0
        };

        let mut new_size = current_soft_max;

        if usage_ratio > config.growth_threshold {
            // Calculate needed size based on allocation rate
            let rate_per_sec = alloc_rate.average_rate_per_sec();
            if rate_per_sec > 0 {
                // Target: keep 3 seconds of allocation in heap
                let target_size = (rate_per_sec * 3) as usize;
                new_size = (target_size as f64 * config.growth_factor) as usize;
            } else {
                new_size = (current_soft_max as f64 * config.growth_factor) as usize;
            }
            *self.state.write() = AdaptiveState::Growing;
        } else if usage_ratio < config.shrink_threshold && current_soft_max > config.min_size {
            new_size = (current_soft_max as f64 * config.shrink_factor) as usize;
            *self.state.write() = AdaptiveState::Shrinking;
        } else {
            *self.state.write() = AdaptiveState::Stable;
        }

        // Clamp to bounds
        new_size = new_size.clamp(config.min_size, config.max_size);

        // Update soft max
        self.soft_max_heap.store(new_size, Ordering::Relaxed);

        new_size
    }

    pub fn get_current_size(&self) -> usize {
        self.soft_max_heap.load(Ordering::Relaxed)
    }

    pub fn get_state(&self) -> AdaptiveState {
        self.state.read().clone()
    }

    pub fn get_config(&self) -> AdaptiveConfig {
        self.config.clone()
    }

    pub fn set_min_size(&self, size: usize) {
        self.min_heap.store(size, Ordering::Relaxed);
    }

    pub fn set_max_size(&self, size: usize) {
        self.max_heap.store(size, Ordering::Relaxed);
    }

    pub fn get_heap_stats(&self) -> HeapSizeStats {
        let state = self.get_state();
        let alloc_rate = self.alloc_rate.read();
        let stats = self.last_gc.read();

        HeapSizeStats {
            current_size: self.soft_max_heap.load(Ordering::Relaxed),
            min_size: self.min_heap.load(Ordering::Relaxed),
            max_size: self.max_heap.load(Ordering::Relaxed),
            state: format!("{:?}", state),
            gc_count: stats.gc_count,
            heap_used: stats.heap_used_after,
            avg_alloc_rate: alloc_rate.average_rate_per_sec(),
        }
    }
}

impl AllocationRateTracker {
    fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            current_index: 0,
        }
    }

    fn add_sample(&mut self, bytes: usize, time_ms: u64) {
        if self.samples.len() < self.samples.capacity() {
            self.samples.push(AllocationSample {
                bytes_allocated: bytes,
                time_ms,
            });
        } else {
            self.samples[self.current_index] = AllocationSample {
                bytes_allocated: bytes,
                time_ms,
            };
            self.current_index = (self.current_index + 1) % self.samples.capacity();
        }
    }

    fn average_rate_per_sec(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }

        let total_bytes: usize = self.samples.iter().map(|s| s.bytes_allocated).sum();
        let total_time_ms: u64 = self.samples.iter().map(|s| s.time_ms).sum();

        if total_time_ms == 0 {
            return 0;
        }

        // Convert to bytes per second
        (total_bytes as f64 / (total_time_ms as f64 / 1000.0)) as u64
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        self.samples.clear();
        self.current_index = 0;
    }
}

#[derive(Debug, Clone)]
pub struct HeapSizeStats {
    pub current_size: usize,
    pub min_size: usize,
    pub max_size: usize,
    pub state: String,
    pub gc_count: u64,
    pub heap_used: usize,
    pub avg_alloc_rate: u64,
}

impl HeapSizeStats {
    pub fn usage_percent(&self) -> f64 {
        if self.current_size == 0 {
            return 0.0;
        }
        (self.heap_used as f64 / self.current_size as f64) * 100.0
    }

    pub fn current_size_mb(&self) -> f64 {
        self.current_size as f64 / (1024.0 * 1024.0)
    }

    pub fn heap_used_mb(&self) -> f64 {
        self.heap_used as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_heap_controller() {
        let config = AdaptiveConfig::default();
        let controller = AdaptiveHeapController::new(64 * 1024 * 1024, config);

        // Record some allocations
        controller.record_allocation(1024, 10);
        controller.record_allocation(2048, 20);

        // Record a GC
        controller.record_gc(
            50 * 1024 * 1024,
            30 * 1024 * 1024,
            20 * 1024 * 1024,
            "heap_full",
        );

        // Get stats
        let stats = controller.get_heap_stats();
        assert_eq!(stats.gc_count, 1);
        assert_eq!(stats.heap_used, 30 * 1024 * 1024);
    }

    #[test]
    fn test_allocation_rate() {
        let mut tracker = AllocationRateTracker::new(5);

        tracker.add_sample(1000, 10);
        tracker.add_sample(2000, 20);

        let rate = tracker.average_rate_per_sec();
        assert!(rate > 0);
    }
}
