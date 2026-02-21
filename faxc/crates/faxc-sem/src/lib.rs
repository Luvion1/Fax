pub mod types;
pub mod hir;
pub mod scope;
pub mod analysis;
#[cfg(test)]
mod edge_cases;

// Re-export common types
pub use types::*;
pub use hir::*;
pub use scope::*;
pub use analysis::*;
