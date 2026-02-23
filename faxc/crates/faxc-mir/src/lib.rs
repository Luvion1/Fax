//! MIR (Mid-level Intermediate Representation) Crate
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 1
//! Provides MIR constructs, CFG builder, AST lowering, and optimizations.

pub mod mir;
pub mod build;
pub mod lower;
pub mod opt;
pub mod analysis;

pub use mir::*;
pub use build::*;
pub use lower::*;
pub use opt::*;
pub use analysis::*;
