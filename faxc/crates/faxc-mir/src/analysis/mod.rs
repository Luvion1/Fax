//! MIR Analysis Module
//!
//! Provides control flow and data flow analysis for MIR

pub mod cfg;
pub mod dataflow;

pub use cfg::*;
pub use dataflow::*;
