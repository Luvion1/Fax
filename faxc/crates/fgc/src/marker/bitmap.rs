//! Mark Bitmap - Tracking Marked Objects
//!
//! Mark bitmap adalah struktur data untuk tracking object yang sudah marked.
//! 1 bit per N bytes (biasanya 64 bytes) untuk efisiensi memory.
//!
//! Bitmap Structure:
//! ```
//! Region: 2MB (2,097,152 bytes)
//! Granularity: 64 bytes per bit
//! Bitmap size: 2MB / 64 = 32,768 bits = 4KB
//!
//! Object at address 0x1000 (4096):
//! - Offset: 4096 - region_start
//! - Bit index: 4096 / 64 = 64
//! - Byte index: 64 / 8 = 8
//! - Bit offset: 64 % 8 = 0
//! ```

use std::sync::atomic::{AtomicU64, Ordering};

/// MarkBitmap - bitmap untuk tracking marked objects per region
///
/// Bitmap yang menunjukkan object mana yang sudah di-mark.
pub struct MarkBitmap {
    /// Raw bitmap data
    /// 1 bit per 64 bytes (granularity)
    bits: Vec<AtomicU64>,

    /// Ukuran region yang dicover
    region_size: usize,

    /// Granularity (bytes per bit)
    granularity: usize,

    /// Base address region
    base_address: usize,
}

impl Clone for MarkBitmap {
    fn clone(&self) -> Self {
        let bits: Vec<AtomicU64> = self.bits
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
    /// Create new bitmap untuk region dengan ukuran tertentu
    ///
    /// # Arguments
    /// * `region_size` - Size region dalam bytes
    /// * `granularity` - Bytes per bit (default 64)
    /// * `base_address` - Base address region
    pub fn new(region_size: usize, granularity: usize, base_address: usize) -> Self {
        // Calculate jumlah bits yang dibutuhkan
        let bit_count = (region_size + granularity - 1) / granularity;
        let word_count = (bit_count + 63) / 64; // 64 bits per word

        let bits = (0..word_count)
            .map(|_| AtomicU64::new(0))
            .collect();

        Self {
            bits,
            region_size,
            granularity,
            base_address,
        }
    }

    /// Mark object di specific address
    ///
    /// Set bit untuk address tersebut.
    ///
    /// # Arguments
    /// * `address` - Object address
    pub fn mark(&self, address: usize) {
        let (word_index, bit_index) = self.calculate_indices(address);

        if word_index < self.bits.len() {
            self.bits[word_index].fetch_or(1 << bit_index, Ordering::Relaxed);
        }
    }

    /// Check jika object sudah marked
    ///
    /// # Arguments
    /// * `address` - Object address
    ///
    /// # Returns
    /// True jika marked
    pub fn is_marked(&self, address: usize) -> bool {
        let (word_index, bit_index) = self.calculate_indices(address);

        if word_index >= self.bits.len() {
            return false;
        }

        (self.bits[word_index].load(Ordering::Relaxed) & (1 << bit_index)) != 0
    }

    /// Clear semua bits
    pub fn clear(&self) {
        for word in &self.bits {
            word.store(0, Ordering::Relaxed);
        }
    }

    /// Count jumlah marked objects
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

    /// Calculate word dan bit indices untuk address
    fn calculate_indices(&self, address: usize) -> (usize, usize) {
        let offset = address - self.base_address;
        let bit_index = offset / self.granularity;
        let word_index = bit_index / 64;
        let bit_offset = bit_index % 64;

        (word_index, bit_offset)
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

/// MarkBitmap dengan word-at-a-time operations
///
/// Optimized untuk scanning bitmap dengan 64 bits per iteration.
pub struct MarkBitmapScanner<'a> {
    bitmap: &'a MarkBitmap,
    current_word: usize,
}

impl<'a> MarkBitmapScanner<'a> {
    /// Create scanner untuk bitmap
    pub fn new(bitmap: &'a MarkBitmap) -> Self {
        Self {
            bitmap,
            current_word: 0,
        }
    }

    /// Scan next marked object
    ///
    /// Returns address object berikutnya yang marked.
    pub fn next_marked(&mut self) -> Option<usize> {
        while self.current_word < self.bitmap.bits.len() {
            let word = self.bitmap.bits[self.current_word].load(Ordering::Relaxed);

            if word != 0 {
                // Ada marked bits di word ini
                let bit_index = word.trailing_zeros() as usize;
                let bit_position = self.current_word * 64 + bit_index;
                let address = self.bitmap.base_address + (bit_position * self.bitmap.granularity);

                // Clear bit untuk next iteration
                self.bitmap.bits[self.current_word]
                    .fetch_and(!(1 << bit_index), Ordering::Relaxed);

                return Some(address);
            }

            self.current_word += 1;
        }

        None
    }

    /// Reset scanner ke awal
    pub fn reset(&mut self) {
        self.current_word = 0;
    }

    /// Check jika ada lebih banyak marked objects
    pub fn has_more(&self) -> bool {
        for i in self.current_word..self.bitmap.bits.len() {
            if self.bitmap.bits[i].load(Ordering::Relaxed) != 0 {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_and_check() {
        let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000);

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
        let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000);

        bitmap.mark(0x1000);
        bitmap.mark(0x1040);

        bitmap.clear();

        assert!(!bitmap.is_marked(0x1000));
        assert!(!bitmap.is_marked(0x1040));
    }

    #[test]
    fn test_count_marked() {
        let bitmap = MarkBitmap::new(2 * 1024 * 1024, 64, 0x1000);

        bitmap.mark(0x1000);
        bitmap.mark(0x1040);
        bitmap.mark(0x1080);

        assert_eq!(bitmap.count_marked(), 3);
    }
}
