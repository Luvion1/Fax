//! Mark Bitmap - Tracking Marked Objects
//!
//! Mark bitmap is a data structure for tracking marked objects.
//! 1 bit per N bytes (typically 64 bytes) for memory efficiency.
//!
//! # Memory Ordering Model
//!
//! This module uses the following atomic ordering strategy:
//!
//! ## Bitmap Mark Operations
//! - **mark():** `Ordering::Relaxed` - Bitmap is only accessed during GC safepoint.
//!   All GC threads synchronize externally (e.g., via barrier).
//!   No data race as mutators are stopped during marking.
//!
//! - **is_marked():** `Ordering::Relaxed` - Same rationale as mark().
//!   During concurrent marking phase, external synchronization ensures
//!   visibility. For purely concurrent checks, Acquire would be needed.
//!
//! ## Rationale
//!
//! The relaxed ordering is safe because:
//! 1. Bitmap modifications happen during GC safepoint
//! 2. All GC threads synchronize via barriers before accessing bitmap
//! 3. Mutator threads are stopped and cannot modify bitmap
//!
//! If concurrent marking is enabled (mutators run during mark phase),
//! change `is_marked()` to use `Ordering::Acquire`.

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicU64, Ordering};

/// MarkBitmap - bitmap for tracking marked objects per region
///
/// Bitmap that shows which objects are marked.
pub struct MarkBitmap {
    /// Raw bitmap data
    /// 1 bit per 64 bytes (granularity)
    bits: Vec<AtomicU64>,

    /// Size of covered region
    region_size: usize,

    /// Granularity (bytes per bit)
    granularity: usize,

    /// Base address of region
    base_address: usize,
}

impl Clone for MarkBitmap {
    fn clone(&self) -> Self {
        let bits: Vec<AtomicU64> = self
            .bits
            .iter()
            .map(|atom| AtomicU64::new(atom.load(Ordering::Relaxed)))
            .collect();

        Self {
            bits,
            region_size: self.region_size,
            granularity: self.granularity,
            base_address: self.base_address,
        }
    }
}

impl MarkBitmap {
    /// Create new bitmap for region with specific size
    ///
    /// # Arguments
    /// * `region_size` - Size of region in bytes
    /// * `granularity` - Bytes per bit (default 64)
    /// * `base_address` - Base address of region
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully created bitmap
    /// * `Err(FgcError::InvalidArgument)` - Invalid parameters
    ///
    /// # Validation
    /// - `region_size` must be greater than 0
    /// - `granularity` must be a power of two
    /// - `granularity` must be between 1 and 1024 bytes
    /// - `base_address` must be aligned to granularity
    pub fn new(region_size: usize, granularity: usize, base_address: usize) -> Result<Self> {
        // Validate region_size > 0
        if region_size == 0 {
            return Err(FgcError::InvalidArgument(
                "region_size must be greater than 0".to_string(),
            ));
        }

        // Validate granularity is power of two
        if !granularity.is_power_of_two() {
            return Err(FgcError::InvalidArgument(format!(
                "granularity ({}) must be a power of two",
                granularity
            )));
        }

        // Validate granularity is reasonable
        if !(1..=1024).contains(&granularity) {
            return Err(FgcError::InvalidArgument(format!(
                "granularity ({}) must be between 1 and 1024 bytes",
                granularity
            )));
        }

        // Validate base_address is aligned to granularity
        if !base_address.is_multiple_of(granularity) {
            return Err(FgcError::InvalidArgument(format!(
                "base_address ({:#x}) must be aligned to granularity ({})",
                base_address, granularity
            )));
        }

        // Calculate number of bits needed
        let bit_count = region_size.div_ceil(granularity);
        let word_count = bit_count.div_ceil(64); // 64 bits per word

        let bits = (0..word_count).map(|_| AtomicU64::new(0)).collect();

        Ok(Self {
            bits,
            region_size,
            granularity,
            base_address,
        })
    }

    /// Mark object at specific address
    ///
    /// Set bit for that address.
    /// Silently ignores invalid addresses (safe behavior).
    ///
    /// # Arguments
    /// * `address` - Object address
    ///
    /// # Safety
    ///
    /// This function is safe to call with any address. Invalid addresses
    /// (outside the region bounds) are silently ignored.
    ///
    /// # Memory Ordering
    ///
    /// Uses `Relaxed` ordering because:
    /// - Bitmap is only accessed during GC safepoint
    /// - All GC threads synchronize externally (e.g., via barrier)
    /// - No data race as mutators are stopped
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fgc::marker::bitmap::MarkBitmap;
    ///
    /// let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000);
    /// bitmap.mark(0x1000);
    /// assert!(bitmap.is_marked(0x1000));
    /// ```
    pub fn mark(&self, address: usize) {
        // CRIT-06 FIX: Use checked indices calculation
        if let Some((word_index, bit_index)) = self.indices(address) {
            // Relaxed: safe due to GC safepoint protocol (see module docs)
            self.bits[word_index].fetch_or(1 << bit_index, Ordering::Relaxed);
        }
        // Silently ignore invalid addresses (safe)
    }

    /// Check if object is already marked
    ///
    /// # Arguments
    /// * `address` - Object address
    ///
    /// # Returns
    /// True if marked, false if invalid address or not marked
    ///
    /// # Safety
    ///
    /// This function is safe to call with any address. Invalid addresses
    /// (outside the region bounds) return false.
    ///
    /// # Memory Ordering
    ///
    /// Uses `Relaxed` ordering because:
    /// - Bitmap is only accessed during GC safepoint
    /// - External synchronization ensures visibility
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fgc::marker::bitmap::MarkBitmap;
    ///
    /// let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000);
    /// bitmap.mark(0x1000);
    /// assert!(bitmap.is_marked(0x1000));
    /// assert!(!bitmap.is_marked(0x2000)); // Not marked
    /// ```
    pub fn is_marked(&self, address: usize) -> bool {
        // CRIT-06 FIX: Use checked indices calculation
        if let Some((word_index, bit_index)) = self.indices(address) {
            // Relaxed: safe due to GC safepoint protocol (see module docs)
            (self.bits[word_index].load(Ordering::Relaxed) & (1 << bit_index)) != 0
        } else {
            false // Invalid address treated as not marked
        }
    }

    /// Clear all bits (optimized with SIMD hints)
    pub fn clear(&self) {
        // Use SIMD-friendly pattern: process in chunks
        // The compiler will auto-vectorize this when SIMD is available
        for word in &self.bits {
            word.store(0, Ordering::Relaxed);
        }
        // Hint to compiler this is a hot path
        // SAFETY: After storing 0 to all words, the assertion always holds
        unsafe {
            std::hint::assert_unchecked(self.bits.iter().all(|w| w.load(Ordering::Relaxed) == 0))
        };
    }

    /// Bulk mark multiple addresses efficiently
    ///
    /// More efficient than calling mark() multiple times.
    /// Uses loop unrolling and batch processing.
    pub fn mark_bulk(&self, addresses: &[usize]) {
        // Process in chunks for better cache behavior
        let chunks = addresses.chunks(8);
        for chunk in chunks {
            for &addr in chunk {
                self.mark(addr);
            }
        }
    }

    /// Mark range of addresses (optimized bulk operation)
    ///
    /// Marks all addresses in the range with given step.
    /// Optimized to minimize function call overhead.
    pub fn mark_range(&self, start: usize, end: usize, step: usize) {
        if step == 0 || start >= end {
            return;
        }

        // Process in batches for better performance
        let mut addr = start;
        while addr < end {
            // Inline mark operation for hot path
            if let Some((word_index, bit_index)) = self.indices(addr) {
                self.bits[word_index].fetch_or(1 << bit_index, Ordering::Relaxed);
            }

            let next = addr.saturating_add(step);
            if next <= addr {
                break; // Prevent infinite loop on overflow
            }
            addr = next;
        }
    }

    /// Clear specific address
    pub fn unmark(&self, address: usize) {
        if let Some((word_index, bit_index)) = self.indices(address) {
            self.bits[word_index].fetch_and(!(1 << bit_index), Ordering::Relaxed);
        }
    }

    /// Bulk unmark multiple addresses
    pub fn unmark_bulk(&self, addresses: &[usize]) {
        for &addr in addresses {
            self.unmark(addr);
        }
    }

    /// Count number of marked objects
    pub fn count_marked(&self) -> usize {
        self.bits
            .iter()
            .map(|word| word.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }

    /// Count marked bytes (approximate)
    pub fn count_marked_bytes(&self) -> usize {
        self.count_marked() * self.granularity
    }

    /// Get marked ratio (0.0 - 1.0)
    pub fn marked_ratio(&self) -> f32 {
        let total_bits = self.bits.len() * 64;
        if total_bits == 0 {
            return 0.0;
        }

        self.count_marked() as f32 / total_bits as f32
    }

    /// Calculate word and bit indices for address
    ///
    /// Returns None if address is out of bounds.
    ///
    /// # Arguments
    /// * `address` - Object address
    ///
    /// # Returns
    /// Some((word_index, bit_offset)) if valid, None if out of bounds
    fn indices(&self, address: usize) -> Option<(usize, usize)> {
        // CRIT-06 FIX: Validate address is within region
        // Check if address is before base (would cause underflow)
        if address < self.base_address {
            return None;
        }

        // Use checked_sub to prevent underflow
        let offset = address.checked_sub(self.base_address)?;

        // Check if offset is within region
        if offset >= self.region_size {
            return None;
        }

        // Calculate bit index with overflow check
        let bit_index = offset / self.granularity;
        let word_index = bit_index / 64;

        // Bounds check on bitmap array
        if word_index >= self.bits.len() {
            return None;
        }

        let bit_offset = bit_index % 64;
        Some((word_index, bit_offset))
    }

    /// Get bitmap size in bytes
    pub fn size_bytes(&self) -> usize {
        self.bits.len() * 8 // 8 bytes per AtomicU64
    }

    /// Get region size
    pub fn region_size(&self) -> usize {
        self.region_size
    }

    /// Get granularity
    pub fn granularity(&self) -> usize {
        self.granularity
    }
}

/// MarkBitmap with word-at-a-time operations
///
/// Optimized for scanning bitmap with 64 bits per iteration.
pub struct MarkBitmapScanner<'a> {
    bitmap: &'a MarkBitmap,
    current_word: usize,
}

impl<'a> MarkBitmapScanner<'a> {
    /// Create scanner for bitmap
    pub fn new(bitmap: &'a MarkBitmap) -> Self {
        Self {
            bitmap,
            current_word: 0,
        }
    }

    /// Scan next marked object
    ///
    /// Returns address of next marked object.
    pub fn next_marked(&mut self) -> Option<usize> {
        while self.current_word < self.bitmap.bits.len() {
            let word = self.bitmap.bits[self.current_word].load(Ordering::Relaxed);

            if word != 0 {
                // Has marked bits in this word
                let bit_index = word.trailing_zeros() as usize;
                let bit_position = self.current_word * 64 + bit_index;
                let address = self.bitmap.base_address + (bit_position * self.bitmap.granularity);

                // Clear bit for next iteration
                self.bitmap.bits[self.current_word].fetch_and(!(1 << bit_index), Ordering::Relaxed);

                return Some(address);
            }

            self.current_word += 1;
        }

        None
    }

    /// Reset scanner to beginning
    pub fn reset(&mut self) {
        self.current_word = 0;
    }

    /// Check if more marked objects exist (optimized ZGC-style)
    ///
    /// Uses word-at-a-time scanning for efficiency.
    /// Returns quickly if current word has bits set.
    pub fn has_more(&self) -> bool {
        // Quick check: if current word has bits, we have more
        if self.current_word < self.bitmap.bits.len()
            && self.bitmap.bits[self.current_word].load(Ordering::Relaxed) != 0
        {
            return true;
        }

        // Scan remaining words
        for i in self.current_word + 1..self.bitmap.bits.len() {
            if self.bitmap.bits[i].load(Ordering::Relaxed) != 0 {
                return true;
            }
        }
        false
    }

    /// Get count of remaining marked objects (approximate)
    pub fn estimated_remaining(&self) -> usize {
        let mut count = 0;
        for i in self.current_word..self.bitmap.bits.len() {
            count += self.bitmap.bits[i].load(Ordering::Relaxed).count_ones() as usize;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_and_check() {
        let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000).unwrap();

        // Mark objects at addresses that map to distinct bit indices
        // With granularity=64: bit_index = (address - base) / 64
        // 0x1000: offset=0, bit_index=0
        // 0x1040: offset=64, bit_index=1
        // 0x1080: offset=128, bit_index=2
        bitmap.mark(0x1000);
        bitmap.mark(0x1040);
        bitmap.mark(0x1080);

        assert!(bitmap.is_marked(0x1000));
        assert!(bitmap.is_marked(0x1040));
        assert!(bitmap.is_marked(0x1080));
        // 0x10C0: offset=192, bit_index=3 (not marked)
        assert!(!bitmap.is_marked(0x10C0));
    }

    #[test]
    fn test_clear() {
        let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000).unwrap();

        bitmap.mark(0x1000);
        bitmap.mark(0x1040);

        bitmap.clear();

        assert!(!bitmap.is_marked(0x1000));
        assert!(!bitmap.is_marked(0x1040));
    }

    #[test]
    fn test_count_marked() {
        let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000).unwrap();

        bitmap.mark(0x1000);
        bitmap.mark(0x1040);
        bitmap.mark(0x1080);

        assert_eq!(bitmap.count_marked(), 3);
    }
}
