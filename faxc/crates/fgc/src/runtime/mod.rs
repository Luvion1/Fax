//! Runtime Module - GC Runtime Integration
//!
//! Module ini mengintegrasikan GC dengan runtime Fax.
//! Mengelola:
//! - GC initialization
//! - Safepoint management
//! - Finalizer queue
//! - GC thread lifecycle

pub mod init;
pub mod safepoint;
pub mod finalizer;

pub use init::RuntimeInitializer;
pub use safepoint::SafepointManager;
pub use finalizer::Finalizer;

use std::sync::Arc;

/// Runtime - GC runtime orchestrator
///
/// Mengkoordinasikan seluruh GC runtime components.
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
        *self.state.lock().unwrap() = RuntimeState::Running;
        self.safepoint_manager.start()?;
        self.finalizer.start()?;
        Ok(())
    }

    /// Stop runtime
    pub fn stop(&self) -> Result<(), crate::error::FgcError> {
        *self.state.lock().unwrap() = RuntimeState::Stopping;

        self.gc.shutdown()?;
        self.safepoint_manager.stop()?;
        self.finalizer.stop()?;

        *self.state.lock().unwrap() = RuntimeState::Stopped;

        Ok(())
    }

    /// Get GC instance
    pub fn gc(&self) -> &Arc<crate::gc::GarbageCollector> {
        &self.gc
    }

    /// Get runtime state
    pub fn state(&self) -> RuntimeState {
        *self.state.lock().unwrap()
    }

    /// Request GC
    pub fn request_gc(&self, generation: crate::gc::GcGeneration) {
        self.gc.request_gc(
            generation,
            crate::gc::GcReason::Explicit,
        );
    }

    /// Allocate object
    pub fn allocate(&self, size: usize) -> Result<usize, crate::error::FgcError> {
        let heap = self.gc.heap();
        heap.allocate_tlab_memory(size)
    }

    /// Register finalizer untuk object
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
    /// Runtime belum di-start
    Initialized,
    /// Runtime berjalan normal
    Running,
    /// Runtime sedang stop
    Stopping,
    /// Runtime sudah stop
    Stopped,
}

/// GC trigger helper
pub struct GcTrigger;

impl GcTrigger {
    /// Trigger full GC
    pub fn full_gc(runtime: &Runtime) {
        runtime.request_gc(crate::gc::GcGeneration::Full);
    }

    /// Trigger young GC
    pub fn young_gc(runtime: &Runtime) {
        runtime.request_gc(crate::gc::GcGeneration::Young);
    }
}
