//! Common types and utilities for faxt commands.
//!
//! This module provides shared types, constants, and utility functions
//! used across all command implementations to ensure consistency.

use std::path::{Path, PathBuf};

use crate::error::{FaxtError, Result};

// ============================================================================
// Output Format
// ============================================================================

/// Supported output formats for file conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// PDF format
    Pdf,
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
    /// WebP format
    Webp,
    /// TIFF format
    Tiff,
}

impl OutputFormat {
    /// Parse a string into an OutputFormat.
    ///
    /// # Arguments
    /// * `s` - The string to parse (case-insensitive)
    ///
    /// # Returns
    /// * `Option<OutputFormat>` - The parsed format or None if invalid
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "png" => Some(Self::Png),
            "jpeg" | "jpg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            "tiff" | "tif" => Some(Self::Tiff),
            _ => None,
        }
    }

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Tiff => "tiff",
        }
    }
}

// ============================================================================
// Path Utilities
// ============================================================================

/// Sanitize a user-provided path to prevent path traversal attacks.
///
/// Ensures the resolved path is within the current working directory
/// or an allowed base directory.
///
/// # Arguments
/// * `path` - The user-provided path to sanitize
/// * `base_dir` - The allowed base directory (defaults to current dir)
///
/// # Returns
/// * `Result<PathBuf>` - The sanitized path or an error if traversal detected
///
/// # Security
/// This function protects against:
/// - Directory traversal using `..` components
/// - Symbolic link attacks
/// - Absolute path injection
#[allow(dead_code)]
pub fn sanitize_path(path: &Path, base_dir: Option<&Path>) -> Result<PathBuf> {
    let base = base_dir.unwrap_or_else(|| Path::new("."));
    let base_canonical = base
        .canonicalize()
        .map_err(|e| FaxtError::Validation(format!("Invalid base directory: {}", e)))?;

    let path_canonical = path.canonicalize().unwrap_or_else(|_| {
        // For non-existent paths, resolve relative to base
        base.join(path).to_path_buf()
    });

    // Ensure the path is within the base directory
    if !path_canonical.starts_with(&base_canonical) {
        return Err(FaxtError::Validation(
            "Path traversal detected: path must be within current directory".to_string(),
        ));
    }

    Ok(path_canonical)
}

// ============================================================================
// Error Messages
// ============================================================================

/// Standard error message templates.
///
/// These constants provide consistent error messages across all commands.
pub mod error_messages {
    /// Error when no input files are specified.
    pub const NO_INPUT_FILES: &str = "No input files specified";

    /// Error when input path does not exist.
    pub const INPUT_PATH_NOT_EXIST: &str = "Input path does not exist: {}";

    /// Error when input path is not a file.
    pub const INPUT_PATH_NOT_FILE: &str = "Input path is not a file: {}";

    /// Error when input path is not a directory.
    pub const INPUT_PATH_NOT_DIR: &str = "Input path is not a directory: {}";

    /// Error when target path is not a directory.
    pub const TARGET_NOT_DIR: &str = "Target path is not a directory: {}";

    /// Error when directory is not empty.
    pub const DIR_NOT_EMPTY: &str = "Directory is not empty: {}";

    /// Error when output path is not a directory.
    pub const OUTPUT_PATH_NOT_DIR: &str = "Output path is not a directory: {}";

    /// Error when output file already exists.
    pub const OUTPUT_FILE_EXISTS: &str = "Output file already exists: {}";

    /// Error when an unknown format is specified.
    pub const UNKNOWN_FORMAT: &str = "Unknown format: {}";

    /// Error when config has invalid format.
    pub const INVALID_CONFIG_FORMAT: &str = "Invalid format in configuration: {}";

    /// Error when files failed to process.
    pub const FILES_FAILED: &str = "{} file(s) failed to process";

    /// Error when file path is invalid.
    pub const INVALID_FILE_PATH: &str = "Invalid file path";
}

// ============================================================================
// Output Messages
// ============================================================================

/// Standard output message templates.
///
/// These constants provide consistent output messages across all commands.
pub mod output_messages {
    /// Generic info message format.
    pub const INFO: &str = "ℹ️ {}";

    /// Generic warning message format.
    pub const WARNING: &str = "⚠️ {}";

    /// Generic error message format.
    pub const ERROR: &str = "❌ {}";

    /// Message when a directory is created.
    pub const CREATED_DIR: &str = "✅ Created directory: {}";

    /// Message when a file is created.
    pub const CREATED_FILE: &str = "✅ Created file: {}";

    /// Message when an artifact is cleaned.
    pub const CLEANED_ARTIFACT: &str = "🧹 Cleaned: {}";

    /// Message when processing a file.
    pub const PROCESSING_FILE: &str = "🔄 Processing: {} → {}";

    /// Message when a file is converted.
    pub const CONVERTED_FILE: &str = "✅ Converted: {} → {}";

    /// Message when conversion is completed.
    pub const CONVERSION_COMPLETED: &str = "✅ Conversion completed in {:.2}s";

    /// Message showing files converted count.
    pub const FILES_CONVERTED: &str = "📊 Files: {} converted, {} failed";
}
