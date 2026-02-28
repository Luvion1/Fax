//! Fax Runtime Library
//!
//! Provides runtime support for Fax programs including:
//! - GC allocation functions (via FGC)
//! - Runtime initialization

mod gc;

pub use gc::*;
