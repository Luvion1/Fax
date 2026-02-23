//! HIR to MIR Lowering Module
//!
//! Provides utilities for lowering HIR constructs to MIR

pub mod hir_to_mir;

pub use hir_to_mir::{lower_expr, lower_hir_function, lower_stmt};
