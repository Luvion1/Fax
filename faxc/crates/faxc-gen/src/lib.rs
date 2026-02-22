//! CodeGen - LLVM IR Code Generation for Fax Compiler
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 3
//! Complete LLVM IR generation with:
//! - Type mapping (SPEC.md 12.1)
//! - Instruction lowering
//! - System V AMD64 ABI support
//! - Control flow constructs
//! - Aggregate types

pub mod llvm;
pub mod asm;
pub mod linker;
pub mod types;
pub mod error;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod edge_cases;

pub use llvm::*;
pub use asm::*;
pub use linker::*;
pub use types::*;
pub use error::{CodeGenError, Result};
