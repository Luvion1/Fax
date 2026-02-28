//! Finalizer - Object Finalization
//!
//! Finalizer queue for objects that need cleanup before being collected.
//! Finalizers are called after object becomes unreachable but before
//! memory is reclaimed.
//!
//! Warning: Finalizers should be avoided if possible because:
//! - Performance overhead
//! - Unpredictable timing
//! - Potential memory leaks
//!
//! Use only for cleanup of native resources.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Finalizer - manager for object finalization
///
/// Manages finalizer queue and execution.
pub struct Finalizer {
    /// Finalizer queue
    queue: Arc<std::sync::Mutex<VecDeque<FinalizerEntry>>>,

    /// Finalizer thread handle
    thread_handle: std::sync::Mutex<Option<std::thread::JoinHandle<()>>>,

    /// Running flag
    running: Arc<AtomicBool>,

    /// Pending finalizers count
    pending_count: AtomicUsize,
}

impl Finalizer {
    /// Create new finalizer
    pub fn new() -> Self {
        Self {
            queue: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            thread_handle: std::sync::Mutex::new(None),
            running: Arc::new(AtomicBool::new(false)),
            pending_count: AtomicUsize::new(0),
        }
    }

    /// Start finalizer thread
    pub fn start(&self) -> crate::error::Result<()> {
        self.running.store(true, Ordering::Relaxed);

        // Spawn finalizer thread
        let queue = self.queue.clone();
        let running_atomic = self.running.clone();

        let handle = std::thread::spawn(move || {
            while running_atomic.load(Ordering::Relaxed) {
                // Process finalizers
                let mut queue_guard = match queue.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        eprintln!("[Finalizer] Failed to lock queue: {}", e);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    },
                };

                while let Some(entry) = queue_guard.pop_front() {
                    // Execute finalizer
                    (entry.finalizer_fn)(entry.object);
                }

                drop(queue_guard);

                // Sleep briefly
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        *self.thread_handle.lock().map_err(|e| {
            crate::error::FgcError::LockPoisoned(format!("finalizer thread handle poisoned: {}", e))
        })? = Some(handle);

        Ok(())
    }

    /// Stop finalizer thread
    pub fn stop(&self) -> crate::error::Result<()> {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self
            .thread_handle
            .lock()
            .map_err(|e| {
                crate::error::FgcError::LockPoisoned(format!(
                    "finalizer thread handle poisoned: {}",
                    e
                ))
            })?
            .take()
        {
            let _ = handle.join();
        }

        Ok(())
    }

    /// Register finalizer for object
    ///
    /// # Arguments
    /// * `object` - Object address
    /// * `finalizer_fn` - Finalizer function
    pub fn register<F>(&self, object: usize, finalizer_fn: F)
    where
        F: FnOnce(usize) + Send + 'static,
    {
        let entry = FinalizerEntry {
            object,
            finalizer_fn: Box::new(finalizer_fn),
        };

        self.queue.lock().unwrap().push_back(entry);
        self.pending_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get pending finalizers count
    pub fn pending_count(&self) -> usize {
        self.pending_count.load(Ordering::Relaxed)
    }

    /// Check if there are pending finalizers
    pub fn has_pending(&self) -> bool {
        self.pending_count.load(Ordering::Relaxed) > 0
    }

    /// Clear pending finalizers
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
        self.pending_count.store(0, Ordering::Relaxed);
    }
}

impl Default for Finalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Finalizer entry
struct FinalizerEntry {
    /// Object address
    object: usize,
    /// Finalizer function
    finalizer_fn: Box<dyn FnOnce(usize) + Send>,
}

/// Finalizer builder pattern
pub struct FinalizerBuilder {
    object: usize,
    finalizer_fn: Option<Box<dyn FnOnce(usize) + Send>>,
}

impl FinalizerBuilder {
    /// Create new builder
    pub fn new(object: usize) -> Self {
        Self {
            object,
            finalizer_fn: None,
        }
    }

    /// Set finalizer function
    pub fn with_finalizer<F>(mut self, finalizer_fn: F) -> Self
    where
        F: FnOnce(usize) + Send + 'static,
    {
        self.finalizer_fn = Some(Box::new(finalizer_fn));
        self
    }

    /// Build dan register finalizer
    pub fn register(self, finalizer: &Finalizer) {
        if let Some(fn_box) = self.finalizer_fn {
            finalizer.register(self.object, fn_box);
        }
    }
}
