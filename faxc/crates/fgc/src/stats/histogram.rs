//! Histogram - Pause Time Histogram
//!
//! Histogram untuk tracking pause time distribution.
//! Berguna untuk menghitung percentiles (P50, P95, P99, P999).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Histogram - pause time histogram
///
/// Track distribution of pause times untuk statistical analysis.
pub struct Histogram {
    /// Bucket counts (logarithmic buckets)
    buckets: std::sync::Mutex<BTreeMap<u64, u64>>,

    /// Total count
    count: AtomicU64,

    /// Sum of all values
    sum: AtomicU64,

    /// Min value
    min: AtomicU64,

    /// Max value
    max: AtomicU64,
}

impl Histogram {
    /// Create new histogram
    pub fn new() -> Self {
        Self {
            buckets: std::sync::Mutex::new(BTreeMap::new()),
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    /// Record value
    pub fn record(&self, value: u64) {
        let bucket = self.get_bucket(value);

        let mut buckets = self.buckets.lock().unwrap();
        *buckets.entry(bucket).or_insert(0) += 1;

        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);

        // Update min/max
        let mut current_min = self.min.load(Ordering::Relaxed);
        while value < current_min {
            if self
                .min
                .compare_exchange_weak(current_min, value, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            current_min = self.min.load(Ordering::Relaxed);
        }

        let mut current_max = self.max.load(Ordering::Relaxed);
        while value > current_max {
            if self
                .max
                .compare_exchange_weak(current_max, value, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            current_max = self.max.load(Ordering::Relaxed);
        }
    }

    /// Get bucket untuk value (logarithmic)
    fn get_bucket(&self, value: u64) -> u64 {
        if value == 0 {
            0
        } else {
            // Logarithmic buckets: 1, 2, 4, 8, 16, ...
            64 - value.leading_zeros() as u64
        }
    }

    /// Get percentile
    pub fn percentile(&self, p: f64) -> u64 {
        let buckets = self.buckets.lock().unwrap();
        let total = self.count.load(Ordering::Relaxed);

        if total == 0 {
            return 0;
        }

        let target = (total as f64 * p) as u64;
        let mut cumulative = 0;

        for (&bucket, &count) in buckets.iter() {
            cumulative += count;
            if cumulative >= target {
                return bucket;
            }
        }

        buckets.last_key_value().map(|(&k, _)| k).unwrap_or(0)
    }

    /// Get P50 (median)
    pub fn p50(&self) -> u64 {
        self.percentile(0.50)
    }

    /// Get P95
    pub fn p95(&self) -> u64 {
        self.percentile(0.95)
    }

    /// Get P99
    pub fn p99(&self) -> u64 {
        self.percentile(0.99)
    }

    /// Get P999
    pub fn p999(&self) -> u64 {
        self.percentile(0.999)
    }

    /// Get mean
    pub fn mean(&self) -> u64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return 0;
        }
        self.sum.load(Ordering::Relaxed) / count
    }

    /// Get min
    pub fn min(&self) -> u64 {
        let min = self.min.load(Ordering::Relaxed);
        if min == u64::MAX {
            0
        } else {
            min
        }
    }

    /// Get max
    pub fn max(&self) -> u64 {
        self.max.load(Ordering::Relaxed)
    }

    /// Get count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Clear histogram
    pub fn clear(&self) {
        self.buckets.lock().unwrap().clear();
        self.count.store(0, Ordering::Relaxed);
        self.sum.store(0, Ordering::Relaxed);
        self.min.store(u64::MAX, Ordering::Relaxed);
        self.max.store(0, Ordering::Relaxed);
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}
