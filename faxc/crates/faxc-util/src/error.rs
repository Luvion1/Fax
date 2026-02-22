//! Core error types for faxc-util crate
//!
//! This module defines error types used throughout the util crate.

use thiserror::Error;

/// Error type for symbol interning operations
#[derive(Debug, Error)]
pub enum SymbolError {
    /// Failed to intern a symbol
    #[error("Failed to intern symbol: {0}")]
    InternFailed(String),

    /// Symbol not found in the interner
    #[error("Symbol not found: index {index}")]
    NotFound { index: u32 },
}

/// Error type for source map operations
#[derive(Debug, Error)]
pub enum SourceMapError {
    /// File not found in the source map
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid span range
    #[error("Invalid span: start {start} > end {end}")]
    InvalidSpan { start: usize, end: usize },

    /// Span out of bounds for file
    #[error("Span out of bounds: file has {file_len} bytes, span is {span_start}..{span_end}")]
    SpanOutOfBounds {
        file_len: usize,
        span_start: usize,
        span_end: usize,
    },

    /// Invalid line number
    #[error("Invalid line number: {line} (file has {max_lines} lines)")]
    InvalidLineNumber { line: usize, max_lines: usize },

    /// Failed to extract source snippet
    #[error("Failed to extract source: {0}")]
    ExtractFailed(String),
}

/// Error type for index vector operations
#[derive(Debug, Error)]
pub enum IndexVecError {
    /// Index out of bounds
    #[error("Index out of bounds: index {index}, length {length}")]
    OutOfBounds { index: usize, length: usize },

    /// Invalid index
    #[error("Invalid index: {0}")]
    InvalidIndex(String),
}

/// Error type for diagnostic operations
#[derive(Debug, Error)]
pub enum DiagnosticError {
    /// Failed to format diagnostic
    #[error("Failed to format diagnostic: {0}")]
    FormatFailed(String),

    /// Invalid diagnostic code
    #[error("Invalid diagnostic code: {0}")]
    InvalidCode(String),
}

/// Result type alias for symbol operations
pub type SymbolResult<T> = std::result::Result<T, SymbolError>;

/// Result type alias for source map operations
pub type SourceMapResult<T> = std::result::Result<T, SourceMapError>;

/// Result type alias for index vector operations
pub type IndexVecResult<T> = std::result::Result<T, IndexVecError>;

/// Result type alias for diagnostic operations
pub type DiagnosticResult<T> = std::result::Result<T, DiagnosticError>;
