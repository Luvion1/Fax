//! CodeGen - LLVM IR Code Generation for Fax Compiler
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 3
//! Complete LLVM IR generation with:
//! - Type mapping (SPEC.md 12.1)
//! - Instruction lowering
//! - System V AMD64 ABI support
//! - Control flow constructs
//! - Aggregate types

pub mod asm;
pub mod error;
pub mod linker;
pub mod llvm;
pub mod types;
// #[cfg(test)]
// mod tests;
// #[cfg(test)]
// mod edge_cases;

pub use asm::*;
pub use error::{CodeGenError, Result};
pub use linker::*;
pub use linker::*;
pub use llvm::*;
pub use llvm::*;
pub use types::*;
pub use types::*;
