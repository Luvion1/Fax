//! Barrier Module - Colored Pointers & Load Barriers
//!
//! This module implements colored pointers and load barriers,
//! the main innovation of ZGC that enables concurrent operations.
//!
//! Colored Pointers:
//! GC metadata is stored in unused pointer bits (bits 44-47).
//! This allows GC to track object state without modifying object headers.
//!
//! Load Barriers:
//! Intercept pointer reads to perform:
//! - Concurrent marking (mark object when accessed)
//! - Pointer healing (update pointer to new address)
//! - Forwarding lookup (during relocation)
//!
//! Multi-Mapping:
//! Same physical memory is mapped to multiple virtual addresses:
//! - Remapped View: 0x0000_0000_0000 (normal access)
//! - Marked0 View:  0x0001_0000_0000 (GC cycle even)
//! - Marked1 View:  0x0002_0000_0000 (GC cycle odd)
//!
//! With colored pointers, FGC can:
//! - Concurrent marking without stop-the-world
//! - Concurrent relocation with pointer healing
//! - Self-healing pointers (update on-demand)

pub mod address_space;
pub mod colored_ptr;
pub mod fast_path;
pub mod load_barrier;
pub mod read_barrier;
pub mod stats;
pub use read_barrier::write_barrier;

pub use address_space::AddressSpace;
pub use colored_ptr::ColoredPointer;
pub use load_barrier::LoadBarrier;
pub use load_barrier::heal_pointer;
pub use load_barrier::heal_pointer_global;
pub use load_barrier::on_object_read;
