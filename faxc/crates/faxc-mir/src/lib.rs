//! MIR (Mid-level Intermediate Representation) Crate
//! 
//! MIR-LIR-CODEGEN-DEV-001: Subtask 1
//! Provides MIR constructs, CFG builder, AST lowering, and optimizations.

pub mod mir;
pub mod builder;
pub mod lower;
pub mod optimize;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod edge_cases;

pub use mir::*;
pub use builder::*;
pub use lower::*;
pub use optimize::*;
