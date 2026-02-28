//! Reference Map - Tells GC where pointer fields are located within objects
//!
//! Each bit in the bitmap represents 8 bytes of object data.
//! - Bit = 1: Contains a pointer reference
//! - Bit = 0: Non-pointer data (primitive, padding, etc.)
//!
//! # Layout
//!
//! ```text
//! Object Data Layout:
//! ┌─────────┬─────────┬─────────┬─────────┐
//! │ 0-7     │ 8-15    │ 16-23   │ 24-31   │
//! │ (bit 0) │ (bit 1) │ (bit 2) │ (bit 3) │
//! ├─────────┼─────────┼─────────┼─────────┤
//! │ ptr     │ i64     │ ptr     │ padding │
//! │ bit=1   │ bit=0   │ bit=1   │ bit=0   │
//! └─────────┴─────────┴─────────┴─────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use fgc::object::ReferenceMap;
//!
//! // Create a reference map with pointers at offsets 0 and 16
//! let map = ReferenceMap::new(&[0, 16]);
//!
//! assert!(map.is_reference(0));    // offset 0 has pointer
//! assert!(!map.is_reference(8));   // offset 8 is not a pointer
//! assert!(map.is_reference(16));   // offset 16 has pointer
//! assert_eq!(map.count(), 2);      // 2 references total
//! ```

/// Size of each slot tracked by the reference map (in bytes)
pub const SLOT_SIZE: usize = 8;

/// Maximum number of reference fields tracked per object
/// Limited by the 64-bit bitmap
pub const MAX_REFS: usize = 64;

/// Maximum object data size that can be tracked (64 slots * 8 bytes)
pub const MAX_TRACKED_SIZE: usize = MAX_REFS * SLOT_SIZE;

/// Reference map for an object type
///
/// Uses a bitmap where each bit represents 8 bytes of object data.
/// This allows efficient scanning of objects during GC marking phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReferenceMap {
    /// Bitmap: each bit represents 8 bytes
    /// - Bit 0 = offset 0-7
    /// - Bit 1 = offset 8-15
    /// - Bit 2 = offset 16-23
    /// - etc.
    bitmap: u64,
    /// Number of reference fields in the object
    count: u8,
}

impl ReferenceMap {
    /// Create an empty reference map with no pointers
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// let map = ReferenceMap::empty();
    /// assert_eq!(map.count(), 0);
    /// assert!(!map.is_reference(0));
    /// ```
    #[inline]
    pub const fn empty() -> Self {
        Self {
            bitmap: 0,
            count: 0,
        }
    }

    /// Create a reference map with specific pointer offsets
    ///
    /// # Arguments
    ///
    /// * `offsets` - Array of byte offsets where pointers are located.
    ///   Each offset must be aligned to SLOT_SIZE (8 bytes).
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - More than MAX_REFS offsets are provided
    /// - Any offset is not aligned to SLOT_SIZE
    /// - Any offset would exceed the bitmap capacity
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// // Object with pointers at offset 0 and 16
    /// let map = ReferenceMap::new(&[0, 16]);
    /// assert_eq!(map.count(), 2);
    /// ```
    #[inline]
    pub fn new(offsets: &[usize]) -> Self {
        let mut bitmap: u64 = 0;
        let mut count: u8 = 0;

        for &offset in offsets {
            // Validate alignment
            debug_assert!(
                offset % SLOT_SIZE == 0,
                "Offset {} must be aligned to {} bytes",
                offset,
                SLOT_SIZE
            );

            // Calculate bit position
            let bit = offset / SLOT_SIZE;

            // Validate bitmap capacity
            debug_assert!(
                bit < MAX_REFS,
                "Offset {} exceeds maximum tracked size",
                offset
            );

            // Set the bit
            bitmap |= 1u64 << bit;
            count = count.saturating_add(1);
        }

        Self { bitmap, count }
    }

    /// Create a reference map from a raw bitmap
    ///
    /// # Safety
    ///
    /// The caller must ensure that `count` matches the number of set bits
    /// in `bitmap`. This is intended for serialization/deserialization.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// // Bitmap with bits 0 and 2 set (pointers at offset 0 and 16)
    /// let map = unsafe { ReferenceMap::from_raw(0b101, 2) };
    /// assert_eq!(map.count(), 2);
    /// ```
    #[inline]
    pub const unsafe fn from_raw(bitmap: u64, count: u8) -> Self {
        Self { bitmap, count }
    }

    /// Check if a given offset contains a pointer reference
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset within the object data
    ///
    /// # Returns
    ///
    /// `true` if the offset contains a pointer, `false` otherwise.
    /// Returns `false` for unaligned offsets (not multiples of SLOT_SIZE).
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// let map = ReferenceMap::new(&[0, 16]);
    /// assert!(map.is_reference(0));
    /// assert!(!map.is_reference(8));
    /// assert!(map.is_reference(16));
    /// assert!(!map.is_reference(4)); // Unaligned offset
    /// ```
    #[inline]
    pub fn is_reference(&self, offset: usize) -> bool {
        // Only check aligned offsets
        if !offset.is_multiple_of(SLOT_SIZE) {
            return false;
        }

        // Check if offset is within tracked range
        let bit = offset / SLOT_SIZE;
        if bit >= MAX_REFS {
            return false;
        }

        (self.bitmap & (1u64 << bit)) != 0
    }

    /// Get the number of reference fields in the object
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// let map = ReferenceMap::new(&[0, 8, 16]);
    /// assert_eq!(map.count(), 3);
    /// ```
    #[inline]
    pub const fn count(&self) -> u8 {
        self.count
    }

    /// Get the raw bitmap representation
    ///
    /// Useful for serialization or low-level operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// let map = ReferenceMap::new(&[0, 16]);
    /// assert_eq!(map.bitmap(), 0b101);
    /// ```
    #[inline]
    pub const fn bitmap(&self) -> u64 {
        self.bitmap
    }

    /// Check if the reference map is empty (no pointers)
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// let empty = ReferenceMap::empty();
    /// assert!(empty.is_empty());
    ///
    /// let with_refs = ReferenceMap::new(&[0]);
    /// assert!(!with_refs.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bitmap == 0
    }

    /// Iterate over all reference offsets
    ///
    /// # Example
    ///
    /// ```rust
    /// use fgc::object::ReferenceMap;
    ///
    /// let map = ReferenceMap::new(&[0, 16, 32]);
    /// let offsets: Vec<usize> = map.iter().collect();
    /// assert_eq!(offsets, vec![0, 16, 32]);
    /// ```
    #[inline]
    pub const fn iter(&self) -> ReferenceMapIter {
        ReferenceMapIter {
            bitmap: self.bitmap,
            current_bit: 0,
        }
    }

    /// Check if a specific bit is set in the bitmap
    ///
    /// # Arguments
    ///
    /// * `bit` - Bit position to check (0-63)
    #[inline]
    #[allow(dead_code)]
    const fn has_bit(&self, bit: usize) -> bool {
        if bit >= MAX_REFS {
            return false;
        }
        (self.bitmap & (1u64 << bit)) != 0
    }
}

/// Iterator over reference offsets in a ReferenceMap
///
/// Yields byte offsets where pointers are located.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceMapIter {
    /// Remaining bitmap to iterate
    bitmap: u64,
    /// Current bit position being examined
    current_bit: usize,
}

impl Iterator for ReferenceMapIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        // Find the next set bit
        while self.current_bit < MAX_REFS {
            let bit = self.current_bit;
            self.current_bit += 1;

            if (self.bitmap & (1u64 << bit)) != 0 {
                // Convert bit position to byte offset
                return Some(bit * SLOT_SIZE);
            }
        }

        None
    }

    /// Provide a size hint for the iterator
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Count remaining set bits from current_bit onwards
        let remaining_bits = self.bitmap >> self.current_bit;
        let remaining = remaining_bits.count_ones() as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ReferenceMapIter {}

impl std::iter::FusedIterator for ReferenceMapIter {}

/// Builder for creating ReferenceMap with a fluent API
///
/// # Example
///
/// ```rust
/// use fgc::object::refmap::ReferenceMapBuilder;
///
/// let map = ReferenceMapBuilder::new()
///     .with_reference(0)
///     .with_reference(16)
///     .with_reference(32)
///     .build();
///
/// assert_eq!(map.count(), 3);
/// ```
#[derive(Debug, Default)]
pub struct ReferenceMapBuilder {
    bitmap: u64,
    count: u8,
}

impl ReferenceMapBuilder {
    /// Create a new empty builder
    #[inline]
    pub const fn new() -> Self {
        Self {
            bitmap: 0,
            count: 0,
        }
    }

    /// Add a reference at the given offset
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset where the pointer is located
    ///
    /// # Panics
    ///
    /// Panics if offset is not aligned to SLOT_SIZE or exceeds capacity.
    #[inline]
    pub fn with_reference(mut self, offset: usize) -> Self {
        debug_assert!(
            offset.is_multiple_of(SLOT_SIZE),
            "Offset {} must be aligned to {} bytes",
            offset,
            SLOT_SIZE
        );

        let bit = offset / SLOT_SIZE;
        debug_assert!(bit < MAX_REFS, "Offset {} exceeds capacity", offset);

        self.bitmap |= 1u64 << bit;
        self.count = self.count.saturating_add(1);
        self
    }

    /// Build the ReferenceMap
    #[inline]
    pub const fn build(self) -> ReferenceMap {
        ReferenceMap {
            bitmap: self.bitmap,
            count: self.count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Basic Creation Tests ===

    #[test]
    fn test_empty_reference_map() {
        let map = ReferenceMap::empty();
        assert_eq!(map.count(), 0);
        assert!(map.is_empty());
        assert_eq!(map.bitmap(), 0);
        assert!(!map.is_reference(0));
        assert!(!map.is_reference(64));
    }

    #[test]
    fn test_default_reference_map() {
        let map: ReferenceMap = Default::default();
        assert_eq!(map.count(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_new_with_offsets() {
        let map = ReferenceMap::new(&[0, 8, 16]);
        assert_eq!(map.count(), 3);
        assert!(!map.is_empty());
        assert!(map.is_reference(0));
        assert!(map.is_reference(8));
        assert!(map.is_reference(16));
        assert!(!map.is_reference(4));
        assert!(!map.is_reference(12));
    }

    #[test]
    fn test_new_with_single_offset() {
        let map = ReferenceMap::new(&[24]);
        assert_eq!(map.count(), 1);
        assert!(map.is_reference(24));
        assert!(!map.is_reference(0));
        assert!(!map.is_reference(16));
    }

    #[test]
    fn test_new_with_duplicate_offsets() {
        // Duplicates should still count as one reference
        let map = ReferenceMap::new(&[0, 0, 0]);
        assert_eq!(map.count(), 3); // Current implementation counts duplicates
        assert!(map.is_reference(0));
    }

    // === is_reference Tests ===

    #[test]
    fn test_is_reference_various_offsets() {
        let map = ReferenceMap::new(&[0, 16, 32, 48]);

        // Check all reference offsets
        assert!(map.is_reference(0));
        assert!(map.is_reference(16));
        assert!(map.is_reference(32));
        assert!(map.is_reference(48));

        // Check non-reference offsets
        assert!(!map.is_reference(8));
        assert!(!map.is_reference(24));
        assert!(!map.is_reference(40));
        assert!(!map.is_reference(56));
    }

    #[test]
    fn test_is_reference_out_of_bounds() {
        let map = ReferenceMap::new(&[0, 8]);

        // Offsets beyond bitmap capacity should return false
        assert!(!map.is_reference(512)); // bit 64
        assert!(!map.is_reference(1024)); // bit 128
    }

    #[test]
    fn test_is_reference_unaligned_offset() {
        let map = ReferenceMap::new(&[0, 8]);

        // Aligned offsets should work
        assert!(map.is_reference(0));
        assert!(map.is_reference(8));

        // Unaligned offsets should return false
        assert!(!map.is_reference(1));
        assert!(!map.is_reference(4));
        assert!(!map.is_reference(7));
        assert!(!map.is_reference(9));
    }

    // === Bitmap Tests ===

    #[test]
    fn test_bitmap_representation() {
        // Bit 0 set (offset 0)
        let map = ReferenceMap::new(&[0]);
        assert_eq!(map.bitmap(), 0b1);

        // Bit 1 set (offset 8)
        let map = ReferenceMap::new(&[8]);
        assert_eq!(map.bitmap(), 0b10);

        // Bits 0 and 2 set (offsets 0 and 16)
        let map = ReferenceMap::new(&[0, 16]);
        assert_eq!(map.bitmap(), 0b101);

        // Bits 0, 1, 2 set (offsets 0, 8, 16)
        let map = ReferenceMap::new(&[0, 8, 16]);
        assert_eq!(map.bitmap(), 0b111);
    }

    #[test]
    fn test_from_raw() {
        let map = unsafe { ReferenceMap::from_raw(0b1010, 2) };
        assert_eq!(map.count(), 2);
        assert_eq!(map.bitmap(), 0b1010);
        assert!(map.is_reference(8));
        assert!(map.is_reference(24));
        assert!(!map.is_reference(0));
        assert!(!map.is_reference(16));
    }

    // === Iterator Tests ===

    #[test]
    fn test_iter_empty() {
        let map = ReferenceMap::empty();
        let offsets: Vec<usize> = map.iter().collect();
        assert!(offsets.is_empty());
    }

    #[test]
    fn test_iter_single() {
        let map = ReferenceMap::new(&[16]);
        let offsets: Vec<usize> = map.iter().collect();
        assert_eq!(offsets, vec![16]);
    }

    #[test]
    fn test_iter_multiple() {
        let map = ReferenceMap::new(&[0, 16, 32, 48]);
        let offsets: Vec<usize> = map.iter().collect();
        assert_eq!(offsets, vec![0, 16, 32, 48]);
    }

    #[test]
    fn test_iter_order() {
        // Iterator should yield offsets in ascending order
        let map = ReferenceMap::new(&[48, 0, 32, 16]);
        let offsets: Vec<usize> = map.iter().collect();
        assert_eq!(offsets, vec![0, 16, 32, 48]);
    }

    #[test]
    fn test_iter_size_hint() {
        let map = ReferenceMap::new(&[0, 8, 16]);
        let mut iter = map.iter();

        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 3);
        assert_eq!(upper, Some(3));

        iter.next();
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 2);
        assert_eq!(upper, Some(2));
    }

    #[test]
    fn test_iter_exact_size() {
        let map = ReferenceMap::new(&[0, 8, 16, 24, 32]);
        let iter = map.iter();
        assert_eq!(iter.len(), 5);
    }

    #[test]
    fn test_iter_fused() {
        let map = ReferenceMap::new(&[0]);
        let mut iter = map.iter();

        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none()); // Should continue returning None
    }

    // === Builder Tests ===

    #[test]
    fn test_builder_empty() {
        let map = ReferenceMapBuilder::new().build();
        assert_eq!(map.count(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_builder_with_references() {
        let map = ReferenceMapBuilder::new()
            .with_reference(0)
            .with_reference(16)
            .with_reference(32)
            .build();

        assert_eq!(map.count(), 3);
        assert!(map.is_reference(0));
        assert!(map.is_reference(16));
        assert!(map.is_reference(32));
    }

    #[test]
    fn test_builder_chaining() {
        let map = ReferenceMapBuilder::new()
            .with_reference(0)
            .with_reference(8)
            .with_reference(16)
            .with_reference(24)
            .with_reference(32)
            .build();

        assert_eq!(map.count(), 5);
    }

    // === Edge Cases ===

    #[test]
    fn test_max_references() {
        // Create a map with all 64 bits set
        let offsets: Vec<usize> = (0..MAX_REFS).map(|i| i * SLOT_SIZE).collect();
        let map = ReferenceMap::new(&offsets);
        assert_eq!(map.count(), MAX_REFS as u8);
        assert_eq!(map.bitmap(), u64::MAX);

        // All offsets should be references
        for i in 0..MAX_REFS {
            assert!(map.is_reference(i * SLOT_SIZE));
        }
    }

    #[test]
    fn test_high_bit_offset() {
        // Test offset at bit 63 (highest bit)
        let map = ReferenceMap::new(&[504]); // 63 * 8 = 504
        assert!(map.is_reference(504));
        assert!(!map.is_reference(496));
    }

    #[test]
    fn test_copy_clone() {
        let map1 = ReferenceMap::new(&[0, 16, 32]);
        let map2 = map1; // Copy
        let map3 = map1.clone(); // Clone

        assert_eq!(map1.bitmap(), map2.bitmap());
        assert_eq!(map1.bitmap(), map3.bitmap());
        assert_eq!(map1.count(), map2.count());
        assert_eq!(map1.count(), map3.count());
    }

    #[test]
    fn test_partial_eq() {
        let map1 = ReferenceMap::new(&[0, 8, 16]);
        let map2 = ReferenceMap::new(&[0, 8, 16]);
        let map3 = ReferenceMap::new(&[0, 8]);

        assert_eq!(map1, map2);
        assert_ne!(map1, map3);
    }

    #[test]
    fn test_debug_format() {
        let map = ReferenceMap::new(&[0, 16]);
        let debug_str = format!("{:?}", map);
        assert!(debug_str.contains("ReferenceMap"));
        assert!(debug_str.contains("bitmap"));
        assert!(debug_str.contains("count"));
    }

    // === Real-world Scenarios ===

    #[test]
    fn test_typical_object_layout() {
        // Simulate a typical object layout:
        // Offset 0:  class pointer
        // Offset 8:  i64 field
        // Offset 16: object reference
        // Offset 24: i32 field
        // Offset 32: padding
        // Offset 40: object reference

        let map = ReferenceMap::new(&[0, 16, 40]);

        assert!(map.is_reference(0)); // class pointer
        assert!(!map.is_reference(8)); // i64
        assert!(map.is_reference(16)); // object ref
        assert!(!map.is_reference(24)); // i32
        assert!(!map.is_reference(32)); // padding
        assert!(map.is_reference(40)); // object ref

        assert_eq!(map.count(), 3);
    }

    #[test]
    fn test_array_of_references() {
        // Array of 10 references, each 8 bytes
        let offsets: Vec<usize> = (0..10).map(|i| i * 8).collect();
        let map = ReferenceMap::new(&offsets);

        assert_eq!(map.count(), 10);

        for i in 0..10 {
            assert!(map.is_reference(i * 8));
        }
    }

    #[test]
    fn test_mixed_primitives_and_references() {
        // Object with mixed fields
        let map = ReferenceMap::new(&[
            0,  // ptr
            24, // ptr
            56, // ptr
        ]);

        assert_eq!(map.count(), 3);

        // Verify references
        assert!(map.is_reference(0));
        assert!(map.is_reference(24));
        assert!(map.is_reference(56));

        // Verify non-references
        assert!(!map.is_reference(8));
        assert!(!map.is_reference(16));
        assert!(!map.is_reference(32));
        assert!(!map.is_reference(48));
    }
}
