//! Convert command implementation.
//!
//! This module provides functionality to convert files between different
//! formats with configurable quality and metadata preservation options.

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::commands::common::{error_messages, output_messages, OutputFormat};
use crate::commands::traits::{Command, CommandDescription};
use crate::config::{Config, ConvertConfig};
use crate::error::{FaxtError, Result};

/// Arguments for the convert command.
#[derive(Debug, Clone)]
pub struct ConvertArgs {
    /// Enable verbose output.
    pub verbose: bool,
    /// Input files to convert.
    pub input: Vec<PathBuf>,
    /// Output directory or file path.
    pub output: Option<PathBuf>,
    /// Output format specification.
    pub format: Option<String>,
    /// Quality setting (1-100).
    pub quality: Option<u8>,
    /// Preserve metadata during conversion.
    pub preserve_metadata: bool,
    /// Force overwrite of existing files.
    pub force: bool,
}

impl Default for ConvertArgs {
    fn default() -> Self {
        Self {
            verbose: false,
            input: Vec::new(),
            output: None,
            format: None,
            quality: None,
            preserve_metadata: true,
            force: false,
        }
    }
}

/// Convert command handler.
pub struct ConvertCommand {
    args: ConvertArgs,
    config: Config,
}

impl ConvertCommand {
    /// Create a new ConvertCommand.
    pub fn new(args: ConvertArgs) -> Self {
        Self {
            args,
            config: Config::default(),
        }
    }

    /// Execute the command.
    pub fn run(&self) -> Result<()> {
        let start_time = Instant::now();
        self.validate_input_files()?;
        let convert_config = self.get_convert_config();
        let output_format = self.determine_output_format(&convert_config)?;
        let (files_converted, files_failed) = self.process_all_files(&output_format, &convert_config)?;
        self.log_completion(start_time.elapsed(), files_converted, files_failed)?;
        self.check_for_failures(files_failed)?;
        Ok(())
    }

    /// Check for conversion failures and return error if any.
    fn check_for_failures(&self, files_failed: usize) -> Result<()> {
        if files_failed > 0 {
            return Err(FaxtError::CommandExecution(format!(
                "{} {}",
                error_messages::FILES_FAILED,
                files_failed
            )));
        }
        Ok(())
    }

    /// Validate that input files are provided.
    fn validate_input_files(&self) -> Result<()> {
        if self.args.input.is_empty() {
            return Err(FaxtError::Validation(
                error_messages::NO_INPUT_FILES.to_string(),
            ));
        }
        Ok(())
    }

    /// Process all input files and return conversion statistics.
    fn process_all_files(
        &self,
        output_format: &OutputFormat,
        config: &ConvertConfig,
    ) -> Result<(usize, usize)> {
        let mut files_converted = 0;
        let mut files_failed = 0;

        for input_path in &self.args.input {
            match self.convert_file(input_path, output_format, config) {
                Ok(_) => files_converted += 1,
                Err(e) => {
                    files_failed += 1;
                    self.log_conversion_error(input_path, &e);
                }
            }
        }

        Ok((files_converted, files_failed))
    }

    /// Log conversion error for a file.
    fn log_conversion_error(&self, input_path: &Path, error: &FaxtError) {
        eprintln!(
            "{} Failed to convert {}: {}",
            output_messages::ERROR,
            input_path.display(),
            error
        );
    }

    /// Log completion statistics if verbose.
    fn log_completion(
        &self,
        elapsed: std::time::Duration,
        files_converted: usize,
        files_failed: usize,
    ) -> Result<()> {
        if self.args.verbose {
            eprintln!(
                "{} {}",
                output_messages::CONVERSION_COMPLETED,
                elapsed.as_secs_f64()
            );
            eprintln!(
                "{} {} {}",
                output_messages::FILES_CONVERTED,
                files_converted,
                files_failed
            );
        }
        Ok(())
    }

    /// Get the effective convert configuration.
    pub fn get_convert_config(&self) -> ConvertConfig {
        let mut config = self.config.convert.clone();

        if let Some(ref format) = self.args.format {
            config.format = format.clone();
        }

        if let Some(quality) = self.args.quality {
            config.quality = quality;
        }

        config.preserve_metadata = self.args.preserve_metadata;

        config
    }

    /// Determine the output format.
    fn determine_output_format(&self, config: &ConvertConfig) -> Result<OutputFormat> {
        if let Some(ref format_str) = self.args.format {
            return OutputFormat::from_str(format_str).ok_or_else(|| {
                FaxtError::Validation(format!(
                    "{} {}",
                    error_messages::UNKNOWN_FORMAT,
                    format_str
                ))
            });
        }

        OutputFormat::from_str(&config.format).ok_or_else(|| {
            FaxtError::Config(format!(
                "{} {}",
                error_messages::INVALID_CONFIG_FORMAT,
                config.format
            ))
        })
    }

    /// Convert a single file.
    fn convert_file(
        &self,
        input_path: &Path,
        output_format: &OutputFormat,
        config: &ConvertConfig,
    ) -> Result<()> {
        self.validate_input_file(input_path)?;

        let output_path = self.determine_output_path(input_path, output_format)?;
        self.check_output_writable(&output_path)?;

        self.log_conversion_details(input_path, &output_path, output_format, config);

        self.perform_conversion(input_path, &output_path, config)?;
        self.log_conversion_success(input_path, &output_path);

        Ok(())
    }

    /// Validate that input file exists and is a file.
    fn validate_input_file(&self, input_path: &Path) -> Result<()> {
        if !input_path.exists() {
            return Err(FaxtError::Validation(format!(
                "{} {}",
                error_messages::INPUT_PATH_NOT_EXIST,
                input_path.display()
            )));
        }

        if !input_path.is_file() {
            return Err(FaxtError::Validation(format!(
                "{} {}",
                error_messages::INPUT_PATH_NOT_FILE,
                input_path.display()
            )));
        }
        Ok(())
    }

    /// Check if output file can be written.
    fn check_output_writable(&self, output_path: &Path) -> Result<()> {
        if output_path.exists() && !self.args.force {
            return Err(FaxtError::Validation(format!(
                "{} {}",
                error_messages::OUTPUT_FILE_EXISTS,
                output_path.display()
            )));
        }
        Ok(())
    }

    /// Log conversion details if verbose.
    fn log_conversion_details(
        &self,
        input_path: &Path,
        output_path: &Path,
        output_format: &OutputFormat,
        config: &ConvertConfig,
    ) {
        if !self.args.verbose {
            return;
        }

        eprintln!(
            "{} {} {}",
            output_messages::PROCESSING_FILE,
            input_path.display(),
            output_path.display()
        );
        eprintln!(
            "{} Format: {}, Quality: {}",
            output_messages::INFO,
            output_format.extension(),
            config.quality
        );
    }

    /// Log successful conversion if verbose.
    fn log_conversion_success(&self, input_path: &Path, output_path: &Path) {
        if self.args.verbose {
            eprintln!(
                "{} {} {}",
                output_messages::CONVERTED_FILE,
                input_path.display(),
                output_path.display()
            );
        }
    }

    /// Determine the output path for a converted file.
    fn determine_output_path(
        &self,
        input_path: &Path,
        output_format: &OutputFormat,
    ) -> Result<PathBuf> {
        if let Some(ref output) = self.args.output {
            if output.is_dir() || (!output.exists() && output.extension().is_none()) {
                let file_name = input_path
                    .file_stem()
                    .ok_or_else(|| {
                        FaxtError::FileOperation(error_messages::INVALID_FILE_PATH.to_string())
                    })?
                    .to_string_lossy()
                    .to_string();
                return Ok(output.join(format!("{}.{}", file_name, output_format.extension())));
            }
            return Ok(output.clone());
        }

        let output_path = input_path.with_extension(output_format.extension());
        Ok(output_path)
    }

    /// Perform the actual file conversion.
    ///
    /// TODO: Implement actual format conversion logic.
    /// Currently copies file bytes without transformation.
    ///
    /// # Arguments
    /// * `input_path` - Path to the input file
    /// * `output_path` - Path for the output file
    /// * `_config` - Convert configuration (unused for now)
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error
    ///
    /// # Supported Formats
    /// - pdf, png, jpeg, webp, tiff
    ///
    /// # Implementation Notes
    /// This is a placeholder implementation. Future implementation should:
    /// - Use appropriate conversion libraries for each format
    /// - Handle quality settings
    /// - Preserve metadata when requested
    fn perform_conversion(
        &self,
        input_path: &Path,
        output_path: &Path,
        _config: &ConvertConfig,
    ) -> Result<()> {
        // TODO: Implement actual format conversion
        // For now, just copy the file as a placeholder
        std::fs::copy(input_path, output_path)?;
        Ok(())
    }
}

impl Command for ConvertCommand {
    type Args = ConvertArgs;
    type Output = ();

    fn new(args: Self::Args) -> Self {
        Self {
            args,
            config: Config::default(),
        }
    }

    fn execute(&self) -> Result<Self::Output> {
        self.run()
    }

    fn name() -> &'static str {
        "convert"
    }
}

impl CommandDescription for ConvertCommand {
    fn description() -> &'static str {
        "Convert files between formats"
    }

    fn help() -> &'static str {
        "Converts input files to the specified output format with \
         configurable quality and metadata options."
    }
}

/// Run the convert command.
pub fn run_convert(args: ConvertArgs) -> Result<()> {
    let command = ConvertCommand::new(args);
    command.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_convert_args_default() {
        let args = ConvertArgs::default();
        assert!(args.input.is_empty());
        assert!(args.output.is_none());
        assert!(args.format.is_none());
        assert!(args.quality.is_none());
        assert!(!args.verbose);
        assert!(args.preserve_metadata);
        assert!(!args.force);
    }

    #[test]
    fn test_convert_command_new() {
        let args = ConvertArgs {
            verbose: true,
            input: vec![PathBuf::from("input.txt")],
            output: None,
            format: Some("png".to_string()),
            quality: Some(80),
            preserve_metadata: true,
            force: true,
        };
        let command = ConvertCommand::new(args.clone());

        assert_eq!(command.args.input.len(), 1);
        assert_eq!(command.args.format, Some("png".to_string()));
        assert!(command.args.verbose);
    }

    #[test]
    fn test_convert_command_name() {
        assert_eq!(<ConvertCommand as Command>::name(), "convert");
    }

    #[test]
    fn test_convert_command_description() {
        assert_eq!(
            <ConvertCommand as CommandDescription>::description(),
            "Convert files between formats"
        );
    }

    #[test]
    fn test_convert_command_get_convert_config() {
        let args = ConvertArgs {
            verbose: false,
            input: Vec::new(),
            output: None,
            format: Some("webp".to_string()),
            quality: Some(75),
            preserve_metadata: false,
            force: false,
        };
        let command = ConvertCommand::new(args);

        let convert_config = command.get_convert_config();
        assert_eq!(convert_config.format, "webp");
        assert_eq!(convert_config.quality, 75);
        assert!(!convert_config.preserve_metadata);
    }

    #[test]
    fn test_convert_command_execute_no_input() {
        let args = ConvertArgs::default();
        let command = ConvertCommand::new(args);

        let result = command.run();
        assert!(result.is_err());
        if let Err(FaxtError::Validation(msg)) = result {
            assert!(msg.contains("No input files"));
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_convert_command_execute() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.txt");
        std::fs::write(&input_file, "test content").unwrap();

        let args = ConvertArgs {
            verbose: false,
            input: vec![input_file.clone()],
            output: None,
            format: Some("pdf".to_string()),
            quality: None,
            preserve_metadata: true,
            force: true,
        };
        let command = ConvertCommand::new(args);

        let result = command.run();
        assert!(result.is_ok());

        let output_file = temp_dir.path().join("test.pdf");
        assert!(output_file.exists());
    }

    #[test]
    fn test_convert_command_execute_nonexistent_input() {
        let args = ConvertArgs {
            verbose: false,
            input: vec![PathBuf::from("/nonexistent/file.txt")],
            output: None,
            format: Some("pdf".to_string()),
            quality: None,
            preserve_metadata: true,
            force: true,
        };
        let command = ConvertCommand::new(args);

        let result = command.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_command_invalid_format() {
        let args = ConvertArgs {
            verbose: false,
            input: vec![PathBuf::from("input.txt")],
            output: None,
            format: Some("invalid_format".to_string()),
            quality: None,
            preserve_metadata: true,
            force: true,
        };
        let command = ConvertCommand::new(args);

        let result = command.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_run_convert_convenience_function() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.txt");
        std::fs::write(&input_file, "test content").unwrap();

        let args = ConvertArgs {
            verbose: false,
            input: vec![input_file],
            output: None,
            format: Some("png".to_string()),
            quality: None,
            preserve_metadata: true,
            force: true,
        };

        let result = run_convert(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_command_trait_implementation() {
        let args = ConvertArgs::default();
        let command = ConvertCommand::new(args);

        assert_eq!(ConvertCommand::name(), "convert");
    }
}
