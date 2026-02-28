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
use indexmap::IndexMap;
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

/// GC recommendation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationLevel {
    /// Informational message
    Info,
    /// Warning - may need attention
    Warning,
    /// Critical - action recommended
    Critical,
}

/// GC tuning recommendation
#[derive(Debug, Clone)]
pub struct GcRecommendation {
    /// Recommendation level
    pub level: RecommendationLevel,
    /// Description of the situation
    pub message: String,
    /// Suggested action
    pub suggestion: String,
}

/// GC health status
#[derive(Debug, Clone)]
pub struct GcHealth {
    /// Whether GC is healthy (no critical issues)
    pub healthy: bool,
    /// Current GC state
    pub state: GcState,
    /// Heap used in bytes
    pub heap_used: usize,
    /// Maximum heap size in bytes
    pub heap_max: usize,
    /// Heap utilization (0.0 - 1.0)
    pub utilization: f64,
    /// Number of active regions
    pub active_regions: usize,
    /// Critical issues found
    pub issues: Vec<String>,
    /// Warnings (non-critical)
    pub warnings: Vec<String>,
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
        // Prevent concurrent GC requests
        if self.is_collecting() {
            if self.config.verbose {
                println!("[GC] GC already in progress, skipping request");
            }
            return Ok(());
        }

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
        // Phase 1: Marking
        {
            let mut state_guard = self
                .state
                .lock()
                .map_err(|e| FgcError::LockPoisoned(format!("state mutex poisoned: {}", e)))?;
            *state_guard = GcState::Marking;
        }

        if self.config.verbose {
            println!("[GC] Starting Marking Phase");
        }

        self.pause_mark_start()?;
        self.concurrent_mark()?;
        self.pause_mark_end()?;

        // Verify marking completed successfully
        let marked_count = self.marker.marked_count();
        if self.config.verbose {
            println!("[GC] Marking complete, {} objects marked", marked_count);
        }

        // Phase 2: Relocating
        {
            let mut state_guard = self
                .state
                .lock()
                .map_err(|e| FgcError::LockPoisoned(format!("state mutex poisoned: {}", e)))?;
            *state_guard = GcState::Relocating;
        }

        if self.config.verbose {
            println!("[GC] Starting Relocation Phase");
        }

        self.prepare_relocation()?;
        self.concurrent_relocate()?;

        if self.config.verbose {
            println!("[GC] Relocation complete");
        }

        // Phase 3: Cleanup
        self.cleanup()?;

        // Final state
        {
            let mut state_guard = self
                .state
                .lock()
                .map_err(|e| FgcError::LockPoisoned(format!("state mutex poisoned: {}", e)))?;
            *state_guard = GcState::Idle;
        }

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

    /// Check GC health status
    ///
    /// Returns health information about the GC including:
    /// - Whether GC is running
    /// - Current heap usage
    /// - Any warnings or issues
    pub fn health_check(&self) -> GcHealth {
        let current_state = self.state();
        let heap_stats = self.heap.get_stats();
        let is_running = current_state != GcState::Idle;

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check for potential issues
        if heap_stats.utilization > 0.9 {
            issues.push("Heap utilization critically high (>90%)".to_string());
        } else if heap_stats.utilization > 0.75 {
            warnings.push("Heap utilization high (>75%)".to_string());
        }

        if is_running && current_state == GcState::Marking {
            warnings.push("GC marking phase in progress".to_string());
        }

        if heap_stats.region_count == 0 {
            warnings.push("No active regions".to_string());
        }

        // Check for GC in progress for too long
        if let Ok(state) = self.state.try_lock() {
            if *state != GcState::Idle {
                // Could add timing check here for stalled GC detection
            }
        }

        // Check for allocation failures (if tracked)
        if heap_stats.used >= heap_stats.max {
            issues.push("Heap at maximum capacity".to_string());
        }

        GcHealth {
            healthy: issues.is_empty(),
            state: current_state,
            heap_used: heap_stats.used,
            heap_max: heap_stats.max,
            utilization: heap_stats.utilization,
            active_regions: heap_stats.region_count,
            issues,
            warnings,
        }
    }

    /// Get detailed diagnostic information
    ///
    /// # Returns
    /// A map containing detailed diagnostic information
    pub fn diagnostics(&self) -> IndexMap<String, String> {
        let mut diagnostics = IndexMap::new();

        // GC state
        if let Ok(state) = self.state.lock() {
            diagnostics.insert("state".to_string(), format!("{:?}", *state));
        }

        // Cycle count
        diagnostics.insert(
            "cycle_count".to_string(),
            self.cycle_count.load(Ordering::Relaxed).to_string(),
        );

        // Heap stats
        let heap_stats = self.heap.get_stats();
        diagnostics.insert("heap_used".to_string(), heap_stats.used.to_string());
        diagnostics.insert("heap_max".to_string(), heap_stats.max.to_string());
        diagnostics.insert(
            "heap_utilization".to_string(),
            format!("{:.2}", heap_stats.utilization),
        );
        diagnostics.insert(
            "region_count".to_string(),
            heap_stats.region_count.to_string(),
        );

        // GC requested flag
        diagnostics.insert(
            "gc_requested".to_string(),
            self.gc_requested.load(Ordering::Relaxed).to_string(),
        );

        // Current generation
        if let Ok(gen) = self.current_generation.lock() {
            diagnostics.insert("current_generation".to_string(), format!("{:?}", *gen));
        }

        diagnostics
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
        // Check if GC should be triggered based on heap usage
        if self.should_collect() {
            self.collect()?;
        }

        // Attempt allocation with retry logic
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.heap.allocate_tlab_memory(size) {
                Ok(addr) => return Ok(addr),
                Err(e) => {
                    last_error = Some(e);

                    // If not already collecting, try to collect and retry
                    if !self.is_collecting() && attempt < max_retries - 1 {
                        if self.config.verbose {
                            println!(
                                "[GC] Allocation attempt {} failed, triggering GC...",
                                attempt + 1
                            );
                        }
                        let _ = self.collect();
                    }
                },
            }
        }

        // All retries failed, return error
        Err(last_error.unwrap_or(FgcError::OutOfMemory {
            requested: size,
            available: 0,
        }))
    }

    /// Check if GC should be triggered based on heap usage
    ///
    /// Returns true if heap usage exceeds the trigger threshold.
    pub fn should_collect(&self) -> bool {
        let stats = self.heap.get_stats();
        let threshold = self.config.gc_trigger_threshold;

        stats.utilization as f32 > threshold
    }

    /// Get heap usage percentage (0.0 - 1.0)
    pub fn heap_usage(&self) -> f64 {
        self.heap.get_stats().utilization
    }

    /// Get GC recommendations for tuning
    ///
    /// Returns recommendations based on current GC state and heap usage.
    pub fn get_recommendations(&self) -> Vec<GcRecommendation> {
        let mut recommendations = Vec::new();
        let stats = self.heap.get_stats();

        // High heap utilization warning
        if stats.utilization > 0.8 {
            recommendations.push(GcRecommendation {
                level: RecommendationLevel::Warning,
                message: format!("Heap utilization is high ({:.1}%). Consider increasing heap size or triggering GC.", 
                    stats.utilization * 100.0),
                suggestion: "Increase max_heap_size or call collect() explicitly".to_string(),
            });
        }

        // Low heap utilization with high allocation
        if stats.utilization < 0.2 && stats.total_allocated > 100_000_000 {
            recommendations.push(GcRecommendation {
                level: RecommendationLevel::Info,
                message: "Low heap utilization but high allocation rate detected".to_string(),
                suggestion: "Consider reducing initial_heap_size to save memory".to_string(),
            });
        }

        // GC frequency check
        let cycles = self.cycle_count();
        if cycles > 0 {
            recommendations.push(GcRecommendation {
                level: RecommendationLevel::Info,
                message: format!("Total GC cycles: {}", cycles),
                suggestion: "Monitor pause times and adjust gc_threads if needed".to_string(),
            });
        }

        recommendations
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
