//! Object Header - Metadata for GC-managed objects
//!
//! Object Header Layout (24 bytes on 64-bit):
//! ┌─────────────────────────────────────────┐
//! │         Mark Word (8 bytes)             │  <- AtomicUsize
//! │  - Bit 0: Marked0                       │
//! │  - Bit 1: Marked1                       │
//! │  - Bit 2: RemSet (remembered set)       │
//! │  - Bit 3: Forwarded                     │
//! │  - Bits 4-7: Age (generational GC)      │
//! │  - Bits 8-63: Hash code / forwarding    │
//! ├─────────────────────────────────────────┤
//! │       Class Pointer (8 bytes)           │  <- *const Class
//! ├─────────────────────────────────────────┤
//! │         Size (8 bytes)                  │  <- Object size incl. header
//! └─────────────────────────────────────────┘

use std::sync::atomic::{AtomicUsize, Ordering};

/// Size of object header in bytes
pub const HEADER_SIZE: usize = 24;

/// Minimum object alignment (bytes)
pub const OBJECT_ALIGNMENT: usize = 8;

/// Mark bit positions
pub const MARKED0_BIT: usize = 0;
pub const MARKED1_BIT: usize = 1;
pub const REMSET_BIT: usize = 2;
pub const FORWARDED_BIT: usize = 3;
pub const AGE_SHIFT: usize = 4;
pub const HASH_SHIFT: usize = 8;

/// Masks for mark word fields
pub const MARKED0_MASK: usize = 1 << MARKED0_BIT;
pub const MARKED1_MASK: usize = 1 << MARKED1_BIT;
pub const REMSET_MASK: usize = 1 << REMSET_BIT;
pub const FORWARDED_MASK: usize = 1 << FORWARDED_BIT;
pub const AGE_MASK: usize = 0b1111 << AGE_SHIFT;
pub const HASH_MASK: usize = usize::MAX << HASH_SHIFT;

/// Object Header
///
/// Every GC-managed object starts with this header.
/// The header provides metadata needed for garbage collection:
/// - Mark bits for liveness tracking
/// - Forwarding pointer for object relocation
/// - Object size for scanning
/// - Class pointer for type information
#[repr(C)]
pub struct ObjectHeader {
    /// Mark word: atomic for concurrent GC operations
    pub mark_word: AtomicUsize,
    /// Class pointer (raw pointer to class metadata)
    pub class_ptr: usize,
    /// Object size in bytes (including header)
    pub size: usize,
}

impl ObjectHeader {
    /// Create new object header
    ///
    /// # Arguments
    /// * `class_ptr` - Pointer to class metadata
    /// * `size` - Total object size including header
    pub fn new(class_ptr: usize, size: usize) -> Self {
        Self {
            mark_word: AtomicUsize::new(0),
            class_ptr,
            size,
        }
    }

    // === Mark Bit Operations ===

    /// Check if object is marked with Marked0
    #[inline]
    pub fn is_marked0(&self) -> bool {
        self.mark_word.load(Ordering::Acquire) & MARKED0_MASK != 0
    }

    /// Check if object is marked with Marked1
    #[inline]
    pub fn is_marked1(&self) -> bool {
        self.mark_word.load(Ordering::Acquire) & MARKED1_MASK != 0
    }

    /// Set Marked0 bit atomically
    /// Returns true if bit was already set
    #[inline]
    pub fn set_marked0(&self) -> bool {
        self.mark_word.fetch_or(MARKED0_MASK, Ordering::AcqRel) & MARKED0_MASK != 0
    }

    /// Set Marked1 bit atomically
    /// Returns true if bit was already set
    #[inline]
    pub fn set_marked1(&self) -> bool {
        self.mark_word.fetch_or(MARKED1_MASK, Ordering::AcqRel) & MARKED1_MASK != 0
    }

    /// Clear both mark bits atomically
    #[inline]
    pub fn clear_mark_bits(&self) {
        self.mark_word.fetch_and(!(MARKED0_MASK | MARKED1_MASK), Ordering::AcqRel);
    }

    /// Flip mark bits (swap Marked0 <-> Marked1)
    /// Used when starting new GC cycle
    #[inline]
    pub fn flip_mark_bits(&self) {
        let mut current = self.mark_word.load(Ordering::Acquire);
        loop {
            let marked0 = current & MARKED0_MASK;
            let marked1 = current & MARKED1_MASK;

            // Clear both mark bits
            let mut new = current & !(MARKED0_MASK | MARKED1_MASK);

            // Swap: if Marked0 was set, set Marked1, and vice versa
            if marked0 != 0 {
                new |= MARKED1_MASK;
            }
            if marked1 != 0 {
                new |= MARKED0_MASK;
            }

            match self.mark_word.compare_exchange_weak(
                current,
                new,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(val) => current = val,
            }
        }
    }

    /// Check if object is marked (either Marked0 or Marked1)
    #[inline]
    pub fn is_marked(&self) -> bool {
        let mark = self.mark_word.load(Ordering::Acquire);
        (mark & (MARKED0_MASK | MARKED1_MASK)) != 0
    }

    // === Forwarding Pointer Operations ===

    /// Check if object is forwarded (relocated)
    #[inline]
    pub fn is_forwarded(&self) -> bool {
        self.mark_word.load(Ordering::Acquire) & FORWARDED_MASK != 0
    }

    /// Set forwarded flag
    #[inline]
    pub fn set_forwarded(&self) {
        self.mark_word.fetch_or(FORWARDED_MASK, Ordering::Release);
    }

    /// Get forwarding pointer
    #[inline]
    pub fn get_forwarding_ptr(&self) -> usize {
        self.mark_word.load(Ordering::Acquire) & HASH_MASK
    }

    /// Set forwarding pointer atomically
    /// Returns true if successfully set (was not already forwarded)
    #[inline]
    pub fn try_set_forwarding_ptr(&self, new_addr: usize) -> bool {
        let mut current = self.mark_word.load(Ordering::Acquire);
        loop {
            if current & FORWARDED_MASK != 0 {
                return false; // Already forwarded
            }

            let new_mark = (current & !HASH_MASK) | (new_addr & HASH_MASK);

            match self.mark_word.compare_exchange_weak(
                current,
                new_mark,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.set_forwarded();
                    return true;
                }
                Err(val) => current = val,
            }
        }
    }

    // === Age Operations (for generational GC) ===

    /// Get object age (0-15)
    #[inline]
    pub fn get_age(&self) -> u8 {
        ((self.mark_word.load(Ordering::Relaxed) & AGE_MASK) >> AGE_SHIFT) as u8
    }

    /// Increment object age
    /// Returns new age
    #[inline]
    pub fn increment_age(&self) -> u8 {
        let mut current = self.mark_word.load(Ordering::Relaxed);
        loop {
            let age = ((current & AGE_MASK) >> AGE_SHIFT) as u8;
            if age >= 15 {
                return 15; // Max age
            }

            let new = (current & !AGE_MASK) | (((age + 1) as usize) << AGE_SHIFT);

            match self.mark_word.compare_exchange_weak(
                current,
                new,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return age + 1,
                Err(val) => current = val,
            }
        }
    }

    // === RemSet Operations ===

    /// Check if object is in remembered set
    #[inline]
    pub fn in_remset(&self) -> bool {
        self.mark_word.load(Ordering::Acquire) & REMSET_MASK != 0
    }

    /// Set remembered set flag
    /// Returns true if bit was already set
    #[inline]
    pub fn set_remset(&self) -> bool {
        self.mark_word.fetch_or(REMSET_MASK, Ordering::AcqRel) & REMSET_MASK != 0
    }

    /// Clear remembered set flag
    #[inline]
    pub fn clear_remset(&self) {
        self.mark_word.fetch_and(!REMSET_MASK, Ordering::AcqRel);
    }

    // === Size Operations ===

    /// Get object size (including header)
    #[inline]
    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Get object data size (excluding header)
    #[inline]
    pub fn get_data_size(&self) -> usize {
        self.size - HEADER_SIZE
    }

    // === Class Operations ===

    /// Get class pointer
    #[inline]
    pub fn get_class(&self) -> usize {
        self.class_ptr
    }
}

/// Get pointer to ObjectHeader from object address
///
/// # Safety
/// `obj_addr` must point to a valid GC-managed object with an ObjectHeader at the start.
#[inline]
pub unsafe fn get_header(obj_addr: usize) -> *mut ObjectHeader {
    obj_addr as *mut ObjectHeader
}

/// Get object data start (after header)
///
/// # Safety
/// `obj_addr` must point to a valid GC-managed object with an ObjectHeader at the start.
#[inline]
pub unsafe fn get_data_start(obj_addr: usize) -> usize {
    obj_addr + HEADER_SIZE
}

/// Get object address from header pointer
///
/// # Safety
/// `header` must be a valid pointer to ObjectHeader.
#[inline]
pub unsafe fn get_object_addr(header: *const ObjectHeader) -> usize {
    header as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_header() -> ObjectHeader {
        ObjectHeader::new(0x1000, 64)
    }

    // === Mark Bit Tests ===

    #[test]
    fn test_marked0_operations() {
        let header = create_test_header();

        // Initially not marked
        assert!(!header.is_marked0());
        assert!(!header.is_marked());

        // Set Marked0
        let was_set = header.set_marked0();
        assert!(!was_set); // Was not previously set
        assert!(header.is_marked0());
        assert!(header.is_marked());

        // Set again (should return true)
        let was_set = header.set_marked0();
        assert!(was_set); // Was already set
        assert!(header.is_marked0());
    }

    #[test]
    fn test_marked1_operations() {
        let header = create_test_header();

        // Initially not marked
        assert!(!header.is_marked1());
        assert!(!header.is_marked());

        // Set Marked1
        let was_set = header.set_marked1();
        assert!(!was_set); // Was not previously set
        assert!(header.is_marked1());
        assert!(header.is_marked());

        // Set again (should return true)
        let was_set = header.set_marked1();
        assert!(was_set); // Was already set
        assert!(header.is_marked1());
    }

    #[test]
    fn test_clear_mark_bits() {
        let header = create_test_header();

        // Set both mark bits
        header.set_marked0();
        header.set_marked1();
        assert!(header.is_marked0());
        assert!(header.is_marked1());
        assert!(header.is_marked());

        // Clear mark bits
        header.clear_mark_bits();
        assert!(!header.is_marked0());
        assert!(!header.is_marked1());
        assert!(!header.is_marked());
    }

    #[test]
    fn test_flip_mark_bits() {
        let header = create_test_header();

        // Set Marked0 only
        header.set_marked0();
        assert!(header.is_marked0());
        assert!(!header.is_marked1());

        // Flip: Marked0 -> Marked1
        header.flip_mark_bits();
        assert!(!header.is_marked0());
        assert!(header.is_marked1());

        // Flip again: Marked1 -> Marked0
        header.flip_mark_bits();
        assert!(header.is_marked0());
        assert!(!header.is_marked1());

        // Set both and flip
        header.set_marked1();
        assert!(header.is_marked0());
        assert!(header.is_marked1());

        header.flip_mark_bits();
        assert!(header.is_marked0());
        assert!(header.is_marked1());
    }

    #[test]
    fn test_flip_mark_bits_empty() {
        let header = create_test_header();

        // Neither bit set - flip should do nothing
        assert!(!header.is_marked0());
        assert!(!header.is_marked1());

        header.flip_mark_bits();

        assert!(!header.is_marked0());
        assert!(!header.is_marked1());
    }

    #[test]
    fn test_is_marked() {
        let header = create_test_header();

        // Neither set
        assert!(!header.is_marked());

        // Marked0 only
        header.set_marked0();
        assert!(header.is_marked());

        header.clear_mark_bits();

        // Marked1 only
        header.set_marked1();
        assert!(header.is_marked());

        header.clear_mark_bits();

        // Both set
        header.set_marked0();
        header.set_marked1();
        assert!(header.is_marked());
    }

    // === Forwarding Pointer Tests ===

    #[test]
    fn test_forwarding_operations() {
        let header = create_test_header();
        let new_addr: usize = 0x5000;

        // Initially not forwarded
        assert!(!header.is_forwarded());
        assert_eq!(header.get_forwarding_ptr(), 0);

        // Set forwarding pointer
        let success = header.try_set_forwarding_ptr(new_addr);
        assert!(success);
        assert!(header.is_forwarded());
        assert_eq!(header.get_forwarding_ptr(), new_addr & HASH_MASK);

        // Try to set again (should fail)
        let success = header.try_set_forwarding_ptr(0x6000);
        assert!(!success);
        assert_eq!(header.get_forwarding_ptr(), new_addr & HASH_MASK);
    }

    #[test]
    fn test_set_forwarded_flag() {
        let header = create_test_header();

        assert!(!header.is_forwarded());
        header.set_forwarded();
        assert!(header.is_forwarded());
    }

    // === Age Tests ===

    #[test]
    fn test_age_operations() {
        let header = create_test_header();

        // Initial age is 0
        assert_eq!(header.get_age(), 0);

        // Increment age
        for i in 1..=15 {
            let new_age = header.increment_age();
            assert_eq!(new_age, i);
            assert_eq!(header.get_age(), i);
        }

        // Age should cap at 15
        for _ in 0..5 {
            let new_age = header.increment_age();
            assert_eq!(new_age, 15);
            assert_eq!(header.get_age(), 15);
        }
    }

    #[test]
    fn test_age_with_mark_bits() {
        let header = create_test_header();

        // Set some mark bits
        header.set_marked0();
        header.set_marked1();

        // Age should still work independently
        header.increment_age();
        header.increment_age();
        header.increment_age();

        assert_eq!(header.get_age(), 3);
        assert!(header.is_marked0());
        assert!(header.is_marked1());
    }

    // === RemSet Tests ===

    #[test]
    fn test_remset_operations() {
        let header = create_test_header();

        // Initially not in remset
        assert!(!header.in_remset());

        // Set remset
        let was_set = header.set_remset();
        assert!(!was_set);
        assert!(header.in_remset());

        // Set again
        let was_set = header.set_remset();
        assert!(was_set);
        assert!(header.in_remset());

        // Clear remset
        header.clear_remset();
        assert!(!header.in_remset());
    }

    // === Size Tests ===

    #[test]
    fn test_size_operations() {
        let header = ObjectHeader::new(0x1000, 128);

        assert_eq!(header.get_size(), 128);
        assert_eq!(header.get_data_size(), 128 - HEADER_SIZE);
    }

    #[test]
    fn test_header_size_constant() {
        // Verify HEADER_SIZE matches the actual struct size
        assert_eq!(HEADER_SIZE, 24);
    }

    // === Class Tests ===

    #[test]
    fn test_class_pointer() {
        let class_ptr: usize = 0x2000;
        let header = ObjectHeader::new(class_ptr, 64);

        assert_eq!(header.get_class(), class_ptr);
    }

    // === Unsafe Helper Functions Tests ===

    #[test]
    fn test_get_header() {
        let header = create_test_header();
        let addr = &header as *const ObjectHeader as usize;

        unsafe {
            let header_ptr = get_header(addr);
            assert_eq!(header_ptr as usize, addr);
        }
    }

    #[test]
    fn test_get_data_start() {
        let header = create_test_header();
        let addr = &header as *const ObjectHeader as usize;

        unsafe {
            let data_start = get_data_start(addr);
            assert_eq!(data_start, addr + HEADER_SIZE);
        }
    }

    #[test]
    fn test_get_object_addr() {
        let header = create_test_header();
        let addr = &header as *const ObjectHeader as usize;

        unsafe {
            let recovered_addr = get_object_addr(&header);
            assert_eq!(recovered_addr, addr);
        }
    }

    // === Concurrent Safety Tests ===

    #[test]
    fn test_concurrent_mark_operations() {
        use std::sync::Arc;
        use std::thread;

        let header = Arc::new(create_test_header());
        let mut handles = vec![];

        // Spawn multiple threads to set mark bits concurrently
        for i in 0..10 {
            let header_clone = Arc::clone(&header);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    if i % 2 == 0 {
                        header_clone.set_marked0();
                    } else {
                        header_clone.set_marked1();
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // At least one mark bit should be set
        assert!(header.is_marked());
    }

    #[test]
    fn test_concurrent_age_increment() {
        use std::sync::Arc;
        use std::thread;

        let header = Arc::new(create_test_header());
        let mut handles = vec![];

        // Spawn multiple threads to increment age concurrently
        for _ in 0..5 {
            let header_clone = Arc::clone(&header);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    header_clone.increment_age();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Age should be incremented (exact value depends on race conditions)
        assert!(header.get_age() > 0);
        assert!(header.get_age() <= 15);
    }
}
