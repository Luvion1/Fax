//! Atomic Utilities
//!
//! Helper functions untuk atomic operations.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// AtomicUtils - utility untuk atomic operations
pub struct AtomicUtils;

impl AtomicUtils {
    /// Atomic fetch-add dengan saturasi
    ///
    /// Tidak overflow, saturate di max value.
    pub fn saturating_add(atomic: &AtomicUsize, value: usize) -> usize {
        let mut current = atomic.load(Ordering::Relaxed);

        loop {
            let new_value = current.saturating_add(value);

            match atomic.compare_exchange_weak(
                current,
                new_value,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomic fetch-sub dengan saturasi
    pub fn saturating_sub(atomic: &AtomicUsize, value: usize) -> usize {
        let mut current = atomic.load(Ordering::Relaxed);

        loop {
            let new_value = current.saturating_sub(value);

            match atomic.compare_exchange_weak(
                current,
                new_value,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomic swap jika kondisi terpenuhi
    pub fn swap_if(atomic: &AtomicBool, expected: bool, new_value: bool) -> bool {
        match atomic.compare_exchange_weak(
            expected,
            new_value,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Spin wait dengan backoff
    pub fn spin_wait<F>(mut condition: F, max_spins: usize) -> bool
    where
        F: FnMut() -> bool,
    {
        let mut spins = 0;
        let mut backoff = 1;

        while !condition() {
            if spins >= max_spins {
                return false;
            }

            // Exponential backoff
            for _ in 0..backoff {
                std::hint::spin_loop();
            }

            backoff = (backoff * 2).min(1000);
            spins += 1;
        }

        true
    }
}
