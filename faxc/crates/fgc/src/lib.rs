//! FGC - Fax Garbage Collector
//!
//! A low-latency garbage collector inspired by ZGC (Z Garbage Collector).
//! Designed for applications requiring sub-10ms pause times regardless of heap size.
//!
//! ## Key Features
//!
//! - **Concurrent**: Marking and compaction run concurrently with the application
//! - **Region-based**: Heap divided into regions for parallel collection
//! - **Compacting**: Eliminates fragmentation through object relocation
//! - **Generational**: Young/old generation separation for efficiency
//! - **Low-Latency**: Target pause times under 10ms
//!
//! ## Architecture
//!
//! FGC implements several innovations from ZGC:
//!
//! - **Colored Pointers**: GC metadata stored in unused pointer bits (44-47)
//! - **Load Barriers**: Intercept pointer reads for concurrent marking and pointer healing
//! - **Region-Based Heap**: Small (2MB), medium (32MB), and large (variable) regions
//! - **Multi-Mapping Virtual Memory**: Physical memory mapped to multiple virtual addresses

// Core GC
pub mod gc;
pub mod config;
pub mod error;

// Allocator subsystem
pub mod allocator;

// Barrier subsystem (colored pointers, load barriers)
pub mod barrier;

// Heap management (regions, pages, virtual memory, NUMA)
pub mod heap;

// Marker subsystem (concurrent marking)
pub mod marker;

// Relocate subsystem (object relocation, compaction)
pub mod relocate;

// Statistics and monitoring
pub mod stats;

// Utilities
pub mod util;

// Runtime integration
pub mod runtime;

// Object model
pub mod object;

// Memory operations
pub mod memory;

// Re-export main types for convenience
pub use config::GcConfig;
pub use error::{FgcError, Result};
pub use gc::{GarbageCollector, GcGeneration, GcReason, GcState};
pub use runtime::Runtime;

/// FGC version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize FGC with default configuration
///
/// # Examples
///
/// ```rust
/// let runtime = fgc::init()?;
/// runtime.start()?;
///
/// // Use GC...
///
/// runtime.stop()?;
/// # Ok::<(), fgc::FgcError>(())
/// ```
pub fn init() -> Result<Runtime> {
    let config = GcConfig::default();
    Runtime::new(config)
}

/// Initialize FGC with custom configuration
///
/// # Examples
///
/// ```rust
/// let config = GcConfig {
///     max_heap_size: 4 * 1024 * 1024 * 1024, // 4GB
///     target_pause_time_ms: 5,
///     generational: true,
///     ..Default::default()
/// };
///
/// let runtime = fgc::init_with_config(config)?;
/// # Ok::<(), fgc::FgcError>(())
/// ```
pub fn init_with_config(config: GcConfig) -> Result<Runtime> {
    Runtime::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_default() {
        let result = init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation() {
        let config = GcConfig::default();
        assert!(config.validate().is_ok());
    }
}
