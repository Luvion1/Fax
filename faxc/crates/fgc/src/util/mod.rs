//! Util Module - Shared Utilities
//!
//! Utilities and helper functions used throughout FGC.

pub mod alignment;
pub mod atomic;
pub mod debug;

pub use alignment::Alignment;
pub use atomic::AtomicUtils;

/// Size class for objects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    Tiny,   // < 64 bytes
    Small,  // 64 - 256 bytes
    Medium, // 256 bytes - 4KB
    Large,  // > 4KB
}

impl SizeClass {
    /// Determine size class from object size
    pub fn from_size(size: usize) -> Self {
        match size {
            0..=63 => Self::Tiny,
            64..=256 => Self::Small,
            257..=4096 => Self::Medium,
            _ => Self::Large,
        }
    }
}

/// Constants for FGC
pub mod constants {
    /// 1 Kilobyte
    pub const KB: usize = 1024;
    /// 1 Megabyte
    pub const MB: usize = 1024 * 1024;
    /// 1 Gigabyte
    pub const GB: usize = 1024 * 1024 * 1024;

    /// Small region size: 2MB
    pub const SMALL_REGION_SIZE: usize = 2 * MB;
    /// Medium region size: 32MB
    pub const MEDIUM_REGION_SIZE: usize = 32 * MB;

    /// Small object threshold: 256 bytes
    pub const SMALL_THRESHOLD: usize = 256;
    /// Large object threshold: 4KB
    pub const LARGE_THRESHOLD: usize = 4 * KB;

    /// Default TLAB size: 256KB
    pub const DEFAULT_TLAB_SIZE: usize = 256 * KB;
    /// Minimum TLAB size: 16KB
    pub const MIN_TLAB_SIZE: usize = 16 * KB;
    /// Maximum TLAB size: 2MB
    pub const MAX_TLAB_SIZE: usize = 2 * MB;

    /// Mark bitmap granularity: 64 bytes per bit
    pub const MARK_BITMAP_GRANULARITY: usize = 64;

    /// Default alignment: 8 bytes
    pub const DEFAULT_ALIGNMENT: usize = 8;
    /// Cache line alignment: 64 bytes
    pub const CACHE_LINE_ALIGNMENT: usize = 64;
    /// Page alignment: 4KB
    pub const PAGE_ALIGNMENT: usize = 4096;
}
