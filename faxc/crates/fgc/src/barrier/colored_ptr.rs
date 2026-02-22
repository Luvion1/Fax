//! Colored Pointer Implementation

use std::sync::atomic::{AtomicUsize, Ordering};

/// ColoredPointer - wrapper for pointer with metadata bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColoredPointer {
    raw: usize,
}

impl ColoredPointer {
    pub const MARKED0_MASK: usize = 1 << 44;
    pub const MARKED1_MASK: usize = 1 << 45;
    pub const REMAPPED_MASK: usize = 1 << 46;
    pub const FINALIZABLE_MASK: usize = 1 << 47;
    #[allow(dead_code)]
    const COLOR_MASK: usize =
        Self::MARKED0_MASK | Self::MARKED1_MASK | Self::REMAPPED_MASK | Self::FINALIZABLE_MASK;
    pub const ADDRESS_MASK: usize = (1 << 44) - 1;

    pub fn new(address: usize) -> Self {
        Self {
            raw: address & Self::ADDRESS_MASK,
        }
    }

    pub fn address(&self) -> usize {
        self.raw & Self::ADDRESS_MASK
    }

    pub fn is_marked0(&self) -> bool {
        (self.raw & Self::MARKED0_MASK) != 0
    }

    pub fn is_marked1(&self) -> bool {
        (self.raw & Self::MARKED1_MASK) != 0
    }

    pub fn is_marked(&self) -> bool {
        self.is_marked0() || self.is_marked1()
    }

    pub fn is_remapped(&self) -> bool {
        (self.raw & Self::REMAPPED_MASK) != 0
    }

    pub fn is_finalizable(&self) -> bool {
        (self.raw & Self::FINALIZABLE_MASK) != 0
    }

    pub fn set_marked0(&mut self) {
        self.raw |= Self::MARKED0_MASK;
    }

    pub fn set_marked1(&mut self) {
        self.raw |= Self::MARKED1_MASK;
    }

    pub fn set_remapped(&mut self) {
        self.raw |= Self::REMAPPED_MASK;
    }

    pub fn set_finalizable(&mut self) {
        self.raw |= Self::FINALIZABLE_MASK;
    }

    pub fn clear_color(&mut self) {
        self.raw &= Self::ADDRESS_MASK;
    }

    pub fn set_marked0_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::MARKED0_MASK, Ordering::Acquire);
    }

    pub fn set_marked1_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::MARKED1_MASK, Ordering::Acquire);
    }

    pub fn set_remapped_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::REMAPPED_MASK, Ordering::Acquire);
    }

    pub fn set_finalizable_atomic(ptr: &AtomicUsize) {
        ptr.fetch_or(Self::FINALIZABLE_MASK, Ordering::Acquire);
    }

    pub fn clear_color_atomic(ptr: &AtomicUsize) {
        ptr.fetch_and(Self::ADDRESS_MASK, Ordering::Release);
    }

    pub fn flip_mark_bit_atomic(ptr: &AtomicUsize) {
        ptr.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            let marked0 = (current & Self::MARKED0_MASK) != 0;
            let marked1 = (current & Self::MARKED1_MASK) != 0;
            let mut new_value = current & !(Self::MARKED0_MASK | Self::MARKED1_MASK);
            if marked0 {
                new_value |= Self::MARKED1_MASK;
            } else if marked1 {
                new_value |= Self::MARKED0_MASK;
            } else {
                new_value |= Self::MARKED0_MASK;
            }
            Some(new_value)
        })
        .ok();
    }

    pub fn test_and_set_marked0(ptr: &AtomicUsize) -> bool {
        let old = ptr.fetch_or(Self::MARKED0_MASK, Ordering::AcqRel);
        (old & Self::MARKED0_MASK) != 0
    }

    pub fn test_and_set_marked1(ptr: &AtomicUsize) -> bool {
        let old = ptr.fetch_or(Self::MARKED1_MASK, Ordering::AcqRel);
        (old & Self::MARKED1_MASK) != 0
    }

    pub fn load_atomic(ptr: &AtomicUsize) -> usize {
        ptr.load(Ordering::Acquire)
    }

    pub fn store_atomic(ptr: &AtomicUsize, value: usize) {
        ptr.store(value, Ordering::Release);
    }

    pub fn cas_atomic(
        ptr: &AtomicUsize,
        expected: usize,
        new: usize,
    ) -> crate::error::Result<usize> {
        match ptr.compare_exchange(expected, new, Ordering::AcqRel, Ordering::Acquire) {
            Ok(val) => Ok(val),
            Err(current) => Err(crate::error::FgcError::AtomicUpdateFailed(current)),
        }
    }

    pub fn flip_mark_bit(&mut self) {
        let marked0 = self.is_marked0();
        let marked1 = self.is_marked1();
        self.raw &= !(Self::MARKED0_MASK | Self::MARKED1_MASK);
        if marked0 {
            self.raw |= Self::MARKED1_MASK;
        } else if marked1 {
            self.raw |= Self::MARKED0_MASK;
        } else {
            self.raw |= Self::MARKED0_MASK;
        }
    }

    pub fn raw(&self) -> usize {
        self.raw
    }

    pub fn from_raw(raw: usize) -> Self {
        Self { raw }
    }

    pub fn needs_processing(&self, gc_phase: GcPhase) -> bool {
        match gc_phase {
            GcPhase::Marking => !self.is_marked(),
            GcPhase::Relocating => !self.is_remapped(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcPhase {
    Idle,
    Marking,
    Relocating,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    None,
    Marked0,
    Marked1,
    Remapped,
    Finalizable,
}

impl From<Color> for usize {
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

    // ========================================================================
    // Basic Constructor and Accessor Tests
    // ========================================================================

    #[test]
    fn test_new_pointer() {
        let ptr = ColoredPointer::new(0x1234);
        assert_eq!(ptr.address(), 0x1234);
        assert!(!ptr.is_marked());
        assert!(!ptr.is_remapped());
        assert!(!ptr.is_finalizable());
    }

    #[test]
    fn test_new_pointer_masks_high_bits() {
        let ptr = ColoredPointer::new(0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(ptr.address(), 0x0FFF_FFFF_FFFF);
    }

    #[test]
    fn test_from_raw() {
        let raw = 0x1234 | ColoredPointer::MARKED0_MASK;
        let ptr = ColoredPointer::from_raw(raw);
        assert_eq!(ptr.raw(), raw);
        assert_eq!(ptr.address(), 0x1234);
        assert!(ptr.is_marked0());
    }

    // ========================================================================
    // Mark Bit Tests
    // ========================================================================

    #[test]
    fn test_set_marked0() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        assert!(ptr.is_marked0());
        assert!(ptr.is_marked());
        assert!(!ptr.is_marked1());
    }

    #[test]
    fn test_set_marked1() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked1();
        assert!(ptr.is_marked1());
        assert!(ptr.is_marked());
        assert!(!ptr.is_marked0());
    }

    #[test]
    fn test_both_mark_bits_can_be_set() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.set_marked1();
        assert!(ptr.is_marked0());
        assert!(ptr.is_marked1());
        assert!(ptr.is_marked());
    }

    #[test]
    fn test_flip_mark_bit_from_marked0() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.flip_mark_bit();
        assert!(!ptr.is_marked0());
        assert!(ptr.is_marked1());
    }

    #[test]
    fn test_flip_mark_bit_from_marked1() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked1();
        ptr.flip_mark_bit();
        assert!(ptr.is_marked0());
        assert!(!ptr.is_marked1());
    }

    #[test]
    fn test_flip_mark_bit_from_neither() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.flip_mark_bit();
        assert!(ptr.is_marked0());
        assert!(!ptr.is_marked1());
    }

    #[test]
    fn test_flip_mark_bit_from_both() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.set_marked1();
        ptr.flip_mark_bit();
        assert!(!ptr.is_marked0());
        assert!(ptr.is_marked1());
    }

    #[test]
    fn test_flip_mark_bit_double_flip() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.flip_mark_bit();
        ptr.flip_mark_bit();
        assert!(ptr.is_marked0());
        assert!(!ptr.is_marked1());
    }

    // ========================================================================
    // Other Color Bit Tests
    // ========================================================================

    #[test]
    fn test_set_remapped() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_remapped();
        assert!(ptr.is_remapped());
    }

    #[test]
    fn test_set_finalizable() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_finalizable();
        assert!(ptr.is_finalizable());
    }

    #[test]
    fn test_all_color_bits_independent() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.set_remapped();
        ptr.set_finalizable();

        assert!(ptr.is_marked0());
        assert!(ptr.is_remapped());
        assert!(ptr.is_finalizable());
        assert_eq!(ptr.address(), 0x1234);
    }

    #[test]
    fn test_clear_color() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        ptr.set_remapped();
        ptr.set_finalizable();

        ptr.clear_color();
        assert!(!ptr.is_marked());
        assert!(!ptr.is_remapped());
        assert!(!ptr.is_finalizable());
        assert_eq!(ptr.address(), 0x1234);
    }

    // ========================================================================
    // Atomic Operations Tests
    // ========================================================================

    #[test]
    fn test_set_marked0_atomic() {
        let atomic = AtomicUsize::new(0x1234);
        ColoredPointer::set_marked0_atomic(&atomic);
        let ptr = ColoredPointer::from_raw(atomic.load(Ordering::Relaxed));
        assert!(ptr.is_marked0());
    }

    #[test]
    fn test_set_marked1_atomic() {
        let atomic = AtomicUsize::new(0x1234);
        ColoredPointer::set_marked1_atomic(&atomic);
        let ptr = ColoredPointer::from_raw(atomic.load(Ordering::Relaxed));
        assert!(ptr.is_marked1());
    }

    #[test]
    fn test_set_remapped_atomic() {
        let atomic = AtomicUsize::new(0x1234);
        ColoredPointer::set_remapped_atomic(&atomic);
        let ptr = ColoredPointer::from_raw(atomic.load(Ordering::Relaxed));
        assert!(ptr.is_remapped());
    }

    #[test]
    fn test_clear_color_atomic() {
        let atomic =
            AtomicUsize::new(0x1234 | ColoredPointer::MARKED0_MASK | ColoredPointer::REMAPPED_MASK);
        ColoredPointer::clear_color_atomic(&atomic);
        let ptr = ColoredPointer::from_raw(atomic.load(Ordering::Relaxed));
        assert!(!ptr.is_marked());
        assert!(!ptr.is_remapped());
    }

    #[test]
    fn test_flip_mark_bit_atomic() {
        let atomic = AtomicUsize::new(0x1234 | ColoredPointer::MARKED0_MASK);
        ColoredPointer::flip_mark_bit_atomic(&atomic);
        let ptr = ColoredPointer::from_raw(atomic.load(Ordering::Relaxed));
        assert!(!ptr.is_marked0());
        assert!(ptr.is_marked1());
    }

    #[test]
    fn test_test_and_set_marked0() {
        let atomic = AtomicUsize::new(0x1234);
        let first = ColoredPointer::test_and_set_marked0(&atomic);
        let second = ColoredPointer::test_and_set_marked0(&atomic);

        assert!(!first); // First call sets the bit
        assert!(second); // Second call sees it was already set
    }

    #[test]
    fn test_load_store_atomic() {
        let atomic = AtomicUsize::new(0x1234);
        let value = ColoredPointer::load_atomic(&atomic);
        assert_eq!(value, 0x1234);

        ColoredPointer::store_atomic(&atomic, 0x5678);
        assert_eq!(ColoredPointer::load_atomic(&atomic), 0x5678);
    }

    #[test]
    fn test_cas_atomic_success() {
        let atomic = AtomicUsize::new(0x1234);
        let result = ColoredPointer::cas_atomic(&atomic, 0x1234, 0x5678);
        assert!(result.is_ok());
        assert_eq!(atomic.load(Ordering::Relaxed), 0x5678);
    }

    #[test]
    fn test_cas_atomic_failure() {
        let atomic = AtomicUsize::new(0x1234);
        let result = ColoredPointer::cas_atomic(&atomic, 0x9999, 0x5678);
        assert!(result.is_err());
        assert_eq!(atomic.load(Ordering::Relaxed), 0x1234);
    }

    // ========================================================================
    // GC Phase Tests
    // ========================================================================

    #[test]
    fn test_needs_processing_marking_unmarked() {
        let ptr = ColoredPointer::new(0x1234);
        assert!(ptr.needs_processing(GcPhase::Marking));
    }

    #[test]
    fn test_needs_processing_marking_marked() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_marked0();
        assert!(!ptr.needs_processing(GcPhase::Marking));
    }

    #[test]
    fn test_needs_processing_relocating_not_remapped() {
        let ptr = ColoredPointer::new(0x1234);
        assert!(ptr.needs_processing(GcPhase::Relocating));
    }

    #[test]
    fn test_needs_processing_relocating_remapped() {
        let mut ptr = ColoredPointer::new(0x1234);
        ptr.set_remapped();
        assert!(!ptr.needs_processing(GcPhase::Relocating));
    }

    #[test]
    fn test_needs_processing_idle() {
        let ptr = ColoredPointer::new(0x1234);
        assert!(!ptr.needs_processing(GcPhase::Idle));
    }

    #[test]
    fn test_needs_processing_cleanup() {
        let ptr = ColoredPointer::new(0x1234);
        assert!(!ptr.needs_processing(GcPhase::Cleanup));
    }

    // ========================================================================
    // Color Enum Tests
    // ========================================================================

    #[test]
    fn test_color_from_enum() {
        assert_eq!(usize::from(Color::None), 0);
        assert_eq!(usize::from(Color::Marked0), ColoredPointer::MARKED0_MASK);
        assert_eq!(usize::from(Color::Marked1), ColoredPointer::MARKED1_MASK);
        assert_eq!(usize::from(Color::Remapped), ColoredPointer::REMAPPED_MASK);
        assert_eq!(
            usize::from(Color::Finalizable),
            ColoredPointer::FINALIZABLE_MASK
        );
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_zero_address() {
        let ptr = ColoredPointer::new(0);
        assert_eq!(ptr.address(), 0);
        assert!(!ptr.is_marked());
    }

    #[test]
    fn test_max_address() {
        let ptr = ColoredPointer::new(ColoredPointer::ADDRESS_MASK);
        assert_eq!(ptr.address(), ColoredPointer::ADDRESS_MASK);
    }

    #[test]
    fn test_concurrent_mark_operations() {
        use std::sync::Arc;
        use std::thread;

        let atomic = Arc::new(AtomicUsize::new(0x1234));
        let mut handles = Vec::new();

        for _ in 0..4 {
            let atomic = Arc::clone(&atomic);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    ColoredPointer::set_marked0_atomic(&atomic);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let ptr = ColoredPointer::from_raw(atomic.load(Ordering::Relaxed));
        assert!(ptr.is_marked0());
        assert_eq!(ptr.address(), 0x1234);
    }
}
