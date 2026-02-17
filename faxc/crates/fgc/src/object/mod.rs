//! Object Module - GC-managed object model
//!
//! This module defines the structure of objects managed by FGC.

pub mod header;
pub mod refmap;

pub use header::{ObjectHeader, HEADER_SIZE, OBJECT_ALIGNMENT};
pub use header::{get_header, get_data_start, get_object_addr};

pub use refmap::ReferenceMap;
