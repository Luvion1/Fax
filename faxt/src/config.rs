//! Configuration module for the faxt CLI.
//!
//! This module handles loading, saving, and managing configuration
//! settings for the faxt application.

use dirs::{config_dir, home_dir};
use num_cpus::get as get_num_cpus;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{FaxtError, Result};

/// Default configuration file name.
pub const CONFIG_FILE_NAME: &str = "faxt.toml";

/// Default number of parallel threads for builds.
/// Chosen as a sensible default that works well on most systems
/// without overwhelming CPU resources on machines with many cores.
const DEFAULT_THREAD_COUNT: u32 = 4;

/// Application configuration structure.
///
/// This struct represents the complete configuration for the faxt CLI,
/// including global settings and command-specific options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Global verbose setting.
    #[serde(default)]
    pub verbose: bool,

    /// Default output directory.
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Default input directory.
    #[serde(default = "default_input_dir")]
    pub input_dir: String,

    /// Build-specific configuration.
    #[serde(default)]
    pub build: BuildConfig,

    /// Convert-specific configuration.
    #[serde(default)]
    pub convert: ConvertConfig,
}

/// Build-specific configuration options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildConfig {
    /// Enable optimizations in build.
    #[serde(default = "default_true")]
    pub optimize: bool,

    /// Target architecture for builds.
    #[serde(default)]
    pub target: Option<String>,

    /// Number of parallel jobs.
    #[serde(default = "default_parallel_jobs")]
    pub jobs: u32,
}

/// Convert-specific configuration options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConvertConfig {
    /// Default output format for conversions.
    #[serde(default = "default_format")]
    pub format: String,

    /// Quality setting for conversions (1-100).
    #[serde(default = "default_quality")]
    pub quality: u8,

    /// Whether to preserve metadata.
    #[serde(default = "default_true")]
    pub preserve_metadata: bool,
}

/// Default value functions for configuration fields.
fn default_output_dir() -> String {
    "output".to_string()
}

fn default_input_dir() -> String {
    "input".to_string()
}

fn default_true() -> bool {
    true
}

/// Get the default number of parallel jobs based on CPU count.
///
/// # Returns
/// The number of available CPUs as u32, or DEFAULT_THREAD_COUNT (4) as fallback.
///
/// # Notes
/// `num_cpus::get()` returns usize, which is always >= 1 and fits in u32
/// on all supported platforms (x86_64, aarch64). The conversion cannot fail
/// in practice, but we use try_into() for correctness and provide a safe
/// fallback for extremely rare edge cases.
fn default_parallel_jobs() -> u32 {
    // num_cpus::get() returns usize, which is always >= 1 and fits in u32
    // on all supported platforms. The conversion cannot fail in practice.
    get_num_cpus().try_into().unwrap_or(DEFAULT_THREAD_COUNT)
}

fn default_format() -> String {
    "pdf".to_string()
}

fn default_quality() -> u8 {
    90
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: false,
            output_dir: default_output_dir(),
            input_dir: default_input_dir(),
            build: BuildConfig::default(),
            convert: ConvertConfig::default(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            optimize: true,
            target: None,
            jobs: default_parallel_jobs(),
        }
    }
}

impl Default for ConvertConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            quality: default_quality(),
            preserve_metadata: true,
        }
    }
}

impl Config {
    /// Load configuration from the default location.
    ///
    /// Searches for configuration in the following order:
    /// 1. Current directory
    /// 2. User's home directory
    /// 3. System configuration directory
    ///
    /// Returns the default configuration if no config file is found.
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_file()?;

        match config_path {
            Some(path) => Self::load_from_path(&path),
            None => Ok(Self::default()),
        }
    }

    /// Load configuration from a specific path.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<Config>` - The loaded configuration or an error
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(FaxtError::Config(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            FaxtError::Config(format!("Failed to parse configuration: {}", e))
        })?;

        Ok(config)
    }

    /// Save configuration to a specific path.
    ///
    /// # Arguments
    /// * `path` - Path where the configuration should be saved
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            FaxtError::Config(format!("Failed to serialize configuration: {}", e))
        })?;

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check for config in current directory.
    fn check_current_dir_config() -> Option<PathBuf> {
        let path = PathBuf::from(CONFIG_FILE_NAME);
        path.exists().then_some(path)
    }

    /// Check for config in home directory.
    fn check_home_config() -> Option<PathBuf> {
        home_dir()
            .map(|dir| dir.join(".config").join("faxt").join(CONFIG_FILE_NAME))
            .filter(|path| path.exists())
    }

    /// Check for config in system config directory.
    fn check_system_config() -> Option<PathBuf> {
        config_dir()
            .map(|dir| dir.join("faxt").join(CONFIG_FILE_NAME))
            .filter(|path| path.exists())
    }

    /// Find the configuration file in standard locations.
    ///
    /// # Returns
    /// * `Result<Option<PathBuf>>` - Path to config file if found, None otherwise
    fn find_config_file() -> Result<Option<PathBuf>> {
        Ok(Self::check_current_dir_config()
            .or_else(Self::check_home_config)
            .or_else(Self::check_system_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            verbose: true,
            output_dir: "/tmp/output".to_string(),
            input_dir: "/tmp/input".to_string(),
            build: BuildConfig {
                optimize: false,
                target: Some("x86_64-unknown-linux-gnu".to_string()),
                jobs: 2,
            },
            convert: ConvertConfig {
                format: "png".to_string(),
                quality: 85,
                preserve_metadata: false,
            },
        }
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.verbose);
        assert_eq!(config.output_dir, "output");
        assert_eq!(config.input_dir, "input");
        assert!(config.build.optimize);
        assert_eq!(config.convert.format, "pdf");
        assert_eq!(config.convert.quality, 90);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let original_config = create_test_config();
        original_config.save_to_path(&config_path).unwrap();

        let loaded_config = Config::load_from_path(&config_path).unwrap();

        assert_eq!(original_config, loaded_config);
    }

    #[test]
    fn test_load_from_nonexistent_path() {
        let result = Config::load_from_path(Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
    }
}
