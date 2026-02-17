//! Barrier Module - Colored Pointers & Load Barriers
//!
//! Module ini mengimplementasikan colored pointers dan load barriers,
//! inovasi utama dari ZGC yang memungkinkan concurrent operations.
//!
//! Colored Pointers:
//! Metadata GC disimpan di bit pointer yang tidak terpakai (bit 44-47).
//! Ini memungkinkan GC untuk track object state tanpa mengubah object header.
//!
//! Load Barriers:
//! Intercept pointer reads untuk melakukan:
//! - Concurrent marking (mark object saat diakses)
//! - Pointer healing (update pointer ke alamat baru)
//! - Forwarding lookup (selama relocation)
//!
//! Multi-Mapping:
//! Physical memory yang sama di-map ke multiple virtual addresses:
//! - Remapped View: 0x0000_0000_0000 (normal access)
//! - Marked0 View:  0x0001_0000_0000 (GC cycle even)
//! - Marked1 View:  0x0002_0000_0000 (GC cycle odd)
//!
//! Dengan colored pointers, FGC bisa:
//! - Concurrent marking tanpa stop-the-world
//! - Concurrent relocation dengan pointer healing
//! - Self-healing pointers (update on-demand)

pub mod colored_ptr;
pub mod load_barrier;
pub mod address_space;

pub use colored_ptr::ColoredPointer;
pub use load_barrier::LoadBarrier;
pub use address_space::AddressSpace;
