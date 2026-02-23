//! LIR (Low-level Intermediate Representation) Crate
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 2
//! Provides LIR constructs with x86-64 instruction set,
//! virtual register management, and System V AMD64 ABI support.

pub mod calling_convention;
pub mod lir;
pub mod lower;
pub mod stack_frame;
#[cfg(test)]
mod tests;
// #[cfg(test)]
// mod edge_cases;

pub use calling_convention::*;
pub use lir::*;
pub use lower::*;
pub use stack_frame::*;
