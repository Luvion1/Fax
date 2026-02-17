//! Configuration Module - GC Tuning Parameters
//!
//! Manages all configuration parameters for FGC.
//! Proper configuration balances throughput, latency, and memory footprint.

/// Main configuration for Fax Garbage Collector
///
/// Stores all parameters affecting GC behavior.
/// Most parameters have sensible defaults.
///
/// # Examples
///
/// ```rust
/// use fgc::GcConfig;
///
/// // Use default configuration
/// let config = GcConfig::default();
///
/// // Custom configuration for low-latency
/// let config = GcConfig {
///     target_pause_time_ms: 5,
///     gc_threads: Some(8),
///     generational: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Minimum heap size in bytes
    ///
    /// Heap will not shrink below this size.
    /// Default: 1/4 of max_heap_size or 16MB (whichever is larger)
    pub min_heap_size: usize,

    /// Maximum heap size in bytes
    ///
    /// Hard limit for heap growth. GC throws OOM if exceeded.
    /// Default: 1/2 of physical memory or 4GB (whichever is smaller)
    pub max_heap_size: usize,

    /// Initial heap size at startup
    ///
    /// Heap starts at this size and grows as needed.
    /// Default: min_heap_size
    pub initial_heap_size: usize,

    /// Soft maximum heap size
    ///
    /// GC tries to keep heap below this limit but can exceed if needed.
    /// Default: max_heap_size
    pub soft_max_heap_size: usize,

    /// Target pause time in milliseconds
    ///
    /// FGC tries to keep all GC pauses below this value.
    /// GC adjusts frequency and parallelism to meet target.
    ///
    /// Recommended values:
    /// - Low-latency apps: 1-10ms
    /// - General purpose: 10-50ms
    /// - Batch processing: 50-200ms
    ///
    /// Default: 10ms
    pub target_pause_time_ms: u64,

    /// Number of GC threads for concurrent operations
    ///
    /// Used for concurrent marking, relocation, and cleanup.
    /// If None, auto-detects based on CPU cores: gc_threads = min(4, num_cpus / 2)
    ///
    /// Default: Auto-detect
    pub gc_threads: Option<usize>,

    /// Enable generational mode
    ///
    /// Separates heap into young and old generations.
    /// Young objects collected frequently with minor GC.
    /// Old objects collected less frequently with major GC.
    ///
    /// Default: true
    pub generational: bool,

    /// Ratio of heap for young generation (0.0 - 1.0)
    ///
    /// Example: 0.3 = 30% heap for young generation
    /// Recommended: 0.2 - 0.5
    ///
    /// Default: 0.3 (30%)
    pub young_ratio: f32,

    /// Tenuring threshold - survives before promotion
    ///
    /// Objects surviving N minor GCs are promoted to old generation.
    /// Recommended: 6-15
    ///
    /// Default: 9
    pub tenure_threshold: u8,

    /// Small region size in bytes
    ///
    /// Used for objects < small_threshold.
    /// Default: 2MB
    pub small_region_size: usize,

    /// Medium region size in bytes
    ///
    /// Used for objects between small_threshold and large_threshold.
    /// Default: 32MB
    pub medium_region_size: usize,

    /// Threshold for small objects (bytes)
    ///
    /// Objects <= this size allocated in small regions.
    /// Default: 256 bytes
    pub small_threshold: usize,

    /// Threshold for large objects (bytes)
    ///
    /// Objects > this size get dedicated large region.
    /// Default: 4KB
    pub large_threshold: usize,

    /// Enable Thread-Local Allocation Buffers
    ///
    /// TLAB allows lock-free allocation by giving each thread a private buffer.
    /// Disable only for single-threaded apps or debugging.
    ///
    /// Default: true
    pub tlab_enabled: bool,

    /// Default TLAB size in bytes
    ///
    /// Each thread gets a TLAB of this size.
    /// TLAB refilled when full.
    ///
    /// Default: 256KB
    pub tlab_size: usize,

    /// Minimum TLAB size
    ///
    /// TLAB will not shrink below this size.
    /// Default: 16KB
    pub tlab_min_size: usize,

    /// Maximum TLAB size
    ///
    /// TLAB will not grow above this size.
    /// Default: 2MB
    pub tlab_max_size: usize,

    /// Enable dynamic TLAB resizing
    ///
    /// FGC adjusts TLAB size based on allocation rate.
    /// Default: true
    pub tlab_resize: bool,

    /// Enable NUMA-aware allocation
    ///
    /// On multi-socket systems, allocates memory on the local NUMA node.
    /// Default: true
    pub numa_aware: bool,

    /// Enable large/huge pages
    ///
    /// Large pages (2MB or 1GB) reduce TLB misses.
    /// Requires kernel support and proper permissions.
    /// Falls back to regular pages if unavailable.
    ///
    /// Default: false
    pub use_large_pages: bool,

    /// Large page size to use
    ///
    /// Options: 2MB or 1GB
    /// 2MB easier to obtain, 1GB better for very large heaps.
    ///
    /// Default: 2MB
    pub large_page_size: usize,

    /// Enable verbose GC logging
    ///
    /// Logs GC cycle start/end, pause times, heap statistics.
    /// Default: false
    pub verbose: bool,

    /// Enable GC statistics collection
    ///
    /// Collects stats for pause time histogram, allocation rates.
    /// Default: true
    pub stats_enabled: bool,

    /// Periodic GC interval (milliseconds)
    ///
    /// If > 0, triggers GC every interval even if heap not full.
    /// If 0, GC triggers on-demand only.
    ///
    /// Default: 0
    pub gc_interval_ms: u64,
}

impl Default for GcConfig {
    /// Default configuration for FGC
    ///
    /// Balanced for general-purpose server applications.
    fn default() -> Self {
        let num_cpus = num_cpus::get();
        let total_memory = get_total_memory();
        let max_heap = calculate_max_heap(total_memory);
        let min_heap = max_heap / 4;

        GcConfig {
            // Heap size
            min_heap_size: min_heap.max(16 * MB),
            max_heap_size: max_heap,
            initial_heap_size: min_heap,
            soft_max_heap_size: max_heap,

            // Pause time
            target_pause_time_ms: 10,

            // Threading
            gc_threads: Some((num_cpus / 2).max(1).min(4)),

            // Generational
            generational: true,
            young_ratio: 0.3,
            tenure_threshold: 9,

            // Regions
            small_region_size: 2 * MB,
            medium_region_size: 32 * MB,
            small_threshold: 256,
            large_threshold: 4 * KB,

            // TLAB
            tlab_enabled: true,
            tlab_size: 256 * KB,
            tlab_min_size: 16 * KB,
            tlab_max_size: 2 * MB,
            tlab_resize: true,

            // NUMA
            numa_aware: true,

            // Large pages
            use_large_pages: false,
            large_page_size: 2 * MB,

            // Debug
            verbose: false,
            stats_enabled: true,
            gc_interval_ms: 0,
        }
    }
}

impl GcConfig {
    /// Validate configuration
    ///
    /// Checks if all values are in valid ranges.
    /// Returns error if configuration is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let config = GcConfig {
    ///     max_heap_size: 0,  // Invalid!
    ///     ..Default::default()
    /// };
    ///
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Heap size validation
        if self.max_heap_size == 0 {
            return Err(ConfigError::InvalidHeapSize(
                "max_heap_size must be > 0".to_string(),
            ));
        }

        if self.min_heap_size > self.max_heap_size {
            return Err(ConfigError::InvalidHeapSize(
                "min_heap_size cannot exceed max_heap_size".to_string(),
            ));
        }

        if self.initial_heap_size < self.min_heap_size
            || self.initial_heap_size > self.max_heap_size
        {
            return Err(ConfigError::InvalidHeapSize(
                "initial_heap_size must be between min and max heap size".to_string(),
            ));
        }

        // Young ratio validation
        if self.young_ratio < 0.05 || self.young_ratio > 0.9 {
            return Err(ConfigError::InvalidYoungRatio(
                "young_ratio must be between 0.05 and 0.9".to_string(),
            ));
        }

        // TLAB validation
        if self.tlab_size < self.tlab_min_size {
            return Err(ConfigError::InvalidTlabSize(
                "tlab_size must be >= tlab_min_size".to_string(),
            ));
        }

        if self.tlab_size > self.tlab_max_size {
            return Err(ConfigError::InvalidTlabSize(
                "tlab_size must be <= tlab_max_size".to_string(),
            ));
        }

        // Region size validation
        if self.small_region_size < MB {
            return Err(ConfigError::InvalidRegionSize(
                "small_region_size must be at least 1MB".to_string(),
            ));
        }

        if self.medium_region_size < self.small_region_size {
            return Err(ConfigError::InvalidRegionSize(
                "medium_region_size must be >= small_region_size".to_string(),
            ));
        }

        // Threshold validation
        if self.small_threshold >= self.large_threshold {
            return Err(ConfigError::InvalidThreshold(
                "small_threshold must be < large_threshold".to_string(),
            ));
        }

        // GC threads validation
        if let Some(threads) = self.gc_threads {
            if threads == 0 {
                return Err(ConfigError::InvalidGcThreads(
                    "gc_threads must be > 0".to_string(),
                ));
            }
        }

        // Pause time validation
        if self.target_pause_time_ms == 0 {
            return Err(ConfigError::InvalidPauseTime(
                "target_pause_time_ms must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Build configuration from environment variables
    ///
    /// Overrides defaults with environment variables:
    /// - FGC_MAX_HEAP
    /// - FGC_MIN_HEAP
    /// - FGC_PAUSE_TIME_MS
    /// - FGC_GC_THREADS
    /// - FGC_VERBOSE
    ///
    /// # Examples
    ///
    /// ```bash
    /// export FGC_MAX_HEAP=4294967296  # 4GB
    /// export FGC_PAUSE_TIME_MS=5
    /// export FGC_VERBOSE=1
    /// ```
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("FGC_MAX_HEAP") {
            if let Ok(size) = val.parse::<usize>() {
                config.max_heap_size = size;
            }
        }

        if let Ok(val) = std::env::var("FGC_MIN_HEAP") {
            if let Ok(size) = val.parse::<usize>() {
                config.min_heap_size = size;
            }
        }

        if let Ok(val) = std::env::var("FGC_PAUSE_TIME_MS") {
            if let Ok(ms) = val.parse::<u64>() {
                config.target_pause_time_ms = ms;
            }
        }

        if let Ok(val) = std::env::var("FGC_GC_THREADS") {
            if let Ok(threads) = val.parse::<usize>() {
                config.gc_threads = Some(threads);
            }
        }

        if let Ok(val) = std::env::var("FGC_VERBOSE") {
            config.verbose = val == "1" || val.eq_ignore_ascii_case("true");
        }

        config
    }

    /// Get estimated GC overhead percentage
    ///
    /// Estimates CPU time percentage used by GC based on configuration.
    /// Returns value between 0-100.
    pub fn estimated_overhead(&self) -> f32 {
        let base_overhead = 100.0 / (self.target_pause_time_ms as f32).max(1.0);
        let thread_factor = 1.0 / (self.gc_threads.unwrap_or(1) as f32).sqrt();
        let gen_factor = if self.generational { 0.7 } else { 1.0 };

        (base_overhead * thread_factor * gen_factor * 10.0).min(50.0)
    }
}

/// Error types for configuration
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid heap size: {0}")]
    InvalidHeapSize(String),

    #[error("Invalid young ratio: {0}")]
    InvalidYoungRatio(String),

    #[error("Invalid TLAB size: {0}")]
    InvalidTlabSize(String),

    #[error("Invalid region size: {0}")]
    InvalidRegionSize(String),

    #[error("Invalid threshold: {0}")]
    InvalidThreshold(String),

    #[error("Invalid GC threads: {0}")]
    InvalidGcThreads(String),

    #[error("Invalid pause time: {0}")]
    InvalidPauseTime(String),
}

// ============================================================================
// CONSTANTS & HELPERS
// ============================================================================

const KB: usize = 1024;
const MB: usize = 1024 * 1024;
const GB: usize = 1024 * 1024 * 1024;

/// Get total physical memory in bytes
fn get_total_memory() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return kb * KB;
                        }
                    }
                }
            }
        }
    }

    8 * GB
}

/// Calculate max heap size based on available memory
fn calculate_max_heap(total_memory: usize) -> usize {
    let ratio = if total_memory < 4 * GB {
        0.5
    } else if total_memory < 16 * GB {
        0.4
    } else {
        0.3
    };

    let calculated = (total_memory as f32 * ratio) as usize;
    calculated.min(32 * GB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GcConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.generational);
        assert_eq!(config.target_pause_time_ms, 10);
    }

    #[test]
    fn test_invalid_heap_size() {
        let config = GcConfig {
            max_heap_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_young_ratio() {
        let config = GcConfig {
            young_ratio: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
