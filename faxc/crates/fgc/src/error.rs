//! Error Module - FGC Error Types
//!
//! Defines all error types used in FGC.

use std::sync::PoisonError;
use thiserror::Error;

/// Main error type for all FGC operations
#[derive(Debug, Error)]
pub enum FgcError {
    #[error("Out of memory: requested {requested} bytes, available {available} bytes")]
    OutOfMemory { requested: usize, available: usize },

    #[error("Heap initialization failed: {0}")]
    HeapInitialization(String),

    #[error("Invalid pointer address: {address:#x}")]
    InvalidPointer { address: usize },

    #[error("Region allocation failed: {reason}")]
    RegionAllocationFailed { reason: String },

    #[error("Concurrent modification detected during {operation}")]
    ConcurrentModification { operation: String },

    #[error("GC cycle failed: {reason}")]
    GcCycleFailed { reason: String },

    #[error("Marking phase failed: {0}")]
    MarkingFailed(String),

    #[error("Relocation phase failed: {0}")]
    RelocationFailed(String),

    #[error("Forwarding table error: {0}")]
    ForwardingTableError(String),

    #[error("TLAB error: {0}")]
    TlabError(String),

    #[error("NUMA error: {0}")]
    NumaError(String),

    #[error("Virtual memory error: {0}")]
    VirtualMemoryError(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Atomic update failed: expected value changed during CAS operation, current={0:#x}")]
    AtomicUpdateFailed(usize),

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("Invalid state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Bounds check failed: index {index} out of bounds for length {length}")]
    BoundsCheckFailed { index: usize, length: usize },

    #[error("Alignment error: address {address:#x} is not aligned to {alignment} bytes")]
    AlignmentError { address: usize, alignment: usize },

    #[error("Operation timeout: {0}")]
    Timeout(String),

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

impl FgcError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            FgcError::OutOfMemory { .. }
                | FgcError::Timeout(_)
                | FgcError::ResourceExhausted { .. }
        )
    }

    /// Check if this error indicates a bug in the code
    pub fn is_bug(&self) -> bool {
        matches!(
            self,
            FgcError::InvalidState { .. }
                | FgcError::BoundsCheckFailed { .. }
                | FgcError::Internal(_)
                | FgcError::LockPoisoned(_)
        )
    }
}

impl<T> From<PoisonError<T>> for FgcError {
    fn from(err: PoisonError<T>) -> Self {
        FgcError::LockPoisoned(err.to_string())
    }
}

/// Result type alias for FGC operations
pub type Result<T> = std::result::Result<T, FgcError>;

/// Macro to handle mutex lock with proper error handling
#[macro_export]
macro_rules! lock_result {
    ($lock:expr) => {
        $lock.map_err(|e| $crate::error::FgcError::from(e))
    };
}

/// Macro to unwrap mutex with clear panic message
#[macro_export]
macro_rules! lock_unwrap {
    ($lock:expr) => {
        $lock.unwrap_or_else(|e| {
            panic!(
                "Mutex poisoned - another thread panicked while holding the lock: {}",
                e
            )
        })
    };
}

/// Macro for assertion with context
#[macro_export]
macro_rules! assert_context {
    ($cond:expr, $context:expr) => {
        if !$cond {
            panic!("Assertion failed at {}: {}", stringify!($cond), $context);
        }
    };
    ($cond:expr, $context:expr, $($arg:tt)*) => {
        if !$cond {
            panic!("Assertion failed at {}: {}", stringify!($cond), format!($context, $($arg)*));
        }
    };
}

/// Macro for early return with error
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(format!($fmt, $($arg)*).into())
    };
}

/// Ensure condition is true, otherwise return error
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
    ($cond:expr, $err:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($err(&$($arg)*));
        }
    };
}
