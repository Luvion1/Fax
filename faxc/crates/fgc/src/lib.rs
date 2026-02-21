//! # FGC - High-Performance Concurrent Garbage Collector
//!
//! FGC is a ZGC-inspired concurrent garbage collector for Rust applications, designed
//! for latency-sensitive workloads requiring sub-10ms pause times regardless of heap size.
//!
//! ## Overview
//!
//! FGC implements several advanced garbage collection techniques:
//!
//! - **Concurrent Mark-Compact**: Marking and compaction run concurrently with application threads
//! - **Colored Pointers**: GC metadata stored in unused pointer bits (44-47) eliminates object header overhead
//! - **Load Barriers**: Intercept pointer reads for concurrent marking and pointer healing
//! - **Region-Based Heap**: Heap divided into regions for parallel collection and partial compaction
//! - **Generational Collection**: Young/Old generation separation for improved efficiency
//! - **Thread-Local Allocation Buffers (TLAB)**: Lock-free allocation for hot paths
//!
//! ## Quick Start
//!
//! ```rust
//! use fgc::{GarbageCollector, FgcConfig, GcGeneration};
//!
//! fn main() -> Result<(), fgc::FgcError> {
//!     // Create GC with default configuration
//!     let config = FgcConfig::default();
//!     let gc = GarbageCollector::new(config)?;
//!     
//!     // Allocate objects
//!     let addr = gc.allocate(64)?;
//!     
//!     // Register as root to prevent collection
//!     gc.register_root(addr)?;
//!     
//!     // Use the allocated memory
//!     unsafe {
//!         *(addr as *mut u64) = 0x12345678;
//!     }
//!     
//!     // Trigger GC when needed
//!     gc.collect();
//!     
//!     // Unregister root when done
//!     gc.unregister_root(addr)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Mutator Threads                       │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
//! │  │  TLAB    │  │  TLAB    │  │  TLAB    │              │
//! │  └────┬─────┘  └────┬─────┘  └────┬─────┘              │
//! │       │             │             │                     │
//! │       └─────────────┴─────────────┘                     │
//! │                           │                              │
//! │                    Write Barrier                        │
//! │                           │                              │
//! └───────────────────────────┼──────────────────────────────┘
//!                             │
//! ┌───────────────────────────┼──────────────────────────────┐
//! │                    GC Threads                            │
//! │                           ▼                              │
//! │  ┌───────────────────────────────────────────┐          │
//! │  │              Mark Phase                    │          │
//! │  │  - Concurrent marking from roots           │          │
//! │  │  - Stack scanning at safepoint             │          │
//! │  └───────────────────────────────────────────┘          │
//! │                           │                              │
//! │                           ▼                              │
//! │  ┌───────────────────────────────────────────┐          │
//! │  │           Relocation Phase                 │          │
//! │  │  - Select live objects                     │          │
//! │  │  - Copy to new location                    │          │
//! │  │  - Update references                       │          │
//! │  └───────────────────────────────────────────┘          │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ### GC Cycle Phases
//!
//! 1. **Pause Mark Start** (STW < 1ms): Flip mark bits, scan roots
//! 2. **Concurrent Mark** (No STW): Mark objects via load barriers
//! 3. **Pause Mark End** (STW < 1ms): Finalize marking
//! 4. **Prepare Relocation** (No STW): Setup forwarding tables
//! 5. **Concurrent Relocate** (No STW): Copy objects, pointer healing
//! 6. **Cleanup** (No STW): Free old regions
//!
//! ### Colored Pointers
//!
//! FGC stores GC metadata in pointer bits 44-47:
//!
//! ```text
//! 64-bit Pointer Layout:
//! ┌────────────┬─────┬─────┬─────┬─────┬──────────────────────┐
//! │  Unused    │ Fin │ Rem │ M1  │ M0  │     Address          │
//! │  63-48     │ 47  │ 46  │ 45  │ 44  │       43-0           │
//! └────────────┴─────┴─────┴─────┴─────┴──────────────────────┘
//!
//! Color Bits:
//! - M0 (Marked0): Object marked in even GC cycle
//! - M1 (Marked1): Object marked in odd GC cycle
//! - Rem (Remapped): Pointer already remapped
//! - Fin (Finalizable): Object needs finalization
//! ```
//!
//! ## Platform Support
//!
//! | Platform | Status | Notes |
//! |----------|--------|-------|
//! | Linux x86_64 | ✅ Full | Multi-mapping with mmap |
//! | Linux aarch64 | ✅ Full | Multi-mapping with mmap |
//! | macOS x86_64 | ⚠️ Partial | shm_open based multi-mapping |
//! | macOS Apple Silicon | ⚠️ Partial | shm_open based multi-mapping |
//! | Windows x86_64 | ✅ Full | Multi-mapping with CreateFileMapping/MapViewOfFile |
//!
//! ### Multi-Mapping Implementation by Platform
//!
//! **Unix (Linux/macOS):** Uses `mmap` with `MAP_SHARED` to map the same file descriptor
//! at three different virtual addresses. All views share the same physical pages.
//!
//! **Windows:** Uses `CreateFileMappingW` to create a file mapping object backed by the
//! system paging file, then `MapViewOfFile` to map three views of the same physical pages
//! at different virtual addresses.
//!
//! ## Safety
//!
//! FGC uses `unsafe` internally but provides safe abstractions.
//! Users must follow these rules:
//!
//! 1. **Always register roots before GC**: Unregistered pointers may be collected
//! 2. **Don't modify registered roots without write barrier**: GC may miss references
//! 3. **Don't use addresses after GC without updating**: Objects may have moved
//! 4. **Respect alignment requirements**: All allocations are 8-byte aligned minimum
//!
//! ### Thread Safety
//!
//! - `GarbageCollector` is `Send + Sync` and safe for concurrent access
//! - `Runtime` is `Send + Sync` and safe for concurrent access
//! - Allocation is thread-safe via TLAB or atomic bump pointer
//! - Root registration requires external synchronization if called from multiple threads
//!
//! ## Performance
//!
//! | Operation | Expected Time |
//! |-----------|---------------|
//! | TLAB Allocation (hit) | < 10ns |
//! | TLAB Allocation (miss) | ~100ns |
//! | Load Barrier (fast path) | < 5ns |
//! | Load Barrier (slow path) | ~50ns |
//! | GC Pause (mark start) | < 1ms |
//! | GC Pause (mark end) | < 1ms |
//!
//! ## Example: Custom Configuration
//!
//! ```rust
//! use fgc::{GarbageCollector, FgcConfig};
//!
//! fn main() -> Result<(), fgc::FgcError> {
//!     let config = FgcConfig {
//!         max_heap_size: 256 * 1024 * 1024,    // 256MB
//!         min_heap_size: 64 * 1024 * 1024,     // 64MB
//!         target_pause_time_ms: 5,              // 5ms target
//!         gc_threads: Some(4),                  // 4 GC threads
//!         generational: true,                   // Enable generational
//!         young_ratio: 0.3,                     // 30% young gen
//!         tlab_enabled: true,                   // Enable TLAB
//!         verbose: true,                        // Enable logging
//!         ..Default::default()
//!     };
//!     
//!     let gc = GarbageCollector::new(config)?;
//!     
//!     // Allocate and use objects
//!     let obj = gc.allocate(128)?;
//!     gc.register_root(obj)?;
//!     
//!     unsafe {
//!         *(obj as *mut u64) = 42;
//!     }
//!     
//!     // GC will preserve obj (it's rooted)
//!     gc.collect();
//!     
//!     gc.unregister_root(obj)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Example: Using Runtime
//!
//! ```rust
//! use fgc::{Runtime, GcConfig, GcGeneration};
//!
//! fn main() -> Result<(), fgc::FgcError> {
//!     let config = GcConfig::default();
//!     let runtime = Runtime::new(config)?;
//!     runtime.start()?;
//!     
//!     // Allocate through runtime
//!     let addr = runtime.allocate(256)?;
//!     
//!     // Request GC
//!     runtime.request_gc(GcGeneration::Young);
//!     
//!     // Check safepoint in long loops
//!     for i in 0..1000000 {
//!         if i % 1000 == 0 {
//!             runtime.check_safepoint();
//!         }
//!     }
//!     
//!     runtime.stop()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`allocator`]: Memory allocation strategies (bump pointer, TLAB, large objects)
//! - [`barrier`]: Colored pointers and load barriers for concurrent operations
//! - [`config`]: GC configuration parameters and validation
//! - [`error`]: Error types for all FGC operations
//! - [`gc`]: Core GC cycle management and orchestration
//! - [`heap`]: Region-based heap management and virtual memory
//! - [`marker`]: Concurrent marking system and root scanning
//! - [`memory`]: Low-level memory operations
//! - [`object`]: Object model and reference maps
//! - [`relocate`]: Object relocation and compaction
//! - [`runtime`]: Runtime integration, safepoints, and finalizers
//! - [`stats`]: Performance statistics and monitoring
//! - [`util`]: Utility functions and helpers
//!
//! ## Limitations
//!
//! - **Stack Scanning**: Conservative stack scanning may keep garbage alive longer than necessary
//! - **Platform Support**: Full multi-mapping only available on Linux
//! - **Precise GC**: Requires explicit root registration
//!
//! ## Getting Help
//!
//! - [API Documentation](https://docs.rs/fgc)
//! - [GitHub Issues](https://github.com/your-org/fgc/issues)
//! - [Troubleshooting Guide](../TROUBLESHOOTING.md)

// Core GC modules
pub mod gc;
pub mod config;
pub mod error;

// Memory management subsystems
pub mod allocator;
pub mod heap;
pub mod memory;
pub mod object;

// GC algorithm components
pub mod barrier;
pub mod marker;
pub mod relocate;

// Runtime and monitoring
pub mod runtime;
pub mod stats;

// Utilities
pub mod util;

// Re-export main types for convenience
pub use config::GcConfig;

/// Type alias for backward compatibility
pub type FgcConfig = GcConfig;
pub use error::{FgcError, Result};
pub use gc::{GarbageCollector, GcGeneration, GcReason, GcState};
pub use runtime::Runtime;

/// FGC version string from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize FGC with default configuration
///
/// Creates a new `Runtime` instance with default `GcConfig`.
/// The runtime must be started before use with `runtime.start()`.
///
/// # Returns
///
/// - `Ok(Runtime)` - Runtime instance ready to start
/// - `Err(FgcError)` - Initialization failed (e.g., invalid config, memory allocation failure)
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
/// Creates a new `Runtime` instance with the specified configuration.
/// Allows fine-tuning GC behavior for specific workloads.
///
/// # Arguments
///
/// * `config` - Custom GC configuration parameters
///
/// # Returns
///
/// - `Ok(Runtime)` - Runtime instance ready to start
/// - `Err(FgcError)` - Initialization failed
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

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }
}
