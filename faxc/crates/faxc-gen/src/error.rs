//! Error types for LLVM code generation
//!
//! This module defines error types for the faxc-gen crate, providing
//! proper error handling instead of panics.

use thiserror::Error;

/// Error type for LLVM code generation
#[derive(Debug, Error)]
pub enum CodeGenError {
    /// Target block not found during code generation
    #[error("Target block '{0}' not found")]
    BlockNotFound(String),

    /// Missing comparison before conditional jump
    #[error("No comparison before conditional jump")]
    MissingComparison,

    /// LLVM operation failed
    #[error("LLVM operation failed: {0}")]
    LlvmOperationFailed(String),

    /// Function not found
    #[error("Function '{0}' not found")]
    FunctionNotFound(String),

    /// Invalid operand type
    #[error("Invalid operand type: {0}")]
    InvalidOperandType(String),

    /// Type mapping error
    #[error("Type mapping error: {0}")]
    TypeMappingError(String),

    /// Register allocation failed
    #[error("Register allocation failed: {0}")]
    RegisterAllocationFailed(String),

    /// Stack frame error
    #[error("Stack frame error: {0}")]
    StackFrameError(String),

    /// ABI error
    #[error("ABI error: {0}")]
    AbiError(String),

    /// Compilation error (target, linking, etc.)
    #[error("Compilation error: {0}")]
    CompilationError(String),

    /// Internal error - indicates a bug
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for code generation operations
pub type Result<T> = std::result::Result<T, CodeGenError>;
