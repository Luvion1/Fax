//! Safepoint Management
//!
//! Safepoint is a point where threads can be interrupted for GC.
//! Threads will block at safepoint until GC completes.
//!
//! Safepoint Strategy:
//! 1. Polling-based safepoints with atomic state machine
//! 2. Thread blocks at safepoint when GC is requested
//! 3. Thread resumes after GC completes
//!
//! ## Safepoint States
//!
//! ```text
//! SAFEPOINT_NONE (0) ─────┐
//!     │                   │
//!     ▼                   │
//! SAFEPOINT_REQUESTED (1) │
//!     │                   │
//!     ▼                   │
//! SAFEPOINT_REACHED (2) ──┘ (after GC release)
//! ```
//!
//! ## Thread Pause Mechanism
//!
//! For proper stack scanning, mutator threads must be paused at safepoints:
//! 1. GC thread requests safepoint (sets state to REQUESTED)
//! 2. Mutator threads poll safepoint and arrive when requested
//! 3. GC waits for all threads to reach safepoint
//! 4. GC performs stack scanning while threads are paused
//! 5. GC releases safepoint and threads resume

use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// Safepoint state constants
pub const SAFEPOINT_NONE: u8 = 0;
pub const SAFEPOINT_REQUESTED: u8 = 1;
pub const SAFEPOINT_REACHED: u8 = 2;

/// Safepoint - coordination point for GC thread synchronization
///
/// Manages safepoint coordination between GC threads and mutator threads.
/// Uses atomic state machine for lock-free operation in the common case.
///
/// # Thread Safety
///
/// Safepoint is designed for concurrent access:
/// - Multiple mutator threads can arrive/release simultaneously
/// - GC thread coordinates safepoint request/release
/// - All operations are atomic and lock-free
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::runtime::safepoint::{Safepoint, SAFEPOINT_NONE};
///
/// let safepoint = Safepoint::new(4); // 4 threads
///
/// // GC thread requests safepoint
/// safepoint.request_safepoint();
///
/// // Mutator thread arrives at safepoint
/// safepoint.arrive();
///
/// // GC waits for all threads
/// safepoint.wait_for_safepoint();
///
/// // GC releases safepoint
/// safepoint.release_safepoint();
/// ```
pub struct Safepoint {
    /// Current safepoint state
    state: AtomicU8,

    /// Number of threads that have arrived at safepoint
    paused_threads: AtomicUsize,

    /// Total number of threads that must reach safepoint
    total_threads: AtomicUsize,
}

impl Safepoint {
    /// Create new safepoint with specified thread count
    ///
    /// # Arguments
    /// * `total_threads` - Number of threads expected to reach safepoint
    ///
    /// # Returns
    /// New Safepoint instance in NONE state
    pub fn new(total_threads: usize) -> Self {
        Self {
            state: AtomicU8::new(SAFEPOINT_NONE),
            paused_threads: AtomicUsize::new(0),
            total_threads: AtomicUsize::new(total_threads),
        }
    }

    /// Request all threads to reach safepoint
    ///
    /// Called by GC thread to initiate a safepoint.
    /// Sets state to REQUESTED, signaling mutator threads to stop.
    ///
    /// # Memory Ordering
    /// Uses SeqCst to ensure all threads see the request immediately.
    pub fn request_safepoint(&self) {
        self.state.store(SAFEPOINT_REQUESTED, Ordering::SeqCst);
    }

    /// Wait for all threads to reach safepoint
    ///
    /// Called by GC thread after requesting safepoint.
    /// Blocks until all registered threads have arrived.
    ///
    /// # Memory Ordering
    /// Uses Acquire to synchronize with arriving threads.
    pub fn wait_for_safepoint(&self) {
        let total = self.total_threads.load(Ordering::Acquire);

        while self.paused_threads.load(Ordering::Acquire) < total {
            std::hint::spin_loop();
        }
    }

    /// Thread signals it has reached safepoint
    ///
    /// Called by mutator threads when they reach a safepoint.
    /// Increments the arrived counter and updates state.
    ///
    /// # Memory Ordering
    /// Uses AcqRel to synchronize with GC thread's wait.
    pub fn arrive(&self) {
        self.paused_threads.fetch_add(1, Ordering::AcqRel);
        self.state.store(SAFEPOINT_REACHED, Ordering::Release);
    }

    /// Release safepoint and resume all threads
    ///
    /// Called by GC thread after completing GC work.
    /// Resets state and counter for next safepoint.
    ///
    /// # Memory Ordering
    /// Uses Release to ensure all GC work is visible before release.
    pub fn release_safepoint(&self) {
        self.paused_threads.store(0, Ordering::Release);
        self.state.store(SAFEPOINT_NONE, Ordering::Release);
    }

    /// Check if safepoint is requested
    ///
    /// Called by mutator threads to check if they should block.
    /// Should be called at poll points in mutator code.
    ///
    /// # Returns
    /// `true` if safepoint has been requested
    ///
    /// # Memory Ordering
    /// Uses Acquire to synchronize with GC thread's request.
    pub fn is_requested(&self) -> bool {
        self.state.load(Ordering::Acquire) != SAFEPOINT_NONE
    }

    /// Get current safepoint state
    ///
    /// # Returns
    /// Current state constant (NONE, REQUESTED, or REACHED)
    pub fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    /// Get number of threads at safepoint
    ///
    /// # Returns
    /// Count of threads that have arrived
    pub fn threads_at_safepoint(&self) -> usize {
        self.paused_threads.load(Ordering::Acquire)
    }

    /// Get total thread count
    ///
    /// # Returns
    /// Expected number of threads
    pub fn total_threads(&self) -> usize {
        self.total_threads.load(Ordering::Acquire)
    }

    /// Update total thread count
    ///
    /// Called when threads are created or destroyed.
    ///
    /// # Arguments
    /// * `count` - New total thread count
    pub fn set_total_threads(&self, count: usize) {
        self.total_threads.store(count, Ordering::Release);
    }

    /// Block at safepoint until released
    ///
    /// Combined operation: arrive at safepoint and wait for release.
    /// Called by mutator threads when they detect a safepoint request.
    ///
    /// # Memory Ordering
    /// Uses appropriate ordering for arrive and spin-wait.
    pub fn block_until_released(&self) {
        // Signal arrival
        self.arrive();

        // Wait for release (state back to NONE)
        while self.state.load(Ordering::Acquire) != SAFEPOINT_NONE {
            std::hint::spin_loop();
        }
    }
}

impl Default for Safepoint {
    fn default() -> Self {
        Self::new(1)
    }
}

/// SafepointManager - manager for safepoint coordination
///
/// Manages safepoints for all threads.
/// Deprecated: Use `Safepoint` directly for new code.
pub struct SafepointManager {
    /// GC in progress
    gc_in_progress: AtomicBool,

    /// Threads at safepoint
    threads_at_safepoint: std::sync::atomic::AtomicUsize,

    /// Total threads
    total_threads: std::sync::atomic::AtomicUsize,

    /// Safepoint lock
    #[allow(dead_code)]
    lock: std::sync::Mutex<()>,
}

use std::sync::atomic::AtomicBool;

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

    /// Check if thread should block at safepoint
    pub fn should_block(&self) -> bool {
        self.gc_in_progress.load(Ordering::Relaxed)
    }

    /// Block thread at safepoint
    pub fn block_at_safepoint(&self) {
        self.threads_at_safepoint.fetch_add(1, Ordering::SeqCst);

        // Wait until GC completes
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

/// Safepoint guard for RAII
pub struct SafepointGuard<'a> {
    manager: &'a SafepointManager,
}

impl<'a> SafepointGuard<'a> {
    /// Create safepoint guard
    pub fn new(manager: &'a SafepointManager) -> Self {
        manager.threads_at_safepoint.fetch_add(1, Ordering::SeqCst);
        Self { manager }
    }

    /// Check if should block
    pub fn check(&self) {
        if self.manager.should_block() {
            self.manager.block_at_safepoint();
        }
    }
}

impl<'a> Drop for SafepointGuard<'a> {
    fn drop(&mut self) {
        self.manager
            .threads_at_safepoint
            .fetch_sub(1, Ordering::SeqCst);
    }
}
