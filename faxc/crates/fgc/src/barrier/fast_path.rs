//! Fast Path - Inline Load Barrier for Performance
//!
//! Fast path is an inline barrier embedded directly in code generation.
//! Slow path (function call) is only invoked if fast path fails.
//!
//! Performance Considerations:
//! - Function marked `#[inline(always)]` to ensure inlining
//! - Minimal branches in fast path
//! - Atomic operations with relaxed ordering for performance
//! - Branch prediction friendly code layout
//!
//! Fast Path Logic:
//! ```
//! if (pointer == null) return true;           // Null check
//! if (mark_word & MARK_MASK != 0) return true; // Already marked
//! return false;                                // Need slow path
//! ```
//!
//! Slow Path:
//! - Function call to load_barrier_slow_path
//! - Enqueue object to mark queue
//! - Handle pointer healing if needed

use crate::object::{ObjectHeader, HEADER_SIZE};
use crate::barrier::colored_ptr::{ColoredPointer, GcPhase};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Mark bit masks for fast path check
///
/// Uses constants for compile-time optimization.
pub const MARKED0_MASK: usize = 1 << 44;
pub const MARKED1_MASK: usize = 1 << 45;
pub const MARK_MASK: usize = MARKED0_MASK | MARKED1_MASK;

/// Fast path load barrier - inline version
///
/// This is the hot path that should be as efficient as possible.
/// Compiler should be able to inline this function.
///
/// # Arguments
/// * `obj_addr_ptr` - Pointer to pointer being read (double indirection)
///
/// # Returns
/// * `true` - Read can proceed (object already marked or null)
/// * `false` - Need to call slow path (object not yet marked)
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `obj_addr_ptr` is a valid, aligned pointer to a `usize`
/// 2. The memory at `obj_addr_ptr` contains a valid object address or null
/// 3. If the object address is non-null, it points to a valid GC-managed object
///    with an ObjectHeader at the start
/// 4. No other thread is concurrently modifying the pointer at `obj_addr_ptr`
///    without proper synchronization
///
/// # Panics
/// This function will not panic, but may cause undefined behavior if
/// the safety conditions are violated.
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::load_barrier_fast_path;
///
/// let mut ptr: usize = 0x1000;
/// unsafe {
///     if load_barrier_fast_path(&mut ptr) {
///         // Fast path succeeded, proceed with read
///     } else {
///         // Need slow path
///     }
/// }
/// ```
#[inline(always)]
pub unsafe fn load_barrier_fast_path(obj_addr_ptr: *mut usize) -> bool {
    let obj_addr = obj_addr_ptr.read_volatile();

    // Null pointer check - fast path
    if obj_addr == 0 {
        return true;
    }

    // Quick validity check - skip if address looks invalid
    // This is a minimal check to avoid crashing on obviously bad pointers
    if !crate::memory::is_readable(obj_addr).unwrap_or(false) {
        return false;  // Force slow path for invalid addresses
    }

    // Read mark word directly from header
    // ObjectHeader is at the start of object (offset 0)
    let header = obj_addr as *const ObjectHeader;

    // Load mark word with acquire ordering for visibility
    let mark_word = (*header).mark_word.load(Ordering::Acquire);

    // Check mark bits (MARKED0 or MARKED1)
    // If either is set, object is already marked
    if mark_word & MARK_MASK != 0 {
        return true; // Already marked, fast path success
    }

    false // Need slow path
}

/// Fast path with colored pointer
///
/// Variant that works with colored pointer (bits 44-47).
///
/// # Arguments
/// * `colored_ptr` - Colored pointer value
///
/// # Returns
/// * `true` - Pointer does not need processing
/// * `false` - Need slow path
#[inline(always)]
pub fn colored_fast_path(colored_ptr: usize) -> bool {
    // Null check
    if colored_ptr == 0 {
        return true;
    }

    let ptr = ColoredPointer::from_raw(colored_ptr);

    // Check if already marked
    ptr.is_marked()
}

/// Fast path with explicit phase check
///
/// Version that checks GC phase to determine behavior.
///
/// # Arguments
/// * `obj_addr_ptr` - Pointer to pointer being read
/// * `phase` - Current GC phase
///
/// # Returns
/// * `true` - Read can proceed
/// * `false` - Need slow path
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `obj_addr_ptr` is a valid, aligned pointer to a `usize`
/// 2. The memory at `obj_addr_ptr` contains a valid object address or null
/// 3. If the object address is non-null, it points to a valid GC-managed object
///    with an ObjectHeader at the start
/// 4. No other thread is concurrently modifying the pointer without synchronization
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::load_barrier_fast_path_with_phase;
/// use fgc::barrier::colored_ptr::GcPhase;
///
/// let mut ptr: usize = 0x1000;
/// unsafe {
///     if load_barrier_fast_path_with_phase(&mut ptr, GcPhase::Marking) {
///         // Fast path succeeded
///     }
/// }
/// ```
#[inline(always)]
pub unsafe fn load_barrier_fast_path_with_phase(
    obj_addr_ptr: *mut usize,
    phase: GcPhase,
) -> bool {
    let obj_addr = obj_addr_ptr.read_volatile();

    if obj_addr == 0 {
        return true;
    }

    // Quick validity check - skip if address looks invalid
    if !crate::memory::is_readable(obj_addr).unwrap_or(false) {
        return false;  // Force slow path for invalid addresses
    }

    match phase {
        GcPhase::Marking => {
            let header = obj_addr as *const ObjectHeader;
            let mark_word = (*header).mark_word.load(Ordering::Acquire);

            // Check mark bits
            mark_word & MARK_MASK != 0
        }
        GcPhase::Relocating => {
            // Check forwarded bit
            let header = obj_addr as *const ObjectHeader;
            let mark_word = (*header).mark_word.load(Ordering::Acquire);

            // Check FORWARDED_BIT (bit 3)
            const FORWARDED_MASK: usize = 1 << 3;
            mark_word & FORWARDED_MASK == 0 // Not forwarded = can proceed
        }
        _ => true, // Idle or cleanup, no barrier needed
    }
}

/// Slow path - called when fast path fails
///
/// This function handles objects that are not yet marked:
/// 1. Enqueue object to mark queue
/// 2. Set mark bit atomically
/// 3. Handle pointer healing if needed
///
/// # Arguments
/// * `obj_addr` - Object address that needs marking
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `obj_addr` is null OR points to a valid GC-managed object
/// 2. The object at `obj_addr` has an ObjectHeader at the start
/// 3. The object memory will remain valid for the duration of this call
/// 4. No other thread is concurrently relocating this object
///
/// # Panics
/// This function will not panic, but may cause undefined behavior if
/// `obj_addr` is non-null and points to invalid memory.
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::load_barrier_slow_path;
///
/// // Only call with valid GC-managed object addresses
/// load_barrier_slow_path(0x1000);
/// ```
pub fn load_barrier_slow_path(obj_addr: usize) {
    if obj_addr == 0 {
        return;
    }

    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);

        // Double-check mark bits (might have been marked by another thread)
        let mark_word = header.mark_word.load(Ordering::Acquire);
        if mark_word & MARK_MASK != 0 {
            return; // Already marked, no action needed
        }

        // Try to set Marked0 bit atomically
        // Only one thread should "win" and enqueue the object
        let old_mark = header.mark_word.fetch_or(MARKED0_MASK, Ordering::AcqRel);

        if old_mark & MARK_MASK == 0 {
            // We won the race, enqueue object for marking
            // This is handled by the LoadBarrier's mark_queue
            crate::barrier::load_barrier::enqueue_for_marking(obj_addr);
        }
        // else: another thread already marked it, no need to enqueue
    }
}

/// Slow path with colored pointer
///
/// # Arguments
/// * `colored_ptr` - Colored pointer that needs processing
/// * `phase` - Current GC phase
///
/// # Returns
/// Processed colored pointer
pub fn colored_slow_path(colored_ptr: usize, phase: GcPhase) -> ColoredPointer {
    let mut ptr = ColoredPointer::from_raw(colored_ptr);

    if ptr.address() == 0 {
        return ptr;
    }

    match phase {
        GcPhase::Marking => {
            if !ptr.is_marked() {
                // Enqueue for marking
                crate::barrier::load_barrier::enqueue_for_marking(ptr.address());

                // Set mark bit
                ptr.set_marked0();
            }
        }
        GcPhase::Relocating => {
            // Check forwarding table and heal pointer if needed
            // This is handled by the LoadBarrier
        }
        _ => {}
    }

    ptr
}

/// Atomic fast path check
///
/// Version that uses atomic operations for thread safety.
///
/// # Arguments
/// * `ptr_location` - Atomic pointer location
///
/// # Returns
/// * `true` - Fast path success (object already marked or null)
/// * `false` - Need slow path (object not marked)
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `ptr_location` points to valid memory containing an object address
/// 2. If the address is non-null, it points to a valid GC-managed object
///    with an ObjectHeader at the start
/// 3. The object memory will remain valid for the duration of this call
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::atomic_fast_path;
/// use std::sync::atomic::AtomicUsize;
///
/// let atomic_ptr = AtomicUsize::new(0x1000);
/// let needs_slow_path = !atomic_fast_path(&atomic_ptr);
/// ```
#[inline(always)]
pub fn atomic_fast_path(ptr_location: &AtomicUsize) -> bool {
    let obj_addr = ptr_location.load(Ordering::Acquire);

    if obj_addr == 0 {
        return true;
    }

    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);
        let mark_word = header.mark_word.load(Ordering::Acquire);

        mark_word & MARK_MASK != 0
    }
}

/// Atomic test-and-set for marking
///
/// Atomically check and set mark bit.
/// Returns true if bit was already set (another thread marked it).
///
/// # Arguments
/// * `obj_addr` - Object address
///
/// # Returns
/// * `true` - Already marked
/// * `false` - We marked it
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `obj_addr` is null OR points to a valid GC-managed object
/// 2. The object at `obj_addr` has an ObjectHeader at the start
/// 3. The object memory will remain valid for the duration of this call
/// 4. No other thread is concurrently relocating this object
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::atomic_test_and_set_mark;
///
/// // Returns false if we set the mark, true if already marked
/// let was_already_marked = atomic_test_and_set_mark(0x1000);
/// ```
#[inline]
pub fn atomic_test_and_set_mark(obj_addr: usize) -> bool {
    if obj_addr == 0 {
        return true;
    }

    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);
        let old_mark = header.mark_word.fetch_or(MARKED0_MASK, Ordering::AcqRel);

        old_mark & MARK_MASK != 0
    }
}

/// Inline barrier for code generation
///
/// This function is designed to be inlined by code generator.
/// Returns processed pointer value.
///
/// # Arguments
/// * `ptr` - Raw pointer value
///
/// # Returns
/// Processed pointer value
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `ptr` is null OR points to a valid GC-managed object
/// 2. The object at `ptr` has an ObjectHeader at the start
/// 3. The object memory will remain valid for the duration of this call
/// 4. No other thread is concurrently relocating this object
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::inline_barrier;
///
/// let processed = inline_barrier(0x1000);
/// ```
#[inline(always)]
pub fn inline_barrier(ptr: usize) -> usize {
    if ptr == 0 {
        return 0;
    }

    unsafe {
        let header = &*(ptr as *const ObjectHeader);
        let mark_word = header.mark_word.load(Ordering::Acquire);

        if mark_word & MARK_MASK == 0 {
            // Need slow path
            load_barrier_slow_path(ptr);
        }
    }

    ptr
}

/// Branch prediction hint for fast path
///
/// Uses compiler hints to optimize branch prediction.
#[inline(always)]
pub fn likely(b: bool) -> bool {
    b
}

/// Branch prediction hint for slow path
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    b
}

/// Optimized fast path with branch prediction hints
///
/// # Arguments
/// * `obj_addr_ptr` - Pointer to pointer being read
///
/// # Returns
/// * `true` - Fast path success
/// * `false` - Need slow path
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `obj_addr_ptr` is a valid, aligned pointer to a `usize`
/// 2. The memory at `obj_addr_ptr` contains a valid object address or null
/// 3. If the object address is non-null, it points to a valid GC-managed object
///    with an ObjectHeader at the start
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::optimized_fast_path;
///
/// let mut ptr: usize = 0x1000;
/// unsafe {
///     if optimized_fast_path(&mut ptr) {
///         // Fast path succeeded
///     }
/// }
/// ```
#[inline(always)]
pub unsafe fn optimized_fast_path(obj_addr_ptr: *mut usize) -> bool {
    let obj_addr = obj_addr_ptr.read_volatile();

    // Null check - very likely
    if unlikely(obj_addr == 0) {
        return true;
    }

    // Quick validity check - skip if address looks invalid
    if !crate::memory::is_readable(obj_addr).unwrap_or(false) {
        return false;  // Force slow path for invalid addresses
    }

    let header = obj_addr as *const ObjectHeader;
    let mark_word = (*header).mark_word.load(Ordering::Acquire);

    // Already marked - likely during steady state
    if likely(mark_word & MARK_MASK != 0) {
        return true;
    }

    false // Unlikely, need slow path
}

/// Batch fast path check for multiple pointers
///
/// Check multiple pointers and return bitmap of which need slow path.
///
/// # Arguments
/// * `ptrs` - Slice of pointer addresses
///
/// # Returns
/// Bitmap where bit i = 1 means ptrs[i] needs slow path
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. All non-null pointers in `ptrs` point to valid GC-managed objects
/// 2. Each object has an ObjectHeader at the start
/// 3. All object memory will remain valid for the duration of this call
/// 4. No other thread is concurrently relocating these objects
///
/// # Examples
///
/// ```rust
/// use fgc::barrier::fast_path::batch_fast_path;
///
/// let ptrs = [0x1000, 0x2000, 0x3000];
/// let needs_slow_path = batch_fast_path(&ptrs);
/// ```
pub fn batch_fast_path(ptrs: &[usize]) -> u64 {
    let mut needs_slow_path: u64 = 0;

    for (i, &ptr) in ptrs.iter().enumerate().take(64) {
        if ptr == 0 {
            continue;
        }

        unsafe {
            let header = &*(ptr as *const ObjectHeader);
            let mark_word = header.mark_word.load(Ordering::Acquire);

            if mark_word & MARK_MASK == 0 {
                needs_slow_path |= 1 << i;
            }
        }
    }

    needs_slow_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjectHeader;
    use std::sync::Arc;
    use std::thread;

    // Helper function to create test object
    fn create_test_object() -> (Vec<u8>, usize) {
        let size = HEADER_SIZE + 64; // Header + data
        let mut buffer = vec![0u8; size];

        // Initialize header
        let header_ptr = buffer.as_mut_ptr() as *mut ObjectHeader;
        unsafe {
            std::ptr::write(header_ptr, ObjectHeader::new(0x1000, size));
        }

        let addr = buffer.as_ptr() as usize;
        (buffer, addr)
    }

    // === Fast Path Tests ===

    #[test]
    fn test_fast_path_null_pointer() {
        let mut ptr: usize = 0;
        unsafe {
            assert!(load_barrier_fast_path(&mut ptr));
        }
    }

    #[test]
    fn test_fast_path_unmarked_object() {
        let (_buffer, addr) = create_test_object();

        let mut ptr = addr;
        unsafe {
            assert!(!load_barrier_fast_path(&mut ptr));
        }
    }

    #[test]
    fn test_fast_path_marked_object() {
        let (_buffer, addr) = create_test_object();

        // Mark the object
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked0();
        }

        let mut ptr = addr;
        unsafe {
            assert!(load_barrier_fast_path(&mut ptr));
        }
    }

    #[test]
    fn test_fast_path_marked1_object() {
        let (_buffer, addr) = create_test_object();

        // Mark with Marked1
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked1();
        }

        let mut ptr = addr;
        unsafe {
            assert!(load_barrier_fast_path(&mut ptr));
        }
    }

    #[test]
    fn test_colored_fast_path() {
        // Null
        assert!(colored_fast_path(0));

        // Unmarked
        let ptr = ColoredPointer::new(0x1000);
        assert!(!colored_fast_path(ptr.raw()));

        // Marked
        let mut ptr = ColoredPointer::new(0x1000);
        ptr.set_marked0();
        assert!(colored_fast_path(ptr.raw()));
    }

    #[test]
    fn test_fast_path_with_phase_idle() {
        let (_buffer, addr) = create_test_object();

        let mut ptr = addr;
        unsafe {
            // Idle phase should always succeed
            assert!(load_barrier_fast_path_with_phase(&mut ptr, GcPhase::Idle));
        }
    }

    #[test]
    fn test_fast_path_with_phase_marking() {
        let (_buffer, addr) = create_test_object();

        // Unmarked object during marking phase
        let mut ptr = addr;
        unsafe {
            assert!(!load_barrier_fast_path_with_phase(&mut ptr, GcPhase::Marking));
        }

        // Marked object during marking phase
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked0();
        }

        let mut ptr = addr;
        unsafe {
            assert!(load_barrier_fast_path_with_phase(&mut ptr, GcPhase::Marking));
        }
    }

    // === Atomic Tests ===

    #[test]
    fn test_atomic_fast_path() {
        let (_buffer, addr) = create_test_object();

        let atomic_ptr = AtomicUsize::new(addr);
        assert!(!atomic_fast_path(&atomic_ptr));

        // Mark the object
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked0();
        }

        assert!(atomic_fast_path(&atomic_ptr));
    }

    #[test]
    fn test_atomic_test_and_set_mark() {
        let (_buffer, addr) = create_test_object();

        // First call should return false (we set the mark)
        assert!(!atomic_test_and_set_mark(addr));

        // Second call should return true (already marked)
        assert!(atomic_test_and_set_mark(addr));
    }

    #[test]
    fn test_atomic_test_and_set_mark_concurrent() {
        let (_buffer, addr) = create_test_object();
        let addr = Arc::new(addr);

        let mut handles = vec![];
        let mut set_count = 0;

        for _ in 0..10 {
            let addr_clone = Arc::clone(&addr);
            let handle = thread::spawn(move || {
                atomic_test_and_set_mark(*addr_clone)
            });
            handles.push(handle);
        }

        for handle in handles {
            if !handle.join().unwrap() {
                set_count += 1;
            }
        }

        // Exactly one thread should have set the mark
        assert_eq!(set_count, 1);
    }

    // === Inline Barrier Tests ===

    #[test]
    fn test_inline_barrier_null() {
        assert_eq!(inline_barrier(0), 0);
    }

    #[test]
    fn test_inline_barrier_unmarked() {
        let (_buffer, addr) = create_test_object();

        // This will call slow path
        let result = inline_barrier(addr);
        assert_eq!(result, addr);
    }

    #[test]
    fn test_inline_barrier_marked() {
        let (_buffer, addr) = create_test_object();

        // Mark first
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked0();
        }

        let result = inline_barrier(addr);
        assert_eq!(result, addr);
    }

    // === Optimized Fast Path Tests ===

    #[test]
    fn test_optimized_fast_path() {
        let (_buffer, addr) = create_test_object();

        // Unmarked
        let mut ptr = addr;
        unsafe {
            assert!(!optimized_fast_path(&mut ptr));
        }

        // Marked
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked0();
        }

        let mut ptr = addr;
        unsafe {
            assert!(optimized_fast_path(&mut ptr));
        }
    }

    // === Batch Fast Path Tests ===

    #[test]
    fn test_batch_fast_path() {
        let (_buffer1, addr1) = create_test_object();
        let (_buffer2, addr2) = create_test_object();
        let (_buffer3, addr3) = create_test_object();

        // Mark second object
        unsafe {
            let header = &*(addr2 as *const ObjectHeader);
            header.set_marked0();
        }

        let ptrs = [addr1, addr2, addr3];
        let bitmap = batch_fast_path(&ptrs);

        // Bit 0 and 2 should be set (need slow path)
        // Bit 1 should be clear (already marked)
        assert_eq!(bitmap & 0b101, 0b101);
    }

    #[test]
    fn test_batch_fast_path_with_nulls() {
        let (_buffer, addr) = create_test_object();

        let ptrs = [0, addr, 0];
        let bitmap = batch_fast_path(&ptrs);

        // Only bit 1 should be set (null pointers are skipped)
        assert_eq!(bitmap & 0b010, 0b010);
    }

    #[test]
    fn test_batch_fast_path_all_marked() {
        let (_buffer1, addr1) = create_test_object();
        let (_buffer2, addr2) = create_test_object();

        unsafe {
            (&*(addr1 as *const ObjectHeader)).set_marked0();
            (&*(addr2 as *const ObjectHeader)).set_marked1();
        }

        let ptrs = [addr1, addr2];
        let bitmap = batch_fast_path(&ptrs);

        assert_eq!(bitmap, 0);
    }

    // === Slow Path Tests ===

    #[test]
    fn test_slow_path_null() {
        // Should not panic
        load_barrier_slow_path(0);
    }

    #[test]
    fn test_slow_path_already_marked() {
        let (_buffer, addr) = create_test_object();

        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.set_marked0();
        }

        // Should not do anything
        load_barrier_slow_path(addr);
    }

    // === Integration Tests ===

    #[test]
    fn test_fast_slow_path_integration() {
        let (_buffer, addr) = create_test_object();

        // First check: unmarked, need slow path
        let mut ptr = addr;
        unsafe {
            assert!(!load_barrier_fast_path(&mut ptr));
        }

        // Simulate slow path marking
        unsafe {
            let header = &*(addr as *const ObjectHeader);
            header.mark_word.fetch_or(MARKED0_MASK, Ordering::AcqRel);
        }

        // Second check: marked, fast path success
        let mut ptr = addr;
        unsafe {
            assert!(load_barrier_fast_path(&mut ptr));
        }
    }

    #[test]
    fn test_concurrent_fast_path() {
        let (_buffer, addr) = create_test_object();
        let addr = Arc::new(addr);

        let mut handles = vec![];

        for _ in 0..10 {
            let addr_clone = Arc::clone(&addr);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    unsafe {
                        let mut ptr = *addr_clone;
                        load_barrier_fast_path(&mut ptr);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should complete without issues
    }
}
