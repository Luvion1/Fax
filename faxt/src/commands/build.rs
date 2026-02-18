//! Build command implementation.
//!
//! This module provides functionality to process input files and generate
//! output artifacts according to the build configuration.

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::commands::common::error_messages;
use crate::commands::common::output_messages;
use crate::commands::traits::{Command, CommandDescription};
use crate::config::{BuildConfig, Config};
use crate::error::{FaxtError, Result};

/// Arguments for the build command.
#[derive(Debug, Clone)]
pub struct BuildArgs {
    /// Enable verbose output.
    pub verbose: bool,
    /// Input directory path.
    pub input: Option<PathBuf>,
    /// Output directory path.
    pub output: Option<PathBuf>,
    /// Enable optimizations.
    pub optimize: bool,
    /// Target architecture.
    pub target: Option<String>,
    /// Number of parallel jobs.
    pub jobs: Option<u32>,
    /// Clean build artifacts before building.
    pub clean: bool,
}

impl Default for BuildArgs {
    fn default() -> Self {
        Self {
            verbose: false,
            input: None,
            output: None,
            optimize: true,
            target: None,
            jobs: None,
            clean: false,
        }
    }
}

/// Build command handler.
pub struct BuildCommand {
    args: BuildArgs,
    config: Config,
}

impl BuildCommand {
    /// Create a new BuildCommand.
    pub fn new(args: BuildArgs) -> Self {
        Self {
            args,
            config: Config::default(),
        }
    }

    /// Execute the command.
    pub fn run(&self) -> Result<()> {
        let start_time = Instant::now();
        let input_path = self.get_input_path()?;
        let output_path = self.get_output_path()?;
        self.validate_paths(&input_path, &output_path)?;
        self.maybe_clean_build(&output_path)?;
        self.ensure_output_dir(&output_path)?;
        let files_processed = self.process_input_files(&input_path, &output_path)?;
        self.log_build_complete(start_time.elapsed(), files_processed)?;
        Ok(())
    }

    /// Validate input and output paths.
    fn validate_paths(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.validate_input_path(input_path)?;
        self.validate_output_path(output_path)?;
        Ok(())
    }

    /// Handle clean operation if requested.
    fn maybe_clean_build(&self, output_path: &Path) -> Result<()> {
        if self.args.clean {
            self.clean_build_artifacts(output_path)?;
        }
        Ok(())
    }

    /// Process input files and return count of files processed.
    fn process_input_files(&self, input_path: &Path, output_path: &Path) -> Result<usize> {
        self.process_files(input_path, output_path)
    }

    /// Log build completion if verbose.
    fn log_build_complete(&self, elapsed: std::time::Duration, files: usize) -> Result<()> {
        if self.args.verbose {
            eprintln!("✅ Build completed in {:.2}s", elapsed.as_secs_f64());
            eprintln!("ℹ️ Processed {} file(s)", files);
        }
        Ok(())
    }

    /// Get the effective build configuration.
    pub fn get_build_config(&self) -> BuildConfig {
        let mut config = self.config.build.clone();

        config.optimize = self.args.optimize;

        if let Some(ref target) = self.args.target {
            config.target = Some(target.clone());
        }

        if let Some(jobs) = self.args.jobs {
            config.jobs = jobs;
        }

        config
    }

    /// Get the input path.
    fn get_input_path(&self) -> Result<PathBuf> {
        match &self.args.input {
            Some(path) => Ok(path.clone()),
            None => Ok(PathBuf::from(&self.config.input_dir)),
        }
    }

    /// Get the output path.
    fn get_output_path(&self) -> Result<PathBuf> {
        match &self.args.output {
            Some(path) => Ok(path.clone()),
            None => Ok(PathBuf::from(&self.config.output_dir)),
        }
    }

    /// Validate input path exists and is a directory.
    fn validate_input_path(&self, input_path: &Path) -> Result<()> {
        if !input_path.exists() {
            return Err(FaxtError::Validation(format!(
                "{} {}",
                error_messages::INPUT_PATH_NOT_EXIST,
                input_path.display()
            )));
        }

        if !input_path.is_dir() {
            return Err(FaxtError::Validation(format!(
                "{} {}",
                error_messages::INPUT_PATH_NOT_DIR,
                input_path.display()
            )));
        }

        Ok(())
    }

    /// Validate output path is a directory if it exists.
    fn validate_output_path(&self, output_path: &Path) -> Result<()> {
        if output_path.exists() && !output_path.is_dir() {
            return Err(FaxtError::Validation(format!(
                "{} {}",
                error_messages::OUTPUT_PATH_NOT_DIR,
                output_path.display()
            )));
        }

        Ok(())
    }

    /// Clean build artifacts from the output directory.
    fn clean_build_artifacts(&self, output_path: &Path) -> Result<()> {
        if !output_path.exists() {
            self.log_clean_skip();
            return Ok(());
        }

        self.log_clean_start(output_path);
        self.clean_directory_contents(output_path)?;

        Ok(())
    }

    /// Log message when skipping clean due to missing directory.
    fn log_clean_skip(&self) {
        if self.args.verbose {
            eprintln!(
                "{} Output directory does not exist, nothing to clean",
                output_messages::WARNING
            );
        }
    }

    /// Log start of cleaning operation.
    fn log_clean_start(&self, output_path: &Path) {
        if self.args.verbose {
            eprintln!(
                "{} Cleaning build artifacts from {}",
                output_messages::INFO,
                output_path.display()
            );
        }
    }

    /// Clean all contents within a directory.
    fn clean_directory_contents(&self, output_path: &Path) -> Result<()> {
        for entry in std::fs::read_dir(output_path)? {
            let entry = entry?;
            let path = entry.path();

            Self::remove_path(&path)?;

            if self.args.verbose {
                eprintln!(
                    "{} {}",
                    output_messages::CLEANED_ARTIFACT,
                    path.display()
                );
            }
        }
        Ok(())
    }

    /// Remove a file or directory at the given path.
    fn remove_path(path: &Path) -> Result<()> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Ensure the output directory exists.
    fn ensure_output_dir(&self, output_path: &Path) -> Result<()> {
        if !output_path.exists() {
            std::fs::create_dir_all(output_path)?;
            if self.args.verbose {
                eprintln!(
                    "{} {}",
                    output_messages::CREATED_DIR,
                    output_path.display()
                );
            }
        }
        Ok(())
    }

    /// Process files from input directory to output directory.
    fn process_files(&self, input_path: &Path, output_path: &Path) -> Result<usize> {
        let build_config = self.get_build_config();
        self.log_build_config(&build_config, input_path);

        let mut files_processed = 0;
        for entry in std::fs::read_dir(input_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                self.process_single_file(&path, output_path)?;
                files_processed += 1;
            }
        }

        Ok(files_processed)
    }

    /// Log build configuration details if verbose.
    fn log_build_config(&self, config: &BuildConfig, input_path: &Path) {
        if !self.args.verbose {
            return;
        }

        eprintln!(
            "{} Processing files from {}",
            output_messages::INFO,
            input_path.display()
        );
        eprintln!(
            "{} Build config: optimize={}, jobs={}",
            output_messages::INFO,
            config.optimize, config.jobs
        );
        if let Some(ref target) = config.target {
            eprintln!(
                "{} Target: {}",
                output_messages::INFO,
                target
            );
        }
    }

    /// Process a single file.
    fn process_single_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let file_name = input_path
            .file_name()
            .ok_or_else(|| {
                FaxtError::FileOperation(error_messages::INVALID_FILE_PATH.to_string())
            })?
            .to_string_lossy()
            .to_string();

        let output_file = output_path.join(&file_name);

        if self.args.verbose {
            eprintln!(
                "{} {} {}",
                output_messages::PROCESSING_FILE,
                input_path.display(),
                output_file.display()
            );
        }

        std::fs::copy(input_path, &output_file)?;

        Ok(())
    }
}

impl Command for BuildCommand {
    type Args = BuildArgs;
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
        "build"
    }
}

impl CommandDescription for BuildCommand {
    fn description() -> &'static str {
        "Build project artifacts"
    }

    fn help() -> &'static str {
        "Processes input files and generates output artifacts according \
         to the build configuration."
    }
}

/// Run the build command.
pub fn run_build(args: BuildArgs) -> Result<()> {
    let command = BuildCommand::new(args);
    command.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dirs() -> (TempDir, TempDir) {
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        std::fs::write(input_dir.path().join("test1.txt"), "content1").unwrap();
        std::fs::write(input_dir.path().join("test2.txt"), "content2").unwrap();

        (input_dir, output_dir)
    }

    #[test]
    fn test_build_args_default() {
        let args = BuildArgs::default();
        assert!(!args.verbose);
        assert!(args.input.is_none());
        assert!(args.output.is_none());
        assert!(args.optimize);
        assert!(!args.clean);
    }

    #[test]
    fn test_build_command_new() {
        let args = BuildArgs {
            verbose: true,
            input: Some(PathBuf::from("/input")),
            output: Some(PathBuf::from("/output")),
            optimize: false,
            target: Some("x86_64".to_string()),
            jobs: Some(4),
            clean: true,
        };
        let command = BuildCommand::new(args.clone());

        assert!(command.args.verbose);
        assert!(!command.args.optimize);
    }

    #[test]
    fn test_build_command_name() {
        assert_eq!(<BuildCommand as Command>::name(), "build");
    }

    #[test]
    fn test_build_command_description() {
        assert_eq!(
            <BuildCommand as CommandDescription>::description(),
            "Build project artifacts"
        );
    }

    #[test]
    fn test_build_command_get_build_config() {
        let args = BuildArgs {
            verbose: false,
            optimize: false,
            target: Some("arm64".to_string()),
            jobs: Some(8),
            input: None,
            output: None,
            clean: false,
        };
        let command = BuildCommand::new(args);

        let build_config = command.get_build_config();
        assert!(!build_config.optimize);
        assert_eq!(build_config.target, Some("arm64".to_string()));
        assert_eq!(build_config.jobs, 8);
    }

    #[test]
    fn test_build_command_execute() {
        let (input_dir, output_dir) = setup_test_dirs();

        let args = BuildArgs {
            verbose: false,
            input: Some(input_dir.path().to_path_buf()),
            output: Some(output_dir.path().to_path_buf()),
            optimize: true,
            target: None,
            jobs: None,
            clean: false,
        };
        let command = BuildCommand::new(args);

        let result = command.run();
        assert!(result.is_ok());

        assert!(output_dir.path().join("test1.txt").exists());
        assert!(output_dir.path().join("test2.txt").exists());
    }

    #[test]
    fn test_build_command_execute_nonexistent_input() {
        let temp_dir = TempDir::new().unwrap();

        let args = BuildArgs {
            verbose: false,
            input: Some(PathBuf::from("/nonexistent/path")),
            output: Some(temp_dir.path().to_path_buf()),
            optimize: true,
            target: None,
            jobs: None,
            clean: false,
        };
        let command = BuildCommand::new(args);

        let result = command.run();
        assert!(result.is_err());
        if let Err(FaxtError::Validation(msg)) = result {
            assert!(msg.contains("does not exist"));
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_build_command_execute_clean() {
        let (input_dir, output_dir) = setup_test_dirs();

        std::fs::write(output_dir.path().join("old.txt"), "old content").unwrap();

        let args = BuildArgs {
            verbose: false,
            input: Some(input_dir.path().to_path_buf()),
            output: Some(output_dir.path().to_path_buf()),
            optimize: true,
            target: None,
            jobs: None,
            clean: true,
        };
        let command = BuildCommand::new(args);

        let result = command.run();
        assert!(result.is_ok());

        assert!(!output_dir.path().join("old.txt").exists());
        assert!(output_dir.path().join("test1.txt").exists());
    }

    #[test]
    fn test_run_build_convenience_function() {
        let (input_dir, output_dir) = setup_test_dirs();

        let args = BuildArgs {
            verbose: false,
            input: Some(input_dir.path().to_path_buf()),
            output: Some(output_dir.path().to_path_buf()),
            optimize: true,
            target: None,
            jobs: None,
            clean: false,
        };

        let result = run_build(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_command_trait_implementation() {
        let args = BuildArgs::default();
        let command = BuildCommand::new(args);

        assert_eq!(BuildCommand::name(), "build");
    }
}
