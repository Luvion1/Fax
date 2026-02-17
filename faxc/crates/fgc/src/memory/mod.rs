//! Memory Operations - Safe wrappers around low-level memory operations
//!
//! This module provides safe abstractions for common memory operations
//! used by the garbage collector, including copying, zeroing, and
//! pointer/value read/write operations.
//!
//! # Safety
//!
//! Most functions in this module are `unsafe` because they operate on
//! raw memory addresses. The caller must ensure that:
//! - Addresses are valid and properly aligned
//! - Memory regions do not overlap (for copy operations)
//! - Sizes do not overflow
//!
//! # Example
//!
//! ```rust
//! use fgc::memory::{copy_memory, zero_memory};
//!
//! let mut buffer = [0u8; 64];
//! let src = [1u8, 2, 3, 4];
//!
//! unsafe {
//!     copy_memory(src.as_ptr() as usize, buffer.as_mut_ptr() as usize, 4);
//!     assert_eq!(&buffer[0..4], &[1, 2, 3, 4]);
//! }
//! ```

use std::ptr;

/// Copy memory from source to destination
///
/// This function performs a non-overlapping memory copy, similar to `memcpy`.
/// For overlapping regions, use `copy_memory_overlapping` instead.
///
/// # Safety
///
/// - `src` must be valid for reads of `size` bytes
/// - `dst` must be valid for writes of `size` bytes
/// - `src` and `dst` must not overlap
/// - `size` must not overflow `usize`
/// - If `size` is 0, the function is a no-op
///
/// # Example
///
/// ```rust
/// use fgc::memory::copy_memory;
///
/// let src = [1u8, 2, 3, 4, 5];
/// let mut dst = [0u8; 5];
///
/// unsafe {
///     copy_memory(src.as_ptr() as usize, dst.as_mut_ptr() as usize, 5);
///     assert_eq!(dst, [1, 2, 3, 4, 5]);
/// }
/// ```
#[inline]
pub unsafe fn copy_memory(src: usize, dst: usize, size: usize) {
    if size == 0 {
        return;
    }
    ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
}

/// Copy memory from source to destination with overlapping regions
///
/// This function handles overlapping memory regions correctly, similar to `memmove`.
/// It is slightly slower than `copy_memory` but safe for overlapping regions.
///
/// # Safety
///
/// - `src` must be valid for reads of `size` bytes
/// - `dst` must be valid for writes of `size` bytes
/// - `size` must not overflow `usize`
///
/// # Example
///
/// ```rust
/// use fgc::memory::copy_memory_overlapping;
///
/// let mut buffer = [1u8, 2, 3, 4, 5];
///
/// unsafe {
///     // Copy within the same buffer (overlapping)
///     copy_memory_overlapping(
///         buffer.as_ptr() as usize,
///         buffer.as_mut_ptr().add(1) as usize,
///         4,
///     );
///     assert_eq!(buffer, [1, 1, 2, 3, 4]);
/// }
/// ```
#[inline]
pub unsafe fn copy_memory_overlapping(src: usize, dst: usize, size: usize) {
    if size == 0 {
        return;
    }
    ptr::copy(src as *const u8, dst as *mut u8, size);
}

/// Zero-fill a memory region
///
/// Sets all bytes in the specified memory region to zero.
///
/// # Safety
///
/// - `addr` must be valid for writes of `size` bytes
/// - `size` must not overflow `usize`
/// - If `size` is 0, the function is a no-op
///
/// # Example
///
/// ```rust
/// use fgc::memory::zero_memory;
///
/// let mut buffer = [0xFFu8; 8];
///
/// unsafe {
///     zero_memory(buffer.as_mut_ptr() as usize, 8);
///     assert_eq!(buffer, [0u8; 8]);
/// }
/// ```
#[inline]
pub unsafe fn zero_memory(addr: usize, size: usize) {
    if size == 0 {
        return;
    }
    ptr::write_bytes(addr as *mut u8, 0, size);
}

/// Read a pointer (usize) from an address
///
/// # Safety
///
/// - `addr` must be aligned for `usize` (typically 8 bytes on 64-bit)
/// - `addr` must be valid for reading `usize` bytes
/// - The memory at `addr` must be initialized
///
/// # Example
///
/// ```rust
/// use fgc::memory::read_pointer;
///
/// let ptr_value: usize = 0x12345678;
///
/// unsafe {
///     let result = read_pointer(&ptr_value as *const usize as usize);
///     assert_eq!(result, 0x12345678);
/// }
/// ```
#[inline]
pub unsafe fn read_pointer(addr: usize) -> usize {
    ptr::read(addr as *const usize)
}

/// Write a pointer to an address
///
/// # Safety
///
/// - `addr` must be aligned for `usize` (typically 8 bytes on 64-bit)
/// - `addr` must be valid for writing `usize` bytes
///
/// # Example
///
/// ```rust
/// use fgc::memory::write_pointer;
///
/// let mut ptr_value: usize = 0;
///
/// unsafe {
///     write_pointer(&mut ptr_value as *mut usize as usize, 0x12345678);
///     assert_eq!(ptr_value, 0x12345678);
/// }
/// ```
#[inline]
pub unsafe fn write_pointer(addr: usize, value: usize) {
    ptr::write(addr as *mut usize, value);
}

/// Read a value of type `T` from an address
///
/// This function performs a read and takes ownership of the value.
/// The caller is responsible for ensuring the memory is properly managed
/// after the read (e.g., not double-freed).
///
/// # Safety
///
/// - `addr` must be aligned for type `T`
/// - `addr` must be valid for reading `T`
/// - The memory at `addr` must contain a properly initialized `T`
/// - After calling this function, the caller must not drop the value
///   at `addr` again (ownership is transferred)
///
/// # Example
///
/// ```rust
/// use fgc::memory::read_value;
///
/// let value: i32 = 42;
///
/// unsafe {
///     let result = read_value(&value as *const i32 as usize);
///     assert_eq!(result, 42);
///     // Note: `value` is still valid here because i32 is Copy
/// }
/// ```
#[inline]
pub unsafe fn read_value<T>(addr: usize) -> T {
    ptr::read(addr as *const T)
}

/// Write a value of type `T` to an address
///
/// This function writes the value without dropping any existing value
/// at the destination (similar to `ptr::write`).
///
/// # Safety
///
/// - `addr` must be aligned for type `T`
/// - `addr` must be valid for writing `T`
/// - If there's an existing value at `addr`, it will be overwritten
///   without being dropped
///
/// # Example
///
/// ```rust
/// use fgc::memory::write_value;
///
/// let mut value: i32 = 0;
///
/// unsafe {
///     write_value(&mut value as *mut i32 as usize, 42);
///     assert_eq!(value, 42);
/// }
/// ```
#[inline]
pub unsafe fn write_value<T>(addr: usize, value: T) {
    ptr::write(addr as *mut T, value);
}

/// Read a value of type `T` from an address without taking ownership
///
/// This function copies the value if `T: Copy`, or creates a reference.
/// Unlike `read_value`, this doesn't transfer ownership.
///
/// # Safety
///
/// - `addr` must be aligned for type `T`
/// - `addr` must be valid for reading `T`
/// - The memory at `addr` must contain a properly initialized `T`
///
/// # Example
///
/// ```rust
/// use fgc::memory::peek_value;
///
/// let value: i32 = 42;
///
/// unsafe {
///     let result = peek_value::<i32>(&value as *const i32 as usize);
///     assert_eq!(result, 42);
/// }
/// ```
#[inline]
pub unsafe fn peek_value<T: Copy>(addr: usize) -> T {
    ptr::read_unaligned(addr as *const T)
}

/// Swap two values of type `T` at given addresses
///
/// # Safety
///
/// - Both addresses must be aligned for type `T`
/// - Both addresses must be valid for reading and writing `T`
/// - The addresses must not overlap
///
/// # Example
///
/// ```rust
/// use fgc::memory::swap_values;
///
/// let mut a: i32 = 1;
/// let mut b: i32 = 2;
///
/// unsafe {
///     swap_values(
///         &mut a as *mut i32 as usize,
///         &mut b as *mut i32 as usize,
///     );
///     assert_eq!(a, 2);
///     assert_eq!(b, 1);
/// }
/// ```
#[inline]
pub unsafe fn swap_values<T>(addr1: usize, addr2: usize) {
    let ptr1 = addr1 as *mut T;
    let ptr2 = addr2 as *mut T;
    ptr::swap(ptr1, ptr2);
}

/// Check if a memory address looks readable (best effort)
///
/// This is a heuristic check and cannot guarantee actual readability.
/// It checks for obviously invalid addresses like null or kernel space.
///
/// # Returns
///
/// `true` if the address looks potentially readable, `false` if obviously invalid.
///
/// # Limitations
///
/// - Cannot detect all invalid addresses
/// - May return `true` for addresses that will fault on access
/// - Platform-specific behavior
///
/// # Example
///
/// ```rust
/// use fgc::memory::is_readable;
///
/// assert!(!is_readable(0)); // Null pointer
/// assert!(!is_readable(0xFFFF_FFFF_FFFF_FFFF)); // Likely invalid
/// ```
#[inline]
pub fn is_readable(addr: usize) -> bool {
    if addr == 0 {
        return false;
    }

    // Check for obviously invalid addresses
    // On Linux/x86_64, user space is typically below 0x0000_8000_0000_0000
    // Kernel space starts at 0xFFFF_8000_0000_0000
    #[cfg(target_arch = "x86_64")]
    {
        if addr >= 0xFFFF_0000_0000_0000 {
            return false; // Kernel space
        }
    }

    // Check for unreasonably large addresses on any platform
    if addr > usize::MAX / 2 {
        return false;
    }

    true
}

/// Check if a memory address looks writable (best effort)
///
/// This is a heuristic check and cannot guarantee actual writability.
/// It checks for obviously invalid addresses like null, read-only, or kernel space.
///
/// # Returns
///
/// `true` if the address looks potentially writable, `false` if obviously invalid.
///
/// # Limitations
///
/// - Cannot detect all invalid addresses
/// - Cannot detect read-only memory regions
/// - May return `true` for addresses that will fault on access
/// - Platform-specific behavior
///
/// # Example
///
/// ```rust
/// use fgc::memory::is_writable;
///
/// assert!(!is_writable(0)); // Null pointer
/// assert!(!is_writable(0xFFFF_FFFF_FFFF_FFFF)); // Likely invalid
/// ```
#[inline]
pub fn is_writable(addr: usize) -> bool {
    // Writable addresses must also be readable
    if !is_readable(addr) {
        return false;
    }

    // Additional checks for writable memory
    // Very low addresses are typically not writable
    if addr < 0x1000 {
        return false;
    }

    true
}

/// Compare two memory regions for equality
///
/// # Safety
///
/// - Both `addr1` and `addr2` must be valid for reads of `size` bytes
/// - `size` must not overflow `usize`
///
/// # Example
///
/// ```rust
/// use fgc::memory::compare_memory;
///
/// let a = [1u8, 2, 3, 4];
/// let b = [1u8, 2, 3, 4];
/// let c = [1u8, 2, 3, 5];
///
/// unsafe {
///     assert!(compare_memory(
///         a.as_ptr() as usize,
///         b.as_ptr() as usize,
///         4
///     ));
///     assert!(!compare_memory(
///         a.as_ptr() as usize,
///         c.as_ptr() as usize,
///         4
///     ));
/// }
/// ```
#[inline]
pub unsafe fn compare_memory(addr1: usize, addr2: usize, size: usize) -> bool {
    if size == 0 {
        return true;
    }
    // Use ptr::read to compare byte by byte
    let p1 = addr1 as *const u8;
    let p2 = addr2 as *const u8;
    for i in 0..size {
        if ptr::read(p1.add(i)) != ptr::read(p2.add(i)) {
            return false;
        }
    }
    true
}

/// Fill a memory region with a specific byte value
///
/// # Safety
///
/// - `addr` must be valid for writes of `size` bytes
/// - `size` must not overflow `usize`
///
/// # Example
///
/// ```rust
/// use fgc::memory::fill_memory;
///
/// let mut buffer = [0u8; 8];
///
/// unsafe {
///     fill_memory(buffer.as_mut_ptr() as usize, 0xFF, 8);
///     assert_eq!(buffer, [0xFFu8; 8]);
/// }
/// ```
#[inline]
pub unsafe fn fill_memory(addr: usize, value: u8, size: usize) {
    if size == 0 {
        return;
    }
    ptr::write_bytes(addr as *mut u8, value, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Copy Memory Tests ===

    #[test]
    fn test_copy_memory_basic() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dst = [0u8; 5];

        unsafe {
            copy_memory(src.as_ptr() as usize, dst.as_mut_ptr() as usize, 5);
        }

        assert_eq!(src, dst);
    }

    #[test]
    fn test_copy_memory_partial() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dst = [0u8; 5];

        unsafe {
            copy_memory(src.as_ptr() as usize, dst.as_mut_ptr() as usize, 3);
        }

        assert_eq!(&dst[0..3], &[1, 2, 3]);
        assert_eq!(&dst[3..5], &[0, 0]);
    }

    #[test]
    fn test_copy_memory_zero_size() {
        let src = [1u8, 2, 3];
        let mut dst = [0u8; 3];
        let original_dst = dst.clone();

        unsafe {
            copy_memory(src.as_ptr() as usize, dst.as_mut_ptr() as usize, 0);
        }

        assert_eq!(dst, original_dst);
    }

    #[test]
    fn test_copy_memory_usize() {
        let src: usize = 0x123456789ABCDEF0;
        let mut dst: usize = 0;

        unsafe {
            copy_memory(
                &src as *const usize as usize,
                &mut dst as *mut usize as usize,
                std::mem::size_of::<usize>(),
            );
        }

        assert_eq!(src, dst);
    }

    // === Copy Memory Overlapping Tests ===

    #[test]
    fn test_copy_memory_overlapping_forward() {
        let mut buffer = [1u8, 2, 3, 4, 5];

        unsafe {
            copy_memory_overlapping(
                buffer.as_ptr() as usize,
                buffer.as_mut_ptr().add(1) as usize,
                4,
            );
        }

        assert_eq!(buffer, [1, 1, 2, 3, 4]);
    }

    #[test]
    fn test_copy_memory_overlapping_backward() {
        let mut buffer = [1u8, 2, 3, 4, 5];

        unsafe {
            copy_memory_overlapping(
                buffer.as_ptr().add(1) as usize,
                buffer.as_mut_ptr() as usize,
                4,
            );
        }

        assert_eq!(buffer, [2, 3, 4, 5, 5]);
    }

    #[test]
    fn test_copy_memory_overlapping_zero_size() {
        let mut buffer = [1u8, 2, 3];
        let original = buffer.clone();

        unsafe {
            copy_memory_overlapping(
                buffer.as_ptr() as usize,
                buffer.as_mut_ptr() as usize,
                0,
            );
        }

        assert_eq!(buffer, original);
    }

    // === Zero Memory Tests ===

    #[test]
    fn test_zero_memory_basic() {
        let mut buffer = [0xFFu8; 8];

        unsafe {
            zero_memory(buffer.as_mut_ptr() as usize, 8);
        }

        assert_eq!(buffer, [0u8; 8]);
    }

    #[test]
    fn test_zero_memory_partial() {
        let mut buffer = [0xFFu8; 8];

        unsafe {
            zero_memory(buffer.as_mut_ptr() as usize, 4);
        }

        assert_eq!(&buffer[0..4], &[0u8; 4]);
        assert_eq!(&buffer[4..8], &[0xFFu8; 4]);
    }

    #[test]
    fn test_zero_memory_zero_size() {
        let mut buffer = [0xFFu8; 4];
        let original = buffer.clone();

        unsafe {
            zero_memory(buffer.as_mut_ptr() as usize, 0);
        }

        assert_eq!(buffer, original);
    }

    // === Pointer Read/Write Tests ===

    #[test]
    fn test_read_pointer() {
        let ptr_value: usize = 0x123456789ABCDEF0;

        unsafe {
            let result = read_pointer(&ptr_value as *const usize as usize);
            assert_eq!(result, 0x123456789ABCDEF0);
        }
    }

    #[test]
    fn test_write_pointer() {
        let mut ptr_value: usize = 0;

        unsafe {
            write_pointer(&mut ptr_value as *mut usize as usize, 0xFEDCBA9876543210);
        }

        assert_eq!(ptr_value, 0xFEDCBA9876543210);
    }

    #[test]
    fn test_read_write_pointer_roundtrip() {
        let mut ptr_value: usize = 0;
        let original = 0x0123456789ABCDEF;

        unsafe {
            write_pointer(&mut ptr_value as *mut usize as usize, original);
            let result = read_pointer(&ptr_value as *const usize as usize);
            assert_eq!(result, original);
        }
    }

    // === Value Read/Write Tests ===

    #[test]
    fn test_read_value_i32() {
        let value: i32 = 42;

        unsafe {
            let result: i32 = read_value(&value as *const i32 as usize);
            assert_eq!(result, 42);
        }
    }

    #[test]
    fn test_read_value_u64() {
        let value: u64 = 0x123456789ABCDEF0;

        unsafe {
            let result: u64 = read_value(&value as *const u64 as usize);
            assert_eq!(result, 0x123456789ABCDEF0);
        }
    }

    #[test]
    fn test_write_value_i32() {
        let mut value: i32 = 0;

        unsafe {
            write_value(&mut value as *mut i32 as usize, 100);
        }

        assert_eq!(value, 100);
    }

    #[test]
    fn test_read_write_value_roundtrip() {
        let mut value: u64 = 0;
        let original: u64 = 0xFEDCBA9876543210;

        unsafe {
            write_value(&mut value as *mut u64 as usize, original);
            let result: u64 = read_value(&value as *const u64 as usize);
            assert_eq!(result, original);
        }
    }

    #[test]
    fn test_read_value_struct() {
        #[derive(Debug, PartialEq, Copy, Clone)]
        struct Point {
            x: i32,
            y: i32,
        }

        let point = Point { x: 10, y: 20 };

        unsafe {
            let result: Point = read_value(&point as *const Point as usize);
            assert_eq!(result, point);
        }
    }

    // === Peek Value Tests ===

    #[test]
    fn test_peek_value() {
        let value: i32 = 42;

        unsafe {
            let result: i32 = peek_value::<i32>(&value as *const i32 as usize);
            assert_eq!(result, 42);
        }

        // Original value should still be accessible
        assert_eq!(value, 42);
    }

    // === Swap Values Tests ===

    #[test]
    fn test_swap_values_i32() {
        let mut a: i32 = 1;
        let mut b: i32 = 2;

        unsafe {
            swap_values::<i32>(
                &mut a as *mut i32 as usize,
                &mut b as *mut i32 as usize,
            );
        }

        assert_eq!(a, 2);
        assert_eq!(b, 1);
    }

    #[test]
    fn test_swap_values_usize() {
        let mut a: usize = 0x1000;
        let mut b: usize = 0x2000;

        unsafe {
            swap_values::<usize>(
                &mut a as *mut usize as usize,
                &mut b as *mut usize as usize,
            );
        }

        assert_eq!(a, 0x2000);
        assert_eq!(b, 0x1000);
    }

    #[test]
    fn test_swap_values_same_address() {
        let mut value: i32 = 42;

        unsafe {
            swap_values::<i32>(
                &mut value as *mut i32 as usize,
                &mut value as *mut i32 as usize,
            );
        }

        assert_eq!(value, 42);
    }

    // === Memory Check Tests ===

    #[test]
    fn test_is_readable_null() {
        assert!(!is_readable(0));
    }

    #[test]
    fn test_is_readable_kernel_space() {
        #[cfg(target_arch = "x86_64")]
        {
            assert!(!is_readable(0xFFFF_0000_0000_0000));
            assert!(!is_readable(0xFFFF_FFFF_FFFF_FFFF));
        }
    }

    #[test]
    fn test_is_readable_valid() {
        let value: i32 = 42;
        assert!(is_readable(&value as *const i32 as usize));
    }

    #[test]
    fn test_is_writable_null() {
        assert!(!is_writable(0));
    }

    #[test]
    fn test_is_writable_low_address() {
        assert!(!is_writable(0x100));
        assert!(!is_writable(0x500));
    }

    #[test]
    fn test_is_writable_valid() {
        let mut value: i32 = 42;
        assert!(is_writable(&mut value as *mut i32 as usize));
    }

    #[test]
    fn test_is_writable_implies_readable() {
        // If an address is writable, it should also be readable
        let mut value: i32 = 42;
        let addr = &mut value as *mut i32 as usize;

        if is_writable(addr) {
            assert!(is_readable(addr));
        }
    }

    // === Compare Memory Tests ===

    #[test]
    fn test_compare_memory_equal() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 5];

        unsafe {
            assert!(compare_memory(
                a.as_ptr() as usize,
                b.as_ptr() as usize,
                5
            ));
        }
    }

    #[test]
    fn test_compare_memory_not_equal() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 6];

        unsafe {
            assert!(!compare_memory(
                a.as_ptr() as usize,
                b.as_ptr() as usize,
                5
            ));
        }
    }

    #[test]
    fn test_compare_memory_zero_size() {
        let a = [1u8, 2, 3];
        let b = [4u8, 5, 6];

        unsafe {
            assert!(compare_memory(
                a.as_ptr() as usize,
                b.as_ptr() as usize,
                0
            ));
        }
    }

    #[test]
    fn test_compare_memory_partial() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 9, 9];

        unsafe {
            assert!(compare_memory(
                a.as_ptr() as usize,
                b.as_ptr() as usize,
                3
            ));
            assert!(!compare_memory(
                a.as_ptr() as usize,
                b.as_ptr() as usize,
                5
            ));
        }
    }

    // === Fill Memory Tests ===

    #[test]
    fn test_fill_memory() {
        let mut buffer = [0u8; 8];

        unsafe {
            fill_memory(buffer.as_mut_ptr() as usize, 0xAB, 8);
        }

        assert_eq!(buffer, [0xABu8; 8]);
    }

    #[test]
    fn test_fill_memory_partial() {
        let mut buffer = [0u8; 8];

        unsafe {
            fill_memory(buffer.as_mut_ptr() as usize, 0xFF, 4);
        }

        assert_eq!(&buffer[0..4], &[0xFFu8; 4]);
        assert_eq!(&buffer[4..8], &[0u8; 4]);
    }

    #[test]
    fn test_fill_memory_zero() {
        let mut buffer = [0xFFu8; 4];

        unsafe {
            fill_memory(buffer.as_mut_ptr() as usize, 0, 4);
        }

        assert_eq!(buffer, [0u8; 4]);
    }

    #[test]
    fn test_fill_memory_zero_size() {
        let mut buffer = [0xFFu8; 4];
        let original = buffer.clone();

        unsafe {
            fill_memory(buffer.as_mut_ptr() as usize, 0xAB, 0);
        }

        assert_eq!(buffer, original);
    }

    // === Integration Tests ===

    #[test]
    fn test_memory_operations_integration() {
        let mut buffer = [0u8; 32];

        unsafe {
            // Fill with pattern
            fill_memory(buffer.as_mut_ptr() as usize, 0x55, 32);

            // Verify
            assert!(compare_memory(
                buffer.as_ptr() as usize,
                buffer.as_ptr() as usize,
                32
            ));

            // Copy to another buffer
            let mut dst = [0u8; 32];
            copy_memory(
                buffer.as_ptr() as usize,
                dst.as_mut_ptr() as usize,
                32,
            );
            assert_eq!(buffer, dst);

            // Zero the buffer
            zero_memory(buffer.as_mut_ptr() as usize, 32);
            assert_eq!(buffer, [0u8; 32]);
        }
    }

    #[test]
    fn test_pointer_operations_integration() {
        let mut ptr_storage: usize = 0;
        let target: usize = 0x12345678;

        unsafe {
            // Write pointer
            write_pointer(&mut ptr_storage as *mut usize as usize, target);

            // Read back
            let result = read_pointer(&ptr_storage as *const usize as usize);
            assert_eq!(result, target);

            // Verify with value read
            let value_result = read_value::<usize>(&ptr_storage as *const usize as usize);
            assert_eq!(value_result, target);
        }
    }

    #[test]
    fn test_gc_object_simulation() {
        // Simulate a simple GC object layout:
        // [header: 24 bytes][ptr1: 8][i64: 8][ptr2: 8]

        let mut object = [0u8; 48];

        unsafe {
            // Write some pointer values
            write_pointer(object.as_mut_ptr().add(24) as usize, 0x1000);
            write_value(object.as_mut_ptr().add(32) as usize, 42i64);
            write_pointer(object.as_mut_ptr().add(40) as usize, 0x2000);

            // Read back
            let ptr1 = read_pointer(object.as_ptr().add(24) as usize);
            let val = read_value::<i64>(object.as_ptr().add(32) as usize);
            let ptr2 = read_pointer(object.as_ptr().add(40) as usize);

            assert_eq!(ptr1, 0x1000);
            assert_eq!(val, 42);
            assert_eq!(ptr2, 0x2000);
        }
    }
}
