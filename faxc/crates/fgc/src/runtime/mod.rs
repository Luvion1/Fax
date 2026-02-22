//! Runtime Module - GC Runtime Integration
//!
//! This module integrates GC with the Fax runtime.
//! Manages:
//! - GC initialization
//! - Safepoint management
//! - Finalizer queue
//! - GC thread lifecycle

pub mod finalizer;
pub mod init;
pub mod jit;
pub mod safepoint;

pub use finalizer::Finalizer;
pub use init::RuntimeInitializer;
pub use jit::{
    JitEvent, JitGcInterface, JitIntegration, JitRoot, JitRootType, JitStats, NoopJitIntegration,
};
pub use safepoint::SafepointManager;

use std::sync::Arc;

/// Runtime - GC runtime orchestrator
///
/// Coordinates all GC runtime components.
pub struct Runtime {
    /// GC instance
    gc: Arc<crate::gc::GarbageCollector>,

    /// Safepoint manager
    safepoint_manager: SafepointManager,

    /// Finalizer
    finalizer: Finalizer,

    /// Runtime state
    state: std::sync::Mutex<RuntimeState>,
}

impl Runtime {
    /// Create new runtime
    pub fn new(config: crate::config::GcConfig) -> Result<Self, crate::error::FgcError> {
        let gc = Arc::new(crate::gc::GarbageCollector::new(config.clone())?);

        Ok(Self {
            gc,
            safepoint_manager: SafepointManager::new(),
            finalizer: Finalizer::new(),
            state: std::sync::Mutex::new(RuntimeState::Initialized),
        })
    }

    /// Start runtime
    pub fn start(&self) -> Result<(), crate::error::FgcError> {
        *self.state.lock().map_err(|e| {
            crate::error::FgcError::LockPoisoned(format!("state mutex poisoned: {}", e))
        })? = RuntimeState::Running;
        self.safepoint_manager.start()?;
        self.finalizer.start()?;
        Ok(())
    }

    /// Stop runtime
    pub fn stop(&self) -> Result<(), crate::error::FgcError> {
        *self.state.lock().map_err(|e| {
            crate::error::FgcError::LockPoisoned(format!("state mutex poisoned: {}", e))
        })? = RuntimeState::Stopping;

        self.gc.shutdown()?;
        self.safepoint_manager.stop()?;
        self.finalizer.stop()?;

        *self.state.lock().map_err(|e| {
            crate::error::FgcError::LockPoisoned(format!("state mutex poisoned: {}", e))
        })? = RuntimeState::Stopped;

        Ok(())
    }

    /// Get GC instance
    pub fn gc(&self) -> &Arc<crate::gc::GarbageCollector> {
        &self.gc
    }

    /// Get runtime state
    pub fn state(&self) -> Result<RuntimeState, crate::error::FgcError> {
        Ok(*self.state.lock().map_err(|e| {
            crate::error::FgcError::LockPoisoned(format!("state mutex poisoned: {}", e))
        })?)
    }

    /// Request GC
    pub fn request_gc(&self, generation: crate::gc::GcGeneration) -> crate::error::Result<()> {
        self.gc
            .request_gc(generation, crate::gc::GcReason::Explicit)
    }

    /// Allocate object
    pub fn allocate(&self, size: usize) -> Result<usize, crate::error::FgcError> {
        let heap = self.gc.heap();
        heap.allocate_tlab_memory(size)
    }

    /// Register finalizer for object
    pub fn register_finalizer<F>(&self, object: usize, finalizer_fn: F)
    where
        F: FnOnce(usize) + Send + 'static,
    {
        self.finalizer.register(object, finalizer_fn);
    }

    /// Check safepoint
    pub fn check_safepoint(&self) {
        if self.safepoint_manager.should_block() {
            self.safepoint_manager.block_at_safepoint();
        }
    }
}

/// Runtime state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Runtime not yet started
    Initialized,
    /// Runtime running normally
    Running,
    /// Runtime is stopping
    Stopping,
    /// Runtime has stopped
    Stopped,
}

/// GC trigger helper
pub struct GcTrigger;

impl GcTrigger {
    /// Trigger full GC
    pub fn full_gc(runtime: &Runtime) {
        let _ = runtime.request_gc(crate::gc::GcGeneration::Full);
    }

    /// Trigger young GC
    pub fn young_gc(runtime: &Runtime) {
        let _ = runtime.request_gc(crate::gc::GcGeneration::Young);
    }
}
