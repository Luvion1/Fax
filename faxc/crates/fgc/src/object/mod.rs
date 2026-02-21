//! Object Module - GC-managed object model
//!
//! This module defines the structure of objects managed by FGC.

pub mod header;
pub mod refmap;
pub mod weak;

pub use header::{get_data_start, get_header, get_object_addr};
pub use header::{ObjectHeader, HEADER_SIZE, OBJECT_ALIGNMENT};

pub use refmap::ReferenceMap;
pub use weak::{ReferenceQueue, WeakReference};
