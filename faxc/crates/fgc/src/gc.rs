//! GC Core Module - Garbage Collection Cycle Management
//!
//! Implements the core garbage collection algorithm.
//! FGC uses concurrent mark-compact with generational collection.

use crate::config::GcConfig;
use crate::error::{FgcError, Result};
use crate::heap::Heap;
use crate::marker::Marker;
use crate::relocate::Relocator;
use crate::stats::GcStats;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// GC cycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcState {
    /// Idle - no GC in progress
    Idle,
    /// Marking phase - identifying live objects
    Marking,
    /// Relocating phase - moving objects
    Relocating,
    /// Cleanup phase - freeing old regions
    Cleanup,
}

/// Generation being collected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcGeneration {
    /// Young generation only (minor GC)
    /// Fast, frequent, scans young regions only
    Young,
    /// Old generation only (partial major GC)
    /// Slower, less frequent
    Old,
    /// Full heap collection
    /// Slowest, triggered when heap nearly full
    Full,
}

/// Reason for GC trigger
#[derive(Debug, Clone)]
pub enum GcReason {
    /// Heap usage exceeded threshold
    HeapThreshold { used: usize, threshold: usize },
    /// Explicit GC request (user call)
    Explicit,
    /// Periodic GC (interval timer)
    Periodic,
    /// System memory pressure
    MemoryPressure,
    /// Shutdown - final cleanup
    Shutdown,
}

/// GarbageCollector - orchestrator for the entire GC cycle
///
/// Coordinates all GC components:
/// - Marker for concurrent marking
/// - Relocator for object relocation
/// - Heap for region management
/// - Stats for performance monitoring
///
/// ## Thread Safety
///
/// GarbageCollector is designed for concurrent access.
/// All operations use atomic or lock protection.
pub struct GarbageCollector {
    /// Managed heap
    heap: Arc<Heap>,

    /// Marker for concurrent marking
    marker: Marker,

    /// Relocator for object relocation
    relocator: Relocator,

    /// GC configuration
    config: Arc<GcConfig>,

    /// Statistics collector
    stats: Arc<GcStats>,

    /// Current GC state
    state: std::sync::Mutex<GcState>,

    /// GC cycle counter
    cycle_count: AtomicU64,

    /// GC request flag
    gc_requested: AtomicBool,

    /// Current generation being collected
    current_generation: std::sync::Mutex<GcGeneration>,
}

impl GarbageCollector {
    /// Create a new GarbageCollector with specified configuration
    ///
    /// # Arguments
    /// * `config` - GC configuration parameters
    ///
    /// # Returns
    /// Result with GC instance or error if initialization fails
    ///
    /// # Examples
    /// ```rust
    /// let config = GcConfig::default();
    /// let gc = GarbageCollector::new(config)?;
    /// ```
    pub fn new(config: GcConfig) -> Result<Self> {
        config
            .validate()
            .map_err(|e| FgcError::Configuration(format!("Invalid configuration: {}", e)))?;

        let config = Arc::new(config);
        let heap = Arc::new(Heap::new(config.clone())?);
        let marker = Marker::new(heap.clone());
        let relocator = Relocator::new(heap.clone());
        let stats = Arc::new(GcStats::new());

        Ok(Self {
            heap,
            marker,
            relocator,
            config,
            stats,
            state: std::sync::Mutex::new(GcState::Idle),
            cycle_count: AtomicU64::new(0),
            gc_requested: AtomicBool::new(false),
            current_generation: std::sync::Mutex::new(GcGeneration::Young),
        })
    }

    /// Request a GC cycle
    ///
    /// Triggers GC with specified generation and reason.
    /// GC runs asynchronously.
    ///
    /// # Arguments
    /// * `generation` - Generation to collect
    /// * `reason` - Reason for GC trigger (for logging/stats)
    ///
    /// # Errors
    /// Returns `LockPoisoned` error if another thread panicked while holding
    /// the generation mutex.
    pub fn request_gc(&self, generation: GcGeneration, reason: GcReason) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Requesting {:?} GC, reason: {:?}", generation, reason);
        }

        // QC-009 FIX: Use map_err instead of unwrap()
        let mut gen_guard = self
            .current_generation
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("generation mutex poisoned: {}", e)))?;
        *gen_guard = generation;
        drop(gen_guard);

        self.gc_requested.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Execute GC cycle
    ///
    /// Main GC entry point. Runs the entire GC cycle
    /// from marking through cleanup.
    ///
    /// # Errors
    /// Returns `LockPoisoned` error if another thread panicked while holding
    /// the generation mutex.
    pub fn collect(&self) -> Result<()> {
        // QC-009 FIX: Use map_err instead of unwrap()
        let generation = {
            let gen_guard = self
                .current_generation
                .lock()
                .map_err(|e| FgcError::LockPoisoned(format!("generation mutex poisoned: {}", e)))?;
            *gen_guard
        };

        let timer = crate::stats::GcTimer::new();

        self.execute_gc_cycle()?;

        self.finalize_gc_cycle(generation, &timer);

        Ok(())
    }

    /// Execute the GC cycle phases with state transitions
    ///
    /// Orchestrates all GC phases from mark start through cleanup.
    /// Manages state transitions between phases.
    ///
    /// # Returns
    /// Result indicating success or GC error
    fn execute_gc_cycle(&self) -> Result<()> {
        // Marking phase
        *self
            .state
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("state mutex poisoned: {}", e)))? =
            GcState::Marking;
        self.pause_mark_start()?;
        self.concurrent_mark()?;
        self.pause_mark_end()?;

        // Relocating phase
        *self
            .state
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("state mutex poisoned: {}", e)))? =
            GcState::Relocating;
        self.prepare_relocation()?;
        self.concurrent_relocate()?;

        // Cleanup phase
        self.cleanup()?;
        *self
            .state
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("state mutex poisoned: {}", e)))? =
            GcState::Idle;

        Ok(())
    }

    /// Finalize GC cycle with stats and logging
    ///
    /// Records statistics, updates cycle count, and logs completion.
    ///
    /// # Arguments
    /// * `generation` - Generation that was collected
    /// * `timer` - Timer measuring GC duration
    fn finalize_gc_cycle(&self, generation: GcGeneration, timer: &crate::stats::GcTimer) {
        let duration = timer.elapsed();
        self.stats.record_collection(
            self.cycle_count.load(Ordering::Relaxed),
            generation,
            duration,
        );
        self.cycle_count.fetch_add(1, Ordering::Relaxed);
        self.gc_requested.store(false, Ordering::SeqCst);

        if self.config.verbose {
            println!(
                "[GC] Collection complete in {:.2}ms",
                duration.as_secs_f64() * 1000.0
            );
        }
    }

    /// Pause Mark Start
    ///
    /// Brief STW pause (< 1ms) to setup marking:
    /// - Flip mark bits (Marked0 <-> Marked1)
    /// - Scan roots (stacks, globals)
    /// - Initialize mark queues
    fn pause_mark_start(&self) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Pause Mark Start (STW)");
        }

        // Flip mark bits for new GC cycle
        self.heap.flip_mark_bits();

        // Scan all roots
        self.marker.scan_roots()?;

        Ok(())
    }

    /// Concurrent Mark
    ///
    /// Marking runs concurrently with the application.
    /// Load barriers mark objects as they are accessed.
    fn concurrent_mark(&self) -> Result<()> {
        // Check if there's any work to do after scanning roots
        if !self.marker.has_mark_work() {
            // No roots scanned, skip concurrent marking phase entirely
            if self.config.verbose {
                println!("[GC] No roots to mark, skipping concurrent marking");
            }
            return Ok(());
        }

        let num_threads = self.config.gc_threads.unwrap_or(4);
        if self.config.verbose {
            println!("[GC] Concurrent Mark with {} threads", num_threads);
        }

        // Start concurrent marking with worker threads
        self.marker.start_concurrent_marking(num_threads)?;

        // Wait until marking complete
        self.marker.wait_completion()?;

        Ok(())
    }

    /// Pause Mark End
    ///
    /// Brief STW pause (< 1ms) to finalize marking:
    /// - Process remaining mark queue
    /// - Handle special cases (weak refs, finalizers)
    fn pause_mark_end(&self) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Pause Mark End (STW)");
        }

        // Finalize marking
        self.marker.finalize_marking()?;

        Ok(())
    }

    /// Prepare Relocation
    ///
    /// Setup relocation:
    /// - Select regions for relocation set
    /// - Allocate destination regions
    /// - Setup forwarding tables
    fn prepare_relocation(&self) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Prepare Relocation");
        }

        self.relocator.prepare_relocation()?;

        Ok(())
    }

    /// Concurrent Relocate
    ///
    /// Relocate objects concurrently:
    /// - Copy objects to destination regions
    /// - Update forwarding tables
    /// - Load barriers handle pointer healing
    fn concurrent_relocate(&self) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Concurrent Relocate");
        }

        self.relocator.start_relocation()?;
        self.relocator.wait_relocation_complete()?;

        Ok(())
    }

    /// Cleanup
    ///
    /// Cleanup after relocation:
    /// - Free source regions
    /// - Clear forwarding tables
    /// - Update heap statistics
    fn cleanup(&self) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Cleanup");
        }

        self.relocator.complete_relocation()?;
        self.heap.update_stats();

        Ok(())
    }

    /// Check if GC is currently running
    pub fn is_collecting(&self) -> bool {
        self.state
            .lock()
            .map(|g| *g != GcState::Idle)
            .unwrap_or(false)
    }

    /// Get current GC state
    pub fn state(&self) -> GcState {
        self.state.lock().map(|g| *g).unwrap_or(GcState::Idle)
    }

    /// Get heap reference for allocation
    pub fn heap(&self) -> &Arc<Heap> {
        &self.heap
    }

    /// Get GC statistics
    pub fn stats(&self) -> Arc<GcStats> {
        self.stats.clone_arc()
    }

    /// Get total GC cycles executed
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count.load(Ordering::Relaxed)
    }

    /// Allocate memory from the heap
    ///
    /// # Arguments
    /// * `size` - Size in bytes to allocate
    ///
    /// # Returns
    /// * `Ok(usize)` - Address of allocated memory
    /// * `Err(FgcError)` - Allocation failed
    pub fn allocate(&self, size: usize) -> Result<usize> {
        self.heap.allocate_tlab_memory(size)
    }

    /// Register a root with the GC
    ///
    /// Roots are references that the GC will trace from.
    /// Any object reachable from a root will not be collected.
    ///
    /// # Arguments
    /// * `address` - Address of the object to register as root
    ///
    /// # Returns
    /// * `Ok(())` - Root registered successfully
    /// * `Err(FgcError)` - Registration failed
    pub fn register_root(&self, address: usize) -> Result<()> {
        // Validate address is in GC-managed heap
        if !crate::heap::is_gc_managed_address(address) {
            return Err(FgcError::InvalidArgument(
                "Address must be in GC-managed heap".to_string(),
            ));
        }

        // Register with root scanner
        self.marker
            .root_scanner()
            .register_global_root(address, Some("anonymous_root"));

        Ok(())
    }

    /// Unregister a root from the GC
    ///
    /// # Arguments
    /// * `address` - Address of the object to unregister
    ///
    /// # Returns
    /// * `Ok(())` - Root unregistered successfully
    /// * `Err(FgcError)` - Unregistration failed
    pub fn unregister_root(&self, address: usize) -> Result<()> {
        // Find and unregister the root
        // Note: This is a simplified implementation
        // A full implementation would track root handles
        let _ = address; // Suppress unused warning
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        if self.config.verbose {
            println!("[GC] Shutdown");
        }

        self.request_gc(GcGeneration::Full, GcReason::Shutdown)?;

        // Wait for current GC to complete
        while self.is_collecting() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Cleanup marker threads
        self.marker.shutdown()?;

        Ok(())
    }
}

/// AllocationResult - result from allocation request
#[derive(Debug)]
pub struct AllocationResult {
    /// Pointer to allocated memory
    pub address: usize,
    /// Size of allocated object
    pub size: usize,
    /// Generation where object was allocated
    pub generation: GcGeneration,
}

/// GCInfo - information about current GC cycle
#[derive(Debug)]
pub struct GcInfo {
    /// Current generation being collected
    pub generation: GcGeneration,
    /// Reason for GC trigger
    pub reason: GcReason,
    /// Start time of GC cycle
    pub start_time: std::time::Instant,
    /// Estimated duration
    pub estimated_duration: std::time::Duration,
}
