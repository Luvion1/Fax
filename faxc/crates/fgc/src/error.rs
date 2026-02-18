//! Error Module - FGC Error Types
//!
//! Defines all error types used in FGC.
//!
//! # Error Categories
//!
//! ## Memory Errors
//! - `OutOfMemory` - Heap exhaustion
//! - `InvalidPointer` - Null or invalid pointer
//! - `BoundsCheckFailed` - Access outside valid region
//! - `AlignmentError` - Misaligned memory access
//!
//! ## Concurrency Errors
//! - `LockPoisoned` - Mutex poisoned by thread panic
//! - `ConcurrentModification` - Race condition detected
//! - `AtomicUpdateFailed` - CAS operation failed
//!
//! ## GC Phase Errors
//! - `MarkingFailed` - Mark phase error
//! - `RelocationFailed` - Relocation phase error
//! - `ForwardingTableError` - Forwarding pointer error
//! - `GcCycleFailed` - Overall GC failure
//!
//! ## Configuration Errors
//! - `Configuration` - Invalid configuration
//! - `InvalidState` - Invalid internal state
//! - `InvalidArgument` - Invalid function argument

use std::sync::PoisonError;
use thiserror::Error;

/// Main error type for all FGC operations
///
/// # Examples
///
/// ```rust
/// use fgc::error::FgcError;
///
/// // Pattern match on error variants
/// fn handle_error(err: FgcError) {
///     match err {
///         FgcError::OutOfMemory { requested, available } => {
///             eprintln!("OOM: requested {}, available {}", requested, available);
///         }
///         FgcError::LockPoisoned(msg) => {
///             eprintln!("Lock poisoned: {}", msg);
///         }
///         _ => {
///             eprintln!("Other error: {}", err);
///         }
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum FgcError {
    /// Out of memory - heap exhaustion
    ///
    /// **When returned:** Allocation request exceeds available heap space
    ///
    /// **Recovery strategy:** Trigger GC, expand heap (if possible), or fail gracefully
    ///
    /// **Example scenario:**
    /// ```ignore
    /// let result = heap.allocate(1024)?;
    /// // Returns OOM if heap is full
    /// ```
    #[error("Out of memory: requested {requested} bytes, available {available} bytes")]
    OutOfMemory { requested: usize, available: usize },

    /// Heap initialization failed
    ///
    /// **When returned:** Virtual memory reservation or region setup fails
    ///
    /// **Recovery strategy:** Cannot recover - terminate gracefully
    ///
    /// **Example scenario:** Failed to reserve virtual memory address space
    #[error("Heap initialization failed: {0}")]
    HeapInitialization(String),

    /// Invalid pointer address
    ///
    /// **When returned:** Null pointer (0x0) or obviously invalid address
    ///
    /// **Recovery strategy:** Skip the pointer, log warning
    ///
    /// **Example scenario:** GC root validation rejects null pointer
    #[error("Invalid pointer address: {address:#x}")]
    InvalidPointer { address: usize },

    /// Region allocation failed
    ///
    /// **When returned:** Failed to allocate new memory region
    ///
    /// **Recovery strategy:** Try different region type or trigger GC
    #[error("Region allocation failed: {reason}")]
    RegionAllocationFailed { reason: String },

    /// Concurrent modification detected
    ///
    /// **When returned:** Race condition detected during operation
    ///
    /// **Recovery strategy:** Retry operation with backoff
    ///
    /// **Example scenario:** Two threads modify same data structure simultaneously
    #[error("Concurrent modification detected during {operation}")]
    ConcurrentModification { operation: String },

    /// GC cycle failed
    ///
    /// **When returned:** Garbage collection encountered fatal error
    ///
    /// **Recovery strategy:** Attempt recovery GC or terminate
    #[error("GC cycle failed: {reason}")]
    GcCycleFailed { reason: String },

    /// Marking phase failed
    ///
    /// **When returned:** Concurrent marking encountered error
    ///
    /// **Recovery strategy:** Retry marking or fallback to stop-the-world
    #[error("Marking phase failed: {0}")]
    MarkingFailed(String),

    /// Relocation phase failed
    ///
    /// **When returned:** Object relocation/copying failed
    ///
    /// **Recovery strategy:** Retry relocation or abort GC cycle
    #[error("Relocation phase failed: {0}")]
    RelocationFailed(String),

    /// Forwarding table error
    ///
    /// **When returned:** Forwarding pointer setup or lookup failed
    ///
    /// **Recovery strategy:** Rebuild forwarding table
    #[error("Forwarding table error: {0}")]
    ForwardingTableError(String),

    /// TLAB (Thread-Local Allocation Buffer) error
    ///
    /// **When returned:** TLAB allocation or management failed
    ///
    /// **Recovery strategy:** Fallback to global allocation
    ///
    /// **Example scenarios:**
    /// - Invalid alignment requested
    /// - TLAB race condition (retryable)
    #[error("TLAB error: {0}")]
    TlabError(String),

    /// NUMA (Non-Uniform Memory Access) error
    ///
    /// **When returned:** NUMA-aware allocation failed
    ///
    /// **Recovery strategy:** Fallback to non-NUMA allocation
    #[error("NUMA error: {0}")]
    NumaError(String),

    /// Virtual memory error
    ///
    /// **When returned:** OS virtual memory API call failed
    ///
    /// **Recovery strategy:** Platform-specific recovery or fail
    #[error("Virtual memory error: {0}")]
    VirtualMemoryError(String),

    /// Configuration error
    ///
    /// **When returned:** Invalid GC configuration detected
    ///
    /// **Recovery strategy:** Use default configuration or fail fast
    ///
    /// **Example scenarios:**
    /// - Heap size too small
    /// - Invalid threshold values
    /// - Negative configuration values
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal error - indicates a bug in FGC
    ///
    /// **When returned:** Invariant violation or unexpected state
    ///
    /// **Recovery strategy:** Cannot recover - this is a bug
    ///
    /// **Action required:** Report to developers with full stack trace
    #[error("Internal error: {0}")]
    Internal(String),

    /// Atomic update failed
    ///
    /// **When returned:** Compare-and-swap (CAS) operation failed
    ///
    /// **Recovery strategy:** Retry with new expected value
    ///
    /// **Example scenario:** Lock-free data structure update contention
    #[error("Atomic update failed: expected value changed during CAS operation, current={0:#x}")]
    AtomicUpdateFailed(usize),

    /// Lock poisoned
    ///
    /// **When returned:** Another thread panicked while holding mutex
    ///
    /// **Recovery strategy:** 
    /// - For `lock_with_recovery()`: Log warning and recover with inner data
    /// - For `lock_strict()`: Propagate error, cannot safely continue
    ///
    /// **Example scenario:**
    /// ```ignore
    /// let result = mutex.lock().map_err(|e| FgcError::LockPoisoned(...))?;
    /// ```
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// Invalid state
    ///
    /// **When returned:** Internal state machine violation
    ///
    /// **Recovery strategy:** Cannot recover - indicates bug
    ///
    /// **Example scenario:** GC in Idle state when Marking expected
    #[error("Invalid state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    /// Invalid argument
    ///
    /// **When returned:** Function argument fails validation
    ///
    /// **Recovery strategy:** Fix caller to provide valid argument
    ///
    /// **Example scenarios:**
    /// - Null pointer passed to required parameter
    /// - Size exceeds maximum allowed
    /// - Alignment not power of 2
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Bounds check failed
    ///
    /// **When returned:** Index out of valid range
    ///
    /// **Recovery strategy:** Validate index before access
    ///
    /// **Example scenario:** Array access with index >= length
    #[error("Bounds check failed: index {index} out of bounds for length {length}")]
    BoundsCheckFailed { index: usize, length: usize },

    /// Alignment error
    ///
    /// **When returned:** Address not properly aligned for operation
    ///
    /// **Recovery strategy:** Align address or reject operation
    ///
    /// **Example scenario:** usize read from address not divisible by 8
    #[error("Alignment error: address {address:#x} is not aligned to {alignment} bytes")]
    AlignmentError { address: usize, alignment: usize },

    /// Operation timeout
    ///
    /// **When returned:** Operation exceeded time limit
    ///
    /// **Recovery strategy:** Retry with longer timeout or fail
    ///
    /// **Example scenario:** GC thread didn't respond within timeout
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Resource exhausted
    ///
    /// **When returned:** System resource depleted
    ///
    /// **Recovery strategy:** Free resources or wait for release
    ///
    /// **Example scenarios:**
    /// - Out of file descriptors
    /// - Out of thread handles
    /// - Out of virtual address space
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
