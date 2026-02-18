//! Command modules for the faxt CLI.
//!
//! This module contains implementations for all available subcommands.
//! Each subcommand is implemented in its own file following a standardized pattern.

pub mod traits;
pub mod common;

pub mod init;
pub mod build;
pub mod convert;

// Re-export command types and functions
pub use init::{InitArgs, run_init};
pub use build::{BuildArgs, run_build};
pub use convert::{ConvertArgs, run_convert};
