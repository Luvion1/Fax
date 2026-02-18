//! GC Timer - Timing Utilities
//!
//! Utility for timing GC operations.
//! Using high-precision timer (nanoseconds).

use std::time::Duration;

/// GcTimer - timer for measuring GC operations
pub struct GcTimer {
    start: std::time::Instant,
}

impl GcTimer {
    /// Create new timer
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed nanoseconds
    pub fn elapsed_ns(&self) -> u64 {
        self.elapsed().as_nanos() as u64
    }

    /// Get elapsed microseconds
    pub fn elapsed_us(&self) -> u64 {
        self.elapsed().as_micros() as u64
    }

    /// Get elapsed milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }
}

impl Default for GcTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped timer for automatic timing
pub struct ScopedTimer<'a> {
    name: &'a str,
    start: std::time::Instant,
    callback: Box<dyn FnMut(Duration) + 'a>,
}

impl<'a> ScopedTimer<'a> {
    /// Create scoped timer
    pub fn new<F>(name: &'a str, mut callback: F) -> Self
    where
        F: FnMut(Duration) + 'a,
    {
        Self {
            name,
            start: std::time::Instant::now(),
            callback: Box::new(callback),
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        (self.callback)(duration);
    }
}
