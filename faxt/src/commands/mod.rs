//! Command modules for the faxt CLI.
//!
//! This module contains implementations for all available subcommands.
//! Each subcommand is implemented in its own file following a standardized pattern.

pub mod common;
pub mod traits;

pub mod build;
pub mod convert;
pub mod init;

// Re-export command types and functions (used by main.rs)
#[allow(unused_imports)]
pub use build::{run_build, BuildArgs};
#[allow(unused_imports)]
pub use convert::{run_convert, ConvertArgs};
#[allow(unused_imports)]
pub use init::{run_init, InitArgs};
