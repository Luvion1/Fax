//! Safepoint Management
//!
//! Safepoint adalah titik dimana thread bisa di-interrupt untuk GC.
//! Thread akan block di safepoint sampai GC complete.
//!
//! Safepoint Strategy:
//! 1. Polling-based safepoints
//! 2. Thread block di safepoint saat GC request
//! 3. Thread resume setelah GC complete

use std::sync::atomic::{AtomicBool, Ordering};

/// SafepointManager - manager untuk safepoint coordination
///
/// Mengelola safepoint untuk semua threads.
pub struct SafepointManager {
    /// GC in progress
    gc_in_progress: AtomicBool,

    /// Threads at safepoint
    threads_at_safepoint: std::sync::atomic::AtomicUsize,

    /// Total threads
    total_threads: std::sync::atomic::AtomicUsize,

    /// Safepoint lock
    lock: std::sync::Mutex<()>,
}

impl SafepointManager {
    /// Create new safepoint manager
    pub fn new() -> Self {
        Self {
            gc_in_progress: AtomicBool::new(false),
            threads_at_safepoint: std::sync::atomic::AtomicUsize::new(0),
            total_threads: std::sync::atomic::AtomicUsize::new(0),
            lock: std::sync::Mutex::new(()),
        }
    }

    /// Start safepoint manager
    pub fn start(&self) -> crate::error::Result<()> {
        Ok(())
    }

    /// Stop safepoint manager
    pub fn stop(&self) -> crate::error::Result<()> {
        Ok(())
    }

    /// Check jika thread harus block di safepoint
    pub fn should_block(&self) -> bool {
        self.gc_in_progress.load(Ordering::Relaxed)
    }

    /// Block thread di safepoint
    pub fn block_at_safepoint(&self) {
        self.threads_at_safepoint.fetch_add(1, Ordering::SeqCst);

        // Wait sampai GC complete
        while self.gc_in_progress.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        self.threads_at_safepoint.fetch_sub(1, Ordering::SeqCst);
    }

    /// Request safepoint (start GC)
    pub fn request_safepoint(&self) {
        self.gc_in_progress.store(true, Ordering::SeqCst);

        // Wait semua threads di safepoint
        while self.threads_at_safepoint.load(Ordering::Relaxed)
            < self.total_threads.load(Ordering::Relaxed)
        {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// Release safepoint (GC complete)
    pub fn release_safepoint(&self) {
        self.gc_in_progress.store(false, Ordering::SeqCst);
    }

    /// Set total threads
    pub fn set_total_threads(&self, count: usize) {
        self.total_threads.store(count, Ordering::Relaxed);
    }

    /// Get threads at safepoint count
    pub fn threads_at_safepoint(&self) -> usize {
        self.threads_at_safepoint.load(Ordering::Relaxed)
    }

    /// Check semua threads di safepoint
    pub fn all_at_safepoint(&self) -> bool {
        self.threads_at_safepoint.load(Ordering::Relaxed)
            >= self.total_threads.load(Ordering::Relaxed)
    }
}

impl Default for SafepointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Safepoint guard untuk RAII
pub struct SafepointGuard<'a> {
    manager: &'a SafepointManager,
}

impl<'a> SafepointGuard<'a> {
    /// Create safepoint guard
    pub fn new(manager: &'a SafepointManager) -> Self {
        manager.threads_at_safepoint.fetch_add(1, Ordering::SeqCst);
        Self { manager }
    }

    /// Check jika harus block
    pub fn check(&self) {
        if self.manager.should_block() {
            self.manager.block_at_safepoint();
        }
    }
}

impl<'a> Drop for SafepointGuard<'a> {
    fn drop(&mut self) {
        self.manager.threads_at_safepoint.fetch_sub(1, Ordering::SeqCst);
    }
}
