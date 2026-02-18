//! Error handling module for the faxt CLI.
//!
//! This module provides custom error types using `thiserror` for structured
//! error handling throughout the application.

use thiserror::Error;

/// Main error type for the faxt CLI application.
///
/// This enum represents all possible errors that can occur
/// during the execution of faxt commands.
#[derive(Error, Debug)]
pub enum FaxtError {
    /// Error when a required configuration is missing.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error when file operations fail.
    #[error("File operation failed: {0}")]
    FileOperation(String),

    /// Error when input validation fails.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Error when a command execution fails.
    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    /// Error when IO operations fail.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error when JSON serialization/deserialization fails.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error for any other cases.
    ///
    /// This variant is kept for backward compatibility and future extensibility.
    /// New specific error variants should be preferred over using this.
    ///
    /// # When to Use
    /// - Wrapping errors from external libraries not covered by other variants
    /// - Temporary error handling during development
    /// - Cases where a more specific variant doesn't exist yet
    #[allow(dead_code)]  // Reserved for future use and API compatibility
    #[error("{0}")]
    Other(String),
}

/// Result type alias using FaxtError.
///
/// This type alias simplifies function signatures by providing
/// a consistent result type throughout the application.
pub type Result<T> = std::result::Result<T, FaxtError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = FaxtError::Config("missing field".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing field");
    }

    #[test]
    fn test_file_operation_error_display() {
        let err = FaxtError::FileOperation("permission denied".to_string());
        assert_eq!(err.to_string(), "File operation failed: permission denied");
    }

    #[test]
    fn test_validation_error_display() {
        let err = FaxtError::Validation("invalid format".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid format");
    }

    #[test]
    fn test_command_execution_error_display() {
        let err = FaxtError::CommandExecution("exit code 1".to_string());
        assert_eq!(err.to_string(), "Command execution failed: exit code 1");
    }

    #[test]
    fn test_other_error_display() {
        let err = FaxtError::Other("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let faxt_err: FaxtError = io_err.into();
        assert!(matches!(faxt_err, FaxtError::Io(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let faxt_err: FaxtError = json_err.into();
        assert!(matches!(faxt_err, FaxtError::Json(_)));
    }
}
