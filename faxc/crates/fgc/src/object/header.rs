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
//!
//! # Memory Ordering Model
//!
//! This module uses the following atomic ordering strategy:
//!
//! ## Mark Word Operations
//! - **Load:** `Ordering::Acquire` - Must see prior writes to object state
//! - **Store:** `Ordering::Release` - Must be visible to other threads
//! - **CAS:** `Ordering::AcqRel` - Read-modify-write operation
//!
//! ## Reference Count / Age
//! - **Load:** `Ordering::Relaxed` - Only need eventual consistency
//! - **Store:** `Ordering::Relaxed` - No ordering required
//! - **CAS:** `Ordering::AcqRel` - Must be atomic
//!
//! ## State Flags
//! - **Load:** `Ordering::Acquire` - Must see consistent state
//! - **Store:** `Ordering::Release` - State change must be visible
//!
//! ## Rationale
//!
//! The mark word is part of the GC safepoint protocol and requires
//! stronger ordering to ensure:
//! 1. Mark state is visible to all GC threads
//! 2. Object modifications happen-before mark check
//! 3. No lost updates during concurrent marking
//!
//! Age operations use `Relaxed` ordering since they are statistics
//! and only need eventual consistency for correctness.
//!
//! ## Thread Safety
//!
//! All mark bit operations are atomic and thread-safe. Multiple threads can
//! concurrently mark objects without race conditions.

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
    ///
    /// Uses Acquire ordering to ensure we see prior writes from other threads.
    #[inline]
    pub fn is_marked0(&self) -> bool {
        // Acquire: must see prior writes to mark word from other threads
        self.mark_word.load(Ordering::Acquire) & MARKED0_MASK != 0
    }

    /// Check if object is marked with Marked1
    ///
    /// Uses Acquire ordering to ensure we see prior writes from other threads.
    #[inline]
    pub fn is_marked1(&self) -> bool {
        // Acquire: must see prior writes to mark word from other threads
        self.mark_word.load(Ordering::Acquire) & MARKED1_MASK != 0
    }

    /// Set Marked0 bit atomically
    /// Returns true if bit was already set
    ///
    /// Uses AcqRel ordering for read-modify-write operation.
    #[inline]
    pub fn set_marked0(&self) -> bool {
        // AcqRel: combines Acquire (see prior writes) + Release (make our write visible)
        self.mark_word.fetch_or(MARKED0_MASK, Ordering::AcqRel) & MARKED0_MASK != 0
    }

    /// Set Marked1 bit atomically
    /// Returns true if bit was already set
    ///
    /// Uses AcqRel ordering for read-modify-write operation.
    #[inline]
    pub fn set_marked1(&self) -> bool {
        // AcqRel: combines Acquire (see prior writes) + Release (make our write visible)
        self.mark_word.fetch_or(MARKED1_MASK, Ordering::AcqRel) & MARKED1_MASK != 0
    }

    /// Clear both mark bits atomically
    ///
    /// Uses AcqRel ordering for read-modify-write operation.
    #[inline]
    pub fn clear_mark_bits(&self) {
        // AcqRel: atomic read-modify-write for clearing bits
        self.mark_word
            .fetch_and(!(MARKED0_MASK | MARKED1_MASK), Ordering::AcqRel);
    }

    /// Flip mark bits (swap Marked0 <-> Marked1)
    /// Used when starting new GC cycle
    ///
    /// Uses Acquire for initial read and AcqRel for CAS operations.
    #[inline]
    pub fn flip_mark_bits(&self) {
        // Acquire: must see all prior writes to mark word
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

            // AcqRel for success (atomic update), Acquire for failure (retry with actual value)
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
    ///
    /// Uses Acquire ordering to ensure we see prior writes.
    #[inline]
    pub fn is_marked(&self) -> bool {
        // Acquire: must see all prior writes to mark word
        let mark = self.mark_word.load(Ordering::Acquire);
        (mark & (MARKED0_MASK | MARKED1_MASK)) != 0
    }

    // === Forwarding Pointer Operations ===

    /// Check if object is forwarded (relocated)
    ///
    /// Uses Acquire ordering to ensure we see prior writes.
    #[inline]
    pub fn is_forwarded(&self) -> bool {
        // Acquire: must see prior writes to forwarded flag
        self.mark_word.load(Ordering::Acquire) & FORWARDED_MASK != 0
    }

    /// Set forwarded flag
    ///
    /// Uses Release ordering to ensure our write is visible to other threads.
    #[inline]
    pub fn set_forwarded(&self) {
        // Release: ensure our write is visible before we continue
        self.mark_word.fetch_or(FORWARDED_MASK, Ordering::Release);
    }

    /// Get forwarding pointer
    ///
    /// Uses Acquire ordering to ensure we see prior writes.
    #[inline]
    pub fn get_forwarding_ptr(&self) -> usize {
        // Acquire: must see prior writes to forwarding pointer
        self.mark_word.load(Ordering::Acquire) & HASH_MASK
    }

    /// Set forwarding pointer atomically
    /// Returns true if successfully set (was not already forwarded)
    ///
    /// Uses Acquire for initial read and AcqRel for CAS operations.
    #[inline]
    pub fn try_set_forwarding_ptr(&self, new_addr: usize) -> bool {
        // Acquire: must see prior writes to check if already forwarded
        let mut current = self.mark_word.load(Ordering::Acquire);
        loop {
            if current & FORWARDED_MASK != 0 {
                return false; // Already forwarded
            }

            let new_mark = (current & !HASH_MASK) | (new_addr & HASH_MASK);

            // AcqRel for success (atomic update), Acquire for failure (retry with actual value)
            match self.mark_word.compare_exchange_weak(
                current,
                new_mark,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.set_forwarded();
                    return true;
                },
                Err(val) => current = val,
            }
        }
    }

    // === Age Operations (for generational GC) ===

    /// Get object age (0-15)
    ///
    /// Uses Relaxed ordering since exact age value is not critical for correctness.
    #[inline]
    pub fn get_age(&self) -> u8 {
        // Relaxed: age is statistics, eventual consistency is acceptable
        ((self.mark_word.load(Ordering::Relaxed) & AGE_MASK) >> AGE_SHIFT) as u8
    }

    /// Increment object age
    /// Returns new age
    ///
    /// Uses Relaxed ordering since exact age value is not critical for correctness.
    #[inline]
    pub fn increment_age(&self) -> u8 {
        // Relaxed: age is statistics, eventual consistency is acceptable
        let mut current = self.mark_word.load(Ordering::Relaxed);
        loop {
            let age = ((current & AGE_MASK) >> AGE_SHIFT) as u8;
            if age >= 15 {
                return 15; // Max age
            }

            let new = (current & !AGE_MASK) | (((age + 1) as usize) << AGE_SHIFT);

            // Relaxed for both success and failure: age is statistics
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
    ///
    /// Uses Acquire ordering to ensure we see prior writes.
    #[inline]
    pub fn in_remset(&self) -> bool {
        // Acquire: must see prior writes to remset flag
        self.mark_word.load(Ordering::Acquire) & REMSET_MASK != 0
    }

    /// Set remembered set flag
    /// Returns true if bit was already set
    ///
    /// Uses AcqRel ordering for read-modify-write operation.
    #[inline]
    pub fn set_remset(&self) -> bool {
        // AcqRel: atomic read-modify-write for remset flag
        self.mark_word.fetch_or(REMSET_MASK, Ordering::AcqRel) & REMSET_MASK != 0
    }

    /// Clear remembered set flag
    ///
    /// Uses AcqRel ordering for read-modify-write operation.
    #[inline]
    pub fn clear_remset(&self) {
        // AcqRel: atomic read-modify-write for clearing remset flag
        self.mark_word.fetch_and(!REMSET_MASK, Ordering::AcqRel);
    }

    // === Size Operations ===

    /// Get object size (including header)
    #[inline]
    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Get object data size (excluding header)
    ///
    /// Returns 0 if object size is less than header size (invalid object).
    #[inline]
    pub fn get_data_size(&self) -> usize {
        self.size.saturating_sub(HEADER_SIZE)
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
///
/// This function is safe to call if and only if:
/// 1. `obj_addr` points to a valid GC-managed object
/// 2. The object has an ObjectHeader at the start (offset 0)
/// 3. The object memory will remain valid for the duration of any subsequent operations
/// 4. No other thread is concurrently relocating or freeing this object
///
/// # Panics
/// This function will not panic, but dereferencing the returned pointer
/// when the safety conditions are violated will cause undefined behavior.
///
/// # Examples
///
/// ```rust
/// use fgc::object::header::{ObjectHeader, get_header, HEADER_SIZE};
///
/// let header = ObjectHeader::new(0x1000, 64);
/// let addr = &header as *const ObjectHeader as usize;
///
/// unsafe {
///     let header_ptr = get_header(addr);
///     // Can now access header through the pointer
/// }
/// ```
#[inline]
pub unsafe fn get_header(obj_addr: usize) -> *mut ObjectHeader {
    obj_addr as *mut ObjectHeader
}

/// Get object data start (after header)
///
/// Returns the address immediately after the object header, where the
/// object's actual data begins.
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `obj_addr` points to a valid GC-managed object
/// 2. The object has an ObjectHeader at the start (offset 0)
/// 3. The object size is at least HEADER_SIZE bytes
/// 4. The object memory will remain valid for the duration of any subsequent operations
///
/// # Panics
/// This function will not panic, but accessing memory at the returned address
/// when the safety conditions are violated will cause undefined behavior.
///
/// # Examples
///
/// ```rust
/// use fgc::object::header::{ObjectHeader, get_data_start, HEADER_SIZE};
///
/// let header = ObjectHeader::new(0x1000, 64);
/// let addr = &header as *const ObjectHeader as usize;
///
/// unsafe {
///     let data_start = get_data_start(addr);
///     assert_eq!(data_start, addr + HEADER_SIZE);
/// }
/// ```
#[inline]
pub unsafe fn get_data_start(obj_addr: usize) -> usize {
    obj_addr + HEADER_SIZE
}

/// Get object address from header pointer
///
/// Returns the base address of an object given a pointer to its header.
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `header` is a valid pointer to an ObjectHeader
/// 2. The ObjectHeader is at the start of a GC-managed object
/// 3. The object memory will remain valid for the duration of any subsequent operations
///
/// # Panics
/// This function will not panic. The returned address is simply the
/// numeric value of the pointer.
///
/// # Examples
///
/// ```rust
/// use fgc::object::header::{ObjectHeader, get_object_addr};
///
/// let header = ObjectHeader::new(0x1000, 64);
/// let addr = &header as *const ObjectHeader as usize;
///
/// unsafe {
///     let recovered_addr = get_object_addr(&header);
///     assert_eq!(recovered_addr, addr);
/// }
/// ```
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

    #[test]
    fn test_object_header_atomic_ordering() {
        let header = ObjectHeader::new(0x1000, 1024);

        // Test mark operations are idempotent
        // set_marked0 returns true if bit was ALREADY set, false if we just set it
        assert!(!header.set_marked0()); // First mark: was not set, returns false
        assert!(header.set_marked0()); // Second mark: was already set, returns true
        assert!(header.is_marked0()); // Check sees the mark

        // Test Marked1 operations
        assert!(!header.set_marked1()); // First mark: was not set
        assert!(header.set_marked1()); // Second mark: was already set
        assert!(header.is_marked1()); // Check sees the mark
    }
}
