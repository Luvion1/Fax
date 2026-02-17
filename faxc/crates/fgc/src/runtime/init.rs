//! Runtime Initialization
//!
//! Module untuk GC runtime initialization.

use crate::config::GcConfig;
use crate::error::Result;
use std::sync::Once;

/// RuntimeInitializer - initializer untuk GC runtime
///
/// Mengelola initialization sequence:
/// 1. Validate configuration
/// 2. Initialize heap
/// 3. Start GC threads
/// 4. Register shutdown hooks
pub struct RuntimeInitializer {
    config: GcConfig,
    initialized: std::sync::atomic::AtomicBool,
}

/// Shutdown guard untuk cleanup otomatis
struct ShutdownGuard;

impl ShutdownGuard {
    fn new() -> Self {
        Self
    }
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        // Cleanup saat program exit
        // Note: Ini dipanggil saat runtime drop
    }
}

static SHUTDOWN_INIT: Once = Once::new();
static mut SHUTDOWN_GUARD: Option<ShutdownGuard> = None;

impl RuntimeInitializer {
    /// Create new initializer dengan config
    pub fn new(config: GcConfig) -> Self {
        Self {
            config,
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Initialize runtime
    ///
    /// # Returns
    /// Result dengan initialized Runtime
    pub fn initialize(&self) -> Result<super::Runtime> {
        if self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(crate::error::FgcError::Internal(
                "Runtime already initialized".to_string(),
            ));
        }

        // Validate config
        self.config.validate().map_err(|e| {
            crate::error::FgcError::Configuration(format!("Invalid config: {}", e))
        })?;

        // Create runtime
        let runtime = super::Runtime::new(self.config.clone())?;

        // Start runtime
        runtime.start()?;

        // Register shutdown hook
        self.register_shutdown_hook();

        self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);

        Ok(runtime)
    }

    /// Register shutdown hook untuk cleanup
    fn register_shutdown_hook(&self) {
        // Setup shutdown guard yang akan dipanggil saat program exit
        SHUTDOWN_INIT.call_once(|| {
            unsafe {
                SHUTDOWN_GUARD = Some(ShutdownGuard::new());
            }
        });
    }

    /// Get configuration
    pub fn config(&self) -> &GcConfig {
        &self.config
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Initialize GC dengan default config
pub fn init_default() -> Result<super::Runtime> {
    let config = GcConfig::default();
    let initializer = RuntimeInitializer::new(config);
    initializer.initialize()
}

/// Initialize GC dengan custom config
pub fn init_with_config(config: GcConfig) -> Result<super::Runtime> {
    let initializer = RuntimeInitializer::new(config);
    initializer.initialize()
}
