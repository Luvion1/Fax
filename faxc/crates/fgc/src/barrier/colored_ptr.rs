//! Colored Pointer Implementation
//!
//! Colored pointer adalah teknik dimana metadata GC disimpan langsung di bit-bit
//! yang tidak terpakai dari pointer 64-bit. Ini memungkinkan concurrent operations
//! tanpa mengubah struktur object.
//!
//! Layout Pointer 64-bit:
//! - Bit 48-63: Unused (reserved untuk masa depan)
//! - Bit 44-47: Metadata (Finalizable, Remapped, Marked1, Marked0)
//! - Bit 0-43:  Alamat memori aktual (44 bit = 16TB addressable)
//!
//! Color Bits:
//! - Marked0 (bit 44): Object marked di GC cycle even
//! - Marked1 (bit 45): Object marked di GC cycle odd
//! - Remapped (bit 46): Pointer sudah di-remap ke alamat baru
//! - Finalizable (bit 47): Object perlu finalization sebelum dikoleksi
//!
//! Multi-Mapping Technique:
//! Physical address yang sama bisa diakses via 3 virtual addresses berbeda.
//! Hardware MMU menangani translation, software hanya perlu set color bits.
//!
//! ## Thread Safety
//!
//! `ColoredPointer` sendiri tidak thread-safe (menggunakan `usize`).
//! Untuk concurrent operations, gunakan atomic variants dengan `AtomicUsize`:
//!
//! ```rust
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use fgc::barrier::colored_ptr::ColoredPointer;
//!
//! let atomic_ptr = AtomicUsize::new(0x1000);
//! ColoredPointer::set_marked0_atomic(&atomic_ptr);
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

/// ColoredPointer - wrapper untuk pointer dengan metadata bits
///
/// Representasi pointer 64-bit dengan color bits di bit 44-47.
/// Address effectif ada di bit 0-43 (44 bits = 16TB address space).
///
/// # Examples
///
/// ```rust
/// // Create pointer dari address
/// let ptr = ColoredPointer::new(0x1234);
/// assert_eq!(ptr.address(), 0x1234);
///
/// // Set color bits
/// let mut ptr = ColoredPointer::new(0x1234);
/// ptr.set_marked0();
/// assert!(ptr.is_marked0());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColoredPointer {
    /// Raw 64-bit value dengan color bits
    /// Bit layout: [unused:16][finalizable:1][remapped:1][marked1:1][marked0:1][address:44]
    raw: usize,
}

impl ColoredPointer {
    /// Mask untuk Marked0 bit (bit 44)
    pub const MARKED0_MASK: usize = 1 << 44;

    /// Mask untuk Marked1 bit (bit 45)
    pub const MARKED1_MASK: usize = 1 << 45;

    /// Mask untuk Remapped bit (bit 46)
    pub const REMAPPED_MASK: usize = 1 << 46;

    /// Mask untuk Finalizable bit (bit 47)
    pub const FINALIZABLE_MASK: usize = 1 << 47;

    /// Mask untuk semua color bits (bits 44-47)
    const COLOR_MASK: usize =
        Self::MARKED0_MASK | Self::MARKED1_MASK | Self::REMAPPED_MASK | Self::FINALIZABLE_MASK;

    /// Mask untuk address bits (bits 0-43)
    pub const ADDRESS_MASK: usize = (1 << 44) - 1;

    /// Membuat pointer baru dari address murni
    ///
    /// Address di-mask untuk memastikan hanya 44 bit address.
    /// Color bits diinisialisasi 0 (no color).
    ///
    /// # Arguments
    /// * `address` - Physical address (44 bit max)
    pub fn new(address: usize) -> Self {
        Self {
            raw: address & Self::ADDRESS_MASK,
        }
    }

    /// Mengembalikan address murni tanpa color bits
    ///
    /// Menggunakan bitwise AND dengan ADDRESS_MASK untuk
    /// menghilangkan semua color bits.
    pub fn address(&self) -> usize {
        self.raw & Self::ADDRESS_MASK
    }

    /// Mengecek apakah pointer memiliki Marked0 bit set
    ///
    /// Marked0 menandakan object sudah di-mark di GC cycle even.
    pub fn is_marked0(&self) -> bool {
        (self.raw & Self::MARKED0_MASK) != 0
    }

    /// Mengecek apakah pointer memiliki Marked1 bit set
    ///
    /// Marked1 menandakan object sudah di-mark di GC cycle odd.
    pub fn is_marked1(&self) -> bool {
        (self.raw & Self::MARKED1_MASK) != 0
    }

    /// Mengecek apakah pointer sudah marked (either Marked0 atau Marked1)
    pub fn is_marked(&self) -> bool {
        self.is_marked0() || self.is_marked1()
    }

    /// Mengecek apakah pointer sudah remapped
    ///
    /// Remapped bit menandakan pointer sudah di-update ke alamat baru
    /// setelah relocation.
    pub fn is_remapped(&self) -> bool {
        (self.raw & Self::REMAPPED_MASK) != 0
    }

    /// Mengecek apakah pointer perlu finalization
    ///
    /// Finalizable bit menandakan object memiliki finalizer yang
    /// harus dijalankan sebelum memory di-reclaim.
    pub fn is_finalizable(&self) -> bool {
        (self.raw & Self::FINALIZABLE_MASK) != 0
    }

    /// Set Marked0 bit
    ///
    /// Menggunakan bitwise OR untuk set bit tanpa mengubah bits lain.
    pub fn set_marked0(&mut self) {
        self.raw |= Self::MARKED0_MASK;
    }

    /// Set Marked1 bit
    pub fn set_marked1(&mut self) {
        self.raw |= Self::MARKED1_MASK;
    }

    /// Set Remapped bit
    pub fn set_remapped(&mut self) {
        self.raw |= Self::REMAPPED_MASK;
    }

    /// Set Finalizable bit
    pub fn set_finalizable(&mut self) {
        self.raw |= Self::FINALIZABLE_MASK;
    }

    /// Clear semua color bits (buat jadi no-color / remapped view)
    ///
    /// Menggunakan bitwise AND dengan ADDRESS_MASK untuk
    /// menghilangkan semua color bits.
    pub fn clear_color(&mut self) {
        self.raw &= Self::ADDRESS_MASK;
    }

    // ========================================================================
    // ATOMIC OPERATIONS - Thread-safe variants for concurrent GC
    // ========================================================================
    // These methods operate on AtomicUsize for lock-free concurrent access.
    // Use these when multiple threads may modify the same pointer location.

    /// Set Marked0 bit atomically
    ///
    /// Thread-safe variant of `set_marked0()`. Uses atomic fetch_or
    /// to ensure concurrent marks don't corrupt state.
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Acquire` to ensure subsequent reads see
    /// the updated mark bit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// let atomic_ptr = AtomicUsize::new(0x1000);
    /// ColoredPointer::set_marked0_atomic(&atomic_ptr);
    /// ```
    pub fn set_marked0_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::MARKED0_MASK, Ordering::Acquire);
    }

    /// Set Marked1 bit atomically
    ///
    /// Thread-safe variant of `set_marked1()`.
    pub fn set_marked1_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::MARKED1_MASK, Ordering::Acquire);
    }

    /// Set Remapped bit atomically
    ///
    /// Thread-safe variant of `set_remapped()`.
    pub fn set_remapped_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::REMAPPED_MASK, Ordering::Acquire);
    }

    /// Set Finalizable bit atomically
    ///
    /// Thread-safe variant of `set_finalizable()`.
    pub fn set_finalizable_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::FINALIZABLE_MASK, Ordering::Acquire);
    }

    /// Clear semua color bits atomically
    ///
    /// Thread-safe variant of `clear_color()`. Uses atomic fetch_and
    /// to clear color bits while preserving address.
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Release` to ensure the cleared bits are
    /// visible to other threads before subsequent operations.
    pub fn clear_color_atomic(ptr: &AtomicUsize) {
        ptr.fetch_and(Self::ADDRESS_MASK, Ordering::Release);
    }

    /// Flip Marked0 <-> Marked1 atomically
    ///
    /// Thread-safe variant of `flip_mark_bit()`. Uses CAS loop to
    /// ensure atomic swap of mark bits even under concurrent access.
    ///
    /// # Implementation
    /// Uses compare-and-swap (CAS) loop to atomically:
    /// 1. Read current value
    /// 2. Compute new value with swapped mark bits
    /// 3. CAS to update if value hasn't changed
    ///
    /// # Memory Ordering
    /// Uses `Ordering::AcqRel` (acquire-release) for both success
    /// and failure cases to ensure proper synchronization.
    pub fn flip_mark_bit_atomic(ptr: &AtomicUsize) {
        ptr.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            let marked0 = (current & Self::MARKED0_MASK) != 0;
            let marked1 = (current & Self::MARKED1_MASK) != 0;

            // Clear both mark bits
            let mut new_value = current & !(Self::MARKED0_MASK | Self::MARKED1_MASK);

            // Swap mark bits
            if marked0 {
                new_value |= Self::MARKED1_MASK;
            } else if marked1 {
                new_value |= Self::MARKED0_MASK;
            } else {
                // Neither set - set Marked0 as default
                new_value |= Self::MARKED0_MASK;
            }

            Some(new_value)
        }).ok(); // Ignore error (CAS always succeeds with fetch_update closure)
    }

    /// Check and set Marked0 atomically (test-and-set pattern)
    ///
    /// Returns true if the bit was already set, false if we set it.
    /// Useful for concurrent marking to detect if another thread
    /// already marked this object.
    ///
    /// # Returns
    /// - `true` if Marked0 was already set (another thread marked it)
    /// - `false` if we successfully set Marked0 (we marked it first)
    pub fn test_and_set_marked0(ptr: &AtomicUsize) -> bool {
        let old = ptr.fetch_or(Self::MARKED0_MASK, Ordering::AcqRel);
        (old & Self::MARKED0_MASK) != 0
    }

    /// Check and set Marked1 atomically (test-and-set pattern)
    ///
    /// Returns true if the bit was already set, false if we set it.
    pub fn test_and_set_marked1(ptr: &AtomicUsize) -> bool {
        let old = ptr.fetch_or(Self::MARKED1_MASK, Ordering::AcqRel);
        (old & Self::MARKED1_MASK) != 0
    }

    /// Atomic load of pointer value
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Acquire` to ensure we see all writes
    /// that happened before the pointer was stored.
    pub fn load_atomic(ptr: &AtomicUsize) -> usize {
        ptr.load(Ordering::Acquire)
    }

    /// Atomic store of pointer value
    ///
    /// # Memory Ordering
    /// Uses `Ordering::Release` to ensure all prior writes
    /// are visible before the pointer update.
    pub fn store_atomic(ptr: &AtomicUsize, value: usize) {
        ptr.store(value, Ordering::Release);
    }

    /// Atomic compare-and-swap for pointer update
    ///
    /// Updates pointer only if current value equals `expected`.
    /// Returns `Ok(new_value)` if successful, `Err(current_value)` if not.
    ///
    /// # Use Case
    /// Pointer healing in load barrier - update pointer to new
    /// address only if it hasn't been updated by another thread.
    ///
    /// # Errors
    /// Returns `FgcError::AtomicUpdateFailed` if CAS fails (value changed)
    pub fn cas_atomic(
        ptr: &AtomicUsize,
        expected: usize,
        new: usize,
    ) -> crate::error::Result<usize> {
        match ptr.compare_exchange(expected, new, Ordering::AcqRel, Ordering::Acquire) {
            Ok(val) => Ok(val),
            Err(current) => {
                // CAS failed - value was changed by another thread
                // Return the current value so caller can retry
                Err(crate::error::FgcError::AtomicUpdateFailed(current))
            }
        }
    }

    /// Flip Marked0 <-> Marked1 untuk GC cycle baru
    ///
    /// Dipanggil saat GC cycle berganti (even -> odd atau sebaliknya).
    /// Ini menghindari kebutuhan untuk clear mark bits di awal cycle.
    ///
    /// # Behavior
    /// - Marked0=1, Marked1=0 → Marked0=0, Marked1=1 (swap)
    /// - Marked0=0, Marked1=1 → Marked0=1, Marked1=0 (swap)
    /// - Marked0=0, Marked1=0 → Marked0=1, Marked1=0 (set Marked0 as default)
    /// - Marked0=1, Marked1=1 → Marked0=0, Marked1=1 (treated as Marked0 set, swaps to Marked1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut ptr = ColoredPointer::new(0x1000);
    /// ptr.set_marked0();
    /// assert!(ptr.is_marked0());
    ///
    /// ptr.flip_mark_bit();
    /// assert!(!ptr.is_marked0());
    /// assert!(ptr.is_marked1());
    /// ```
    pub fn flip_mark_bit(&mut self) {
        let marked0 = self.is_marked0();
        let marked1 = self.is_marked1();

        // Clear both mark bits first
        self.raw &= !(Self::MARKED0_MASK | Self::MARKED1_MASK);

        // Swap (not flip!) - set the opposite of what was set
        // If Marked0 was set, now set Marked1, and vice versa
        if marked0 {
            self.raw |= Self::MARKED1_MASK;
        } else if marked1 {
            self.raw |= Self::MARKED0_MASK;
        } else {
            // Neither was set - set Marked0 as default (start of new cycle)
            // This ensures exactly one mark bit is always set after flip
            self.raw |= Self::MARKED0_MASK;
        }
    }

    /// Get raw value untuk pointer
    ///
    /// Untuk debugging dan low-level operations.
    pub fn raw(&self) -> usize {
        self.raw
    }

    /// Create colored pointer dari raw value
    ///
    /// Tidak melakukan validation, gunakan dengan hati-hati.
    pub fn from_raw(raw: usize) -> Self {
        Self { raw }
    }

    /// Check jika pointer perlu processing oleh load barrier
    ///
    /// Pointer perlu processing jika:
    /// - Belum marked (saat marking phase)
    /// - Di relocation set (saat relocating phase)
    pub fn needs_processing(&self, gc_phase: GcPhase) -> bool {
        match gc_phase {
            GcPhase::Marking => !self.is_marked(),
            GcPhase::Relocating => !self.is_remapped(),
            _ => false,
        }
    }
}

/// GC Phase untuk menentukan load barrier behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcPhase {
    /// Idle - tidak ada GC berjalan
    Idle,
    /// Marking phase
    Marking,
    /// Relocating phase
    Relocating,
    /// Cleanup phase
    Cleanup,
}

/// Color - enum untuk tipe color bit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// No color set (remapped view)
    None,
    /// Marked0 bit set (GC cycle even)
    Marked0,
    /// Marked1 bit set (GC cycle odd)
    Marked1,
    /// Remapped bit set
    Remapped,
    /// Finalizable bit set
    Finalizable,
}

impl From<Color> for usize {
    /// Convert Color ke mask value
    fn from(color: Color) -> usize {
        match color {
            Color::None => 0,
            Color::Marked0 => ColoredPointer::MARKED0_MASK,
            Color::Marked1 => ColoredPointer::MARKED1_MASK,
            Color::Remapped => ColoredPointer::REMAPPED_MASK,
            Color::Finalizable => ColoredPointer::FINALIZABLE_MASK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pointer() {
        let ptr = ColoredPointer::new(0x1234);
        assert_eq!(ptr.address(), 0x1234);
        assert!(!ptr.is_marked());
        assert!(!ptr.is_remapped());
    }

    #[test]
    fn test_set_marked() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        assert!(ptr.is_marked0());
        assert!(ptr.is_marked());
        assert!(!ptr.is_marked1());
    }

    #[test]
    fn test_flip_mark_bit() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        assert!(ptr.is_marked0());
        
        ptr.flip_mark_bit();
        assert!(!ptr.is_marked0());
        assert!(ptr.is_marked1());
    }

    #[test]
    fn test_clear_color() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.set_remapped();
        
        ptr.clear_color();
        assert!(!ptr.is_marked());
        assert!(!ptr.is_remapped());
        assert_eq!(ptr.address(), 0x1234);
    }
}
