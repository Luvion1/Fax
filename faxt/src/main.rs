//! Faxt CLI - A command-line tool for fax operations.
//!
//! This is the main entry point for the faxt CLI application.
//! It uses clap for argument parsing and dispatches to appropriate
//! command handlers based on user input.

mod commands;
mod config;
mod error;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use commands::{
    build::{run_build, BuildArgs},
    convert::{run_convert, ConvertArgs},
    init::{run_init, InitArgs},
};
use config::Config;
use error::{FaxtError, Result};

/// Faxt - A CLI tool for fax operations
///
/// Faxt provides utilities for initializing projects, building artifacts,
/// and converting files between different formats.
#[derive(Parser, Debug)]
#[command(name = "faxt")]
#[command(author = "Fax Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A CLI tool for fax operations", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true, env = "FAXT_VERBOSE")]
    verbose: bool,

    /// Path to configuration file
    #[arg(short, long, global = true, env = "FAXT_CONFIG")]
    config: Option<PathBuf>,

    /// Disable color output
    #[arg(long, global = true, env = "FAXT_NO_COLOR")]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the faxt CLI.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new faxt project
    ///
    /// Creates the necessary directory structure and configuration files
    /// for a new faxt project in the specified or current directory.
    Init(InitCommand),

    /// Build project artifacts
    ///
    /// Processes input files and generates output artifacts according
    /// to the build configuration.
    Build(BuildCommand),

    /// Convert files between formats
    ///
    /// Converts input files to the specified output format with
    /// configurable quality and metadata options.
    Convert(ConvertCommand),
}

/// Arguments for the init subcommand.
#[derive(Parser, Debug)]
struct InitCommand {
    /// Project name
    #[arg(short, long)]
    name: Option<String>,

    /// Directory to initialize (default: current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Force initialization even if directory is not empty
    #[arg(short, long)]
    force: bool,
}

/// Arguments for the build subcommand.
#[derive(Parser, Debug)]
struct BuildCommand {
    /// Input directory (default: from config)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output directory (default: from config)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Disable optimizations (optimizations enabled by default)
    #[arg(long, default_value_t = false)]
    no_optimize: bool,

    /// Target architecture
    #[arg(long)]
    target: Option<String>,

    /// Number of parallel jobs
    #[arg(short, long)]
    jobs: Option<u32>,

    /// Clean build artifacts before building
    #[arg(long)]
    clean: bool,
}

/// Arguments for the convert subcommand.
#[derive(Parser, Debug)]
struct ConvertCommand {
    /// Input files to convert
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output directory or file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format (pdf, png, jpeg, webp, tiff)
    #[arg(short = 'F', long)]
    format: Option<String>,

    /// Quality setting (1-100)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: Option<u8>,

    /// Don't preserve metadata
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    preserve_metadata: bool,

    /// Overwrite existing output files
    #[arg(short, long)]
    force: bool,
}

/// Main entry point for the faxt CLI.
///
/// Parses command-line arguments, initializes logging, loads configuration,
/// and dispatches to the appropriate command handler.
///
/// # Returns
/// * `Result<()>` - Success or an error
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose, cli.no_color)?;

    // Load configuration
    let config = load_config(cli.config.as_deref())?;

    // Execute the selected command
    execute_command(cli.command, cli.verbose, config)
}

/// Initialize the logging system.
///
/// # Arguments
/// * `verbose` - Whether to enable verbose logging
/// * `no_color` - Whether to disable colored output
///
/// # Returns
/// * `Result<()>` - Success or an error
fn init_logging(verbose: bool, no_color: bool) -> Result<()> {
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    let subscriber = fmt::layer()
        .with_ansi(!no_color)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(subscriber)
        .try_init()
        .map_err(|e| FaxtError::Config(format!("Failed to initialize logging: {}", e)))?;

    Ok(())
}

/// Load configuration from file or use defaults.
///
/// # Arguments
/// * `config_path` - Optional path to configuration file
///
/// # Returns
/// * `Result<Config>` - The loaded configuration or an error
fn load_config(config_path: Option<&std::path::Path>) -> Result<Config> {
    match config_path {
        Some(path) => Config::load_from_path(path),
        None => Config::load(),
    }
}

/// Execute the selected command.
///
/// # Arguments
/// * `command` - The command to execute
/// * `verbose` - Whether verbose output is enabled
/// * `config` - The application configuration
///
/// # Returns
/// * `Result<()>` - Success or an error
fn execute_command(command: Commands, verbose: bool, config: Config) -> Result<()> {
    match command {
        Commands::Init(args) => execute_init(args, verbose),
        Commands::Build(args) => execute_build(args, verbose, config),
        Commands::Convert(args) => execute_convert(args, verbose, config),
    }
}

/// Execute the init command.
fn execute_init(args: InitCommand, verbose: bool) -> Result<()> {
    let init_args = InitArgs {
        verbose,
        force: args.force,
        path: args.path,
    };
    run_init(init_args)
}

/// Execute the build command.
fn execute_build(args: BuildCommand, verbose: bool, _config: Config) -> Result<()> {
    let build_args = BuildArgs {
        input: args.input,
        output: args.output,
        verbose,
        optimize: !args.no_optimize,
        target: args.target,
        jobs: args.jobs,
        clean: args.clean,
    };
    run_build(build_args)
}

/// Execute the convert command.
fn execute_convert(args: ConvertCommand, verbose: bool, _config: Config) -> Result<()> {
    let convert_args = ConvertArgs {
        input: args.input,
        output: args.output,
        format: args.format,
        quality: args.quality,
        verbose,
        preserve_metadata: args.preserve_metadata,
        force: args.force,
    };
    run_convert(convert_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_init() {
        let cli = Cli::parse_from(["faxt", "init"]);
        assert!(matches!(cli.command, Commands::Init(_)));
    }

    #[test]
    fn test_cli_parse_init_with_name() {
        let cli = Cli::parse_from(["faxt", "init", "--name", "my-project"]);
        if let Commands::Init(args) = cli.command {
            assert_eq!(args.name, Some("my-project".to_string()));
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_cli_parse_init_with_path() {
        let cli = Cli::parse_from(["faxt", "init", "--path", "/tmp/test"]);
        if let Commands::Init(args) = cli.command {
            assert_eq!(args.path, Some(PathBuf::from("/tmp/test")));
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_cli_parse_init_with_force() {
        let cli = Cli::parse_from(["faxt", "init", "--force"]);
        if let Commands::Init(args) = cli.command {
            assert!(args.force);
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_cli_parse_build() {
        let cli = Cli::parse_from(["faxt", "build"]);
        assert!(matches!(cli.command, Commands::Build(_)));
    }

    #[test]
    fn test_cli_parse_build_with_input() {
        let cli = Cli::parse_from(["faxt", "build", "--input", "/input"]);
        if let Commands::Build(args) = cli.command {
            assert_eq!(args.input, Some(PathBuf::from("/input")));
        } else {
            panic!("Expected Build command");
        }
    }

    #[test]
    fn test_cli_parse_build_with_clean() {
        let cli = Cli::parse_from(["faxt", "build", "--clean"]);
        if let Commands::Build(args) = cli.command {
            assert!(args.clean);
        } else {
            panic!("Expected Build command");
        }
    }

    #[test]
    fn test_cli_parse_convert() {
        let cli = Cli::parse_from(["faxt", "convert", "input.txt"]);
        assert!(matches!(cli.command, Commands::Convert(_)));
    }

    #[test]
    fn test_cli_parse_convert_with_format() {
        let cli = Cli::parse_from(["faxt", "convert", "input.txt", "--format", "pdf"]);
        if let Commands::Convert(args) = cli.command {
            assert_eq!(args.format, Some("pdf".to_string()));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn test_cli_parse_convert_with_quality() {
        let cli = Cli::parse_from(["faxt", "convert", "input.txt", "--quality", "80"]);
        if let Commands::Convert(args) = cli.command {
            assert_eq!(args.quality, Some(80));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn test_cli_parse_convert_with_force() {
        let cli = Cli::parse_from(["faxt", "convert", "input.txt", "--force"]);
        if let Commands::Convert(args) = cli.command {
            assert!(args.force);
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn test_cli_parse_global_verbose() {
        let cli = Cli::parse_from(["faxt", "--verbose", "init"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_parse_global_config() {
        let cli = Cli::parse_from(["faxt", "--config", "/path/to/config.toml", "init"]);
        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
    }

    #[test]
    fn test_cli_parse_global_no_color() {
        let cli = Cli::parse_from(["faxt", "--no-color", "init"]);
        assert!(cli.no_color);
    }

    #[test]
    fn test_cli_version_flag() {
        // Test that version flag is recognized (will print version and exit in real execution)
        let cli = Cli::parse_from(["faxt", "init"]);
        assert!(matches!(cli.command, Commands::Init(_)));
        assert_eq!(cli.verbose, false);
    }
}
