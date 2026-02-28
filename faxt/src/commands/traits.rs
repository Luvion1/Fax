//! Command trait and common types for the faxt CLI.
//!
//! This module defines the standard command traits that all commands
//! must implement to ensure consistency across the application.

#![allow(dead_code)]

use crate::error::Result;

/// Standard command trait that all faxt commands must implement.
///
/// This trait ensures consistent structure and behavior across all commands.
///
/// # Type Parameters
/// * `Args` - The arguments type for this command
/// * `Output` - The output type returned by this command
pub trait Command {
    /// The arguments type for this command.
    type Args;

    /// The output type returned by this command.
    type Output;

    /// Create a new command instance with the given arguments.
    ///
    /// # Arguments
    /// * `args` - Command arguments
    ///
    /// # Returns
    /// * `Self` - A new command instance
    fn new(args: Self::Args) -> Self;

    /// Execute the command.
    ///
    /// # Returns
    /// * `Result<Self::Output>` - The command output or an error
    fn execute(&self) -> Result<Self::Output>;

    /// Get the command name.
    ///
    /// # Returns
    /// * `&'static str` - The command name
    fn name() -> &'static str;
}

/// Trait for providing command descriptions and help text.
///
/// This trait allows commands to provide human-readable descriptions
/// and detailed help information for CLI documentation.
pub trait CommandDescription {
    /// Get a short description of the command.
    ///
    /// # Returns
    /// * `&'static str` - A brief one-line description
    fn description() -> &'static str;

    /// Get detailed help text for the command.
    ///
    /// # Returns
    /// * `&'static str` - Multi-line help text explaining usage
    fn help() -> &'static str;
}

/// Trait for reporting progress during long-running operations.
///
/// Commands that perform operations taking significant time
/// can implement this trait to provide progress updates.
pub trait ProgressReporting {
    /// Report progress of an operation.
    ///
    /// # Arguments
    /// * `current` - Current progress value
    /// * `total` - Total expected value
    /// * `message` - Optional progress message
    fn report_progress(&self, current: usize, total: usize, message: Option<&str>);

    /// Check if verbose progress reporting is enabled.
    ///
    /// # Returns
    /// * `bool` - Whether verbose output is enabled
    fn is_verbose(&self) -> bool;
}

/// Common output type for commands that don't return data.
pub type NoOutput = ();

/// Command execution result with metadata.
#[derive(Debug, Clone)]
pub struct CommandResult<T = NoOutput> {
    /// Whether the command succeeded.
    pub success: bool,

    /// The command output data.
    pub data: T,

    /// Number of items processed (files, directories, etc.).
    pub items_processed: usize,

    /// Number of items failed.
    pub items_failed: usize,

    /// Execution time in milliseconds.
    pub execution_time_ms: u64,

    /// Warning messages collected during execution.
    pub warnings: Vec<String>,
}

impl<T: Default> Default for CommandResult<T> {
    fn default() -> Self {
        Self {
            success: true,
            data: T::default(),
            items_processed: 0,
            items_failed: 0,
            execution_time_ms: 0,
            warnings: Vec::new(),
        }
    }
}

impl<T: Default> CommandResult<T> {
    /// Create a new successful command result.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            items_processed: 0,
            items_failed: 0,
            execution_time_ms: 0,
            warnings: Vec::new(),
        }
    }

    /// Create a new failed command result.
    pub fn failure() -> Self {
        Self {
            success: false,
            data: T::default(),
            items_processed: 0,
            items_failed: 0,
            execution_time_ms: 0,
            warnings: Vec::new(),
        }
    }

    /// Set the number of items processed.
    pub fn with_items_processed(mut self, count: usize) -> Self {
        self.items_processed = count;
        self
    }

    /// Set the number of items failed.
    pub fn with_items_failed(mut self, count: usize) -> Self {
        self.items_failed = count;
        self
    }

    /// Set the execution time.
    pub fn with_execution_time_ms(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }

    /// Add a warning message.
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_default() {
        let result: CommandResult = CommandResult::default();
        assert!(result.success);
        assert_eq!(result.items_processed, 0);
        assert_eq!(result.items_failed, 0);
        assert_eq!(result.execution_time_ms, 0);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success(42);
        assert!(result.success);
        assert_eq!(result.data, 42);
    }

    #[test]
    fn test_command_result_failure() {
        let result: CommandResult<i32> = CommandResult::failure();
        assert!(!result.success);
    }

    #[test]
    fn test_command_result_with_methods() {
        let result = CommandResult::success(())
            .with_items_processed(10)
            .with_items_failed(2)
            .with_execution_time_ms(100)
            .with_warning("test warning".to_string());

        assert_eq!(result.items_processed, 10);
        assert_eq!(result.items_failed, 2);
        assert_eq!(result.execution_time_ms, 100);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "test warning");
    }
}
