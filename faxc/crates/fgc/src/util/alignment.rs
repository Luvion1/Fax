//! Alignment Utilities
//!
//! Helper functions for memory alignment.

/// Alignment - utility for alignment operations
pub struct Alignment;

impl Alignment {
    /// Align value up to boundary
    ///
    /// # Examples
    /// ```
    /// assert_eq!(Alignment::align_up(100, 8), 104);
    /// assert_eq!(Alignment::align_up(64, 8), 64);
    /// ```
    pub fn align_up(value: usize, alignment: usize) -> usize {
        (value + alignment - 1) & !(alignment - 1)
    }

    /// Align value down to boundary
    pub fn align_down(value: usize, alignment: usize) -> usize {
        value & !(alignment - 1)
    }

    /// Check if value is aligned
    pub fn is_aligned(value: usize, alignment: usize) -> bool {
        value & (alignment - 1) == 0
    }

    /// Get alignment padding needed
    pub fn padding(value: usize, alignment: usize) -> usize {
        Self::align_up(value, alignment) - value
    }

    /// Default object alignment (8 bytes)
    pub const DEFAULT: usize = 8;

    /// Cache line alignment (64 bytes)
    pub const CACHE_LINE: usize = 64;

    /// Page alignment (4KB)
    pub const PAGE: usize = 4096;
}
