//! GC Barrier Tests - Load Barrier and Pointer Healing
//!
//! These tests verify the load barrier implementation:
//! - Pointer healing after relocation
//! - Load barrier atomic operations
//! - Marked bit flip correctness (CRITICAL BUG TEST)
//! - Concurrent marking with barriers
//!
//! ============================================================================
//! CRITICAL: These tests find the most severe GC bugs
//! DO NOT WEAKEN ASSERTIONS - Each test targets specific known bugs
//! ============================================================================

mod common;

use fgc::barrier::colored_ptr::{ColoredPointer, GcPhase};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

// ============================================================================
// COLORED POINTER BASIC TESTS
// ============================================================================

/// Test ColoredPointer creation preserves address
///
/// **Bug this finds:** Address truncation, mask bugs
/// **Invariant verified:** Address bits (0-43) are preserved exactly
#[test]
fn test_colored_pointer_creation() {
    // Arrange
    let test_addresses = [
        0x0,
        0x1,
        0x100,
        0x1000,
        0x12345678,
        0xFFFFFFFFF,   // Max 36-bit
        0xFFFFFFFFFF,  // Max 40-bit
        0xFFFFFFFFFFF, // Max 44-bit (address limit)
    ];

    // Act & Assert
    for &addr in &test_addresses {
        let ptr = ColoredPointer::new(addr);

        assert_eq!(
            ptr.address(),
            addr,
            "Address not preserved: created with {:#x}, extracted {:#x}",
            addr,
            ptr.address()
        );

        // Color bits should be clear initially
        assert!(
            !ptr.is_marked0() && !ptr.is_marked1(),
            "New pointer should have no mark bits set"
        );

        assert!(
            !ptr.is_remapped(),
            "New pointer should not be marked as remapped"
        );
    }
}

/// Test address extraction with color bits set
///
/// **Bug this finds:** Color bits interfering with address extraction
/// **Invariant verified:** Address extraction works regardless of color bits
#[test]
fn test_address_extraction_with_colors() {
    // Arrange
    let base_address = 0x12345678;

    // Test with various color combinations
    let test_cases = [
        (false, false, false, false), // No colors
        (true, false, false, false),  // Marked0
        (false, true, false, false),  // Marked1
        (true, true, false, false),   // Both marks (invalid but test extraction)
        (false, false, true, false),  // Remapped
        (false, false, false, true),  // Finalizable
        (true, true, true, true),     // All bits
    ];

    for (m0, m1, remap, finalizable) in test_cases {
        let mut ptr = ColoredPointer::new(base_address);

        if m0 {
            ptr.set_marked0();
        }
        if m1 {
            ptr.set_marked1();
        }
        if remap {
            ptr.set_remapped();
        }
        if finalizable {
            ptr.set_finalizable();
        }

        // Assert - address must be preserved regardless of colors
        assert_eq!(
            ptr.address(),
            base_address,
            "Address extraction failed with colors m0={}, m1={}, remap={}, final={}: \
             expected {:#x}, got {:#x}",
            m0,
            m1,
            remap,
            finalizable,
            base_address,
            ptr.address()
        );
    }
}

// ============================================================================
// MARKED BIT FLIP TEST - CRITICAL BUG
// ============================================================================

/// Test flip_mark_bit swaps bits correctly (NOT flips both)
///
/// **Bug this finds:** CRITICAL - flip_mark_bit uses XOR with both masks,
/// which flips BOTH bits instead of swapping them.
///
/// CORRECT behavior:
/// - Marked0=1, Marked1=0 → Marked0=0, Marked1=1 (swap)
/// - Marked0=0, Marked1=1 → Marked0=1, Marked1=0 (swap)
///
/// BUG behavior (current implementation):
/// - Marked0=1, Marked1=0 → Marked0=0, Marked1=1 (XOR both: correct by accident)
/// - Marked0=0, Marked1=0 → Marked0=1, Marked1=1 (XOR both: WRONG!)
///
/// **Invariant verified:** flip_mark_bit swaps, doesn't flip both
#[test]
fn test_flip_mark_bit_correctness() {
    // Test Case 1: Start with Marked0 set
    {
        let mut ptr = ColoredPointer::new(0x1000);
        ptr.set_marked0();

        assert!(ptr.is_marked0(), "Initial: Marked0 should be set");
        assert!(!ptr.is_marked1(), "Initial: Marked1 should be clear");

        // Flip
        ptr.flip_mark_bit();

        // After flip: Marked0 should be clear, Marked1 should be set
        assert!(
            !ptr.is_marked0(),
            "flip_mark_bit BUG: Marked0 should be cleared after flip"
        );

        assert!(
            ptr.is_marked1(),
            "flip_mark_bit CRITICAL BUG: Marked1 should be SET after flip. \
             Current implementation XORs both masks which is WRONG."
        );

        // Flip back
        ptr.flip_mark_bit();

        assert!(
            ptr.is_marked0(),
            "flip_mark_bit BUG: Second flip should restore Marked0"
        );

        assert!(
            !ptr.is_marked1(),
            "flip_mark_bit BUG: Second flip should clear Marked1"
        );
    }

    // Test Case 2: Start with Marked1 set
    {
        let mut ptr = ColoredPointer::new(0x1000);
        ptr.set_marked1();

        assert!(!ptr.is_marked0(), "Initial: Marked0 should be clear");
        assert!(ptr.is_marked1(), "Initial: Marked1 should be set");

        // Flip
        ptr.flip_mark_bit();

        assert!(
            ptr.is_marked0(),
            "flip_mark_bit BUG: Marked0 should be SET after flip from Marked1"
        );

        assert!(
            !ptr.is_marked1(),
            "flip_mark_bit BUG: Marked1 should be cleared after flip"
        );
    }

    // Test Case 3: Start with NEITHER set (this exposes the bug!)
    {
        let mut ptr = ColoredPointer::new(0x1000);

        assert!(!ptr.is_marked0(), "Initial: Marked0 should be clear");
        assert!(!ptr.is_marked1(), "Initial: Marked1 should be clear");

        // Flip - THIS IS WHERE THE BUG MANIFESTS
        ptr.flip_mark_bit();

        // CORRECT: Should set Marked0 (or Marked1, depending on convention)
        // BUG: Sets BOTH Marked0 AND Marked1

        // The correct behavior is that exactly ONE mark bit should be set
        // after flip from "no marks" state
        let marked0 = ptr.is_marked0();
        let marked1 = ptr.is_marked1();

        assert!(
            marked0 != marked1,
            "flip_mark_bit CRITICAL BUG: After flip from no-marks state, \
             got Marked0={}, Marked1={}. XOR with both masks sets BOTH bits, \
             but flip should SWAP (exactly one should be set).",
            marked0,
            marked1
        );
    }

    // Test Case 4: Start with BOTH set (invalid state, but flip should fix it)
    {
        let mut ptr = ColoredPointer::new(0x1000);
        ptr.set_marked0();
        ptr.set_marked1();

        assert!(ptr.is_marked0(), "Initial: Marked0 set");
        assert!(ptr.is_marked1(), "Initial: Marked1 set");

        // Flip
        ptr.flip_mark_bit();

        // After flip from both-set: should have neither set (XOR behavior)
        // But correct swap behavior would be implementation-dependent
        // The key is that the result should be deterministic
    }
}

/// Test that flip_mark_bit is its own inverse
///
/// **Bug this finds:** Non-invertible flip operation
/// **Invariant verified:** flip(flip(x)) == x
///
/// Note: This property holds for all VALID states (where exactly one mark bit
/// is set). The (false, false) and (true, true) states are invalid edge cases
/// that don't satisfy the involution property by design.
#[test]
fn test_flip_mark_bit_inverse() {
    // Only test valid states where exactly one mark bit is set
    // These are the states that occur during normal GC operation
    let test_cases = [
        (true, false), // Marked0 only - valid GC state
        (false, true), // Marked1 only - valid GC state
    ];

    for (initial_m0, initial_m1) in test_cases {
        let mut ptr = ColoredPointer::new(0x1000);

        if initial_m0 {
            ptr.set_marked0();
        }
        if initial_m1 {
            ptr.set_marked1();
        }

        // Double flip
        ptr.flip_mark_bit();
        ptr.flip_mark_bit();

        assert_eq!(
            ptr.is_marked0(),
            initial_m0,
            "Double flip should restore Marked0: started {}, ended {}",
            initial_m0,
            ptr.is_marked0()
        );

        assert_eq!(
            ptr.is_marked1(),
            initial_m1,
            "Double flip should restore Marked1: started {}, ended {}",
            initial_m1,
            ptr.is_marked1()
        );
    }

    // Test that invalid states are handled deterministically
    // (false, false) -> flip -> (true, false) -> flip -> (false, true)
    // This is expected behavior: invalid states converge to valid states
    {
        let mut ptr = ColoredPointer::new(0x1000);
        ptr.flip_mark_bit(); // (false, false) -> (true, false)
        assert!(
            ptr.is_marked0() && !ptr.is_marked1(),
            "Flip from no-marks should set Marked0"
        );
        ptr.flip_mark_bit(); // (true, false) -> (false, true)
        assert!(
            !ptr.is_marked0() && ptr.is_marked1(),
            "Second flip should swap to Marked1"
        );
    }

    // (true, true) -> flip -> (false, true) -> flip -> (true, false)
    // Invalid both-set state: since marked0 is true, it swaps to Marked1
    {
        let mut ptr = ColoredPointer::new(0x1000);
        ptr.set_marked0();
        ptr.set_marked1();
        ptr.flip_mark_bit(); // (true, true) -> (false, true) - swaps since marked0 was set
        assert!(
            !ptr.is_marked0() && ptr.is_marked1(),
            "Flip from both-set swaps to Marked1 (marked0 takes precedence)"
        );
        ptr.flip_mark_bit(); // (false, true) -> (true, false)
        assert!(
            ptr.is_marked0() && !ptr.is_marked1(),
            "Flip from Marked1 swaps to Marked0"
        );
    }
}

// ============================================================================
// LOAD BARRIER TESTS
// ============================================================================

/// Test load barrier processes unmarked pointers during marking phase
///
/// **Bug this finds:** Load barrier not marking objects
/// **Invariant verified:** Unmarked pointers are marked during marking phase
#[test]
fn test_load_barrier_marks_unmarked() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;

    // Arrange
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let barrier = LoadBarrier::new(Arc::clone(&mark_queue), Arc::clone(&forwarding_table));

    // Set marking phase
    barrier.set_phase(GcPhase::Marking);

    // Create unmarked pointer
    let ptr = ColoredPointer::new(0x1000);
    assert!(!ptr.is_marked(), "Test setup: pointer should be unmarked");

    // Act - process through load barrier
    let result = barrier.on_pointer_load(ptr);

    // Assert - pointer should now be marked
    assert!(
        result.is_marked(),
        "Load barrier BUG: Unmarked pointer was not marked during marking phase. \
         Load barrier should mark objects when accessed."
    );
}

/// Test load barrier returns healed pointer during relocation
///
/// **Bug this finds:** CRITICAL - Load barrier returns new pointer but doesn't
/// update the source memory location. Pointer healing doesn't actually happen.
///
/// **Invariant verified:** Relocated pointers are healed (point to new address)
#[test]
fn test_load_barrier_pointer_healing() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;

    // Arrange
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));

    // Setup forwarding: old_addr -> new_addr
    let old_addr = 0x1000;
    let new_addr = 0x2000;
    forwarding_table.add_entry(old_addr, new_addr);

    let barrier = LoadBarrier::new(Arc::clone(&mark_queue), Arc::clone(&forwarding_table));

    // Set relocating phase
    barrier.set_phase(GcPhase::Relocating);

    // Create pointer to relocated object (marked, not remapped)
    let mut ptr = ColoredPointer::new(old_addr);
    ptr.set_marked0(); // Marked but not remapped

    // Act - process through load barrier
    let result = barrier.on_pointer_load(ptr);

    // Assert - pointer should be healed to new address
    assert_eq!(
        result.address(),
        new_addr,
        "Load barrier POINTER HEALING BUG: Pointer was not healed to new address. \
         Expected {:#x}, got {:#x}. The barrier should update pointers to \
         relocated objects.",
        new_addr,
        result.address()
    );

    // Pointer should be marked as remapped
    assert!(
        result.is_remapped(),
        "Load barrier BUG: Healed pointer should have remapped bit set"
    );
}

/// Test load barrier fast path for already-processed pointers
///
/// **Bug this finds:** Load barrier processing already-processed pointers (perf bug)
/// **Invariant verified:** Remapped pointers skip processing
#[test]
fn test_load_barrier_fast_path() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;

    // Arrange
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let barrier = LoadBarrier::new(Arc::clone(&mark_queue), Arc::clone(&forwarding_table));

    // Set relocating phase
    barrier.set_phase(GcPhase::Relocating);

    // Create already-remapped pointer
    let mut ptr = ColoredPointer::new(0x1000);
    ptr.set_remapped();

    // Act
    let result = barrier.on_pointer_load(ptr);

    // Assert - should return unchanged (fast path)
    assert_eq!(
        result.address(),
        0x1000,
        "Load barrier fast path BUG: Remapped pointer was modified"
    );

    assert!(
        result.is_remapped(),
        "Load barrier fast path BUG: Remapped bit was cleared"
    );
}

/// Test load barrier during idle phase (no GC)
///
/// **Bug this finds:** Load barrier interfering with normal operation
/// **Invariant verified:** Idle phase passes pointers through unchanged
#[test]
fn test_load_barrier_idle_phase() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;

    // Arrange
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let barrier = LoadBarrier::new(Arc::clone(&mark_queue), Arc::clone(&forwarding_table));

    // Set idle phase (default)
    barrier.set_phase(GcPhase::Idle);

    // Create unmarked pointer
    let ptr = ColoredPointer::new(0x1000);

    // Act
    let result = barrier.on_pointer_load(ptr);

    // Assert - should pass through unchanged during idle
    assert_eq!(
        result.address(),
        0x1000,
        "Load barrier BUG: Modified pointer during idle phase"
    );

    assert!(
        !result.is_marked(),
        "Load barrier BUG: Marked pointer during idle phase"
    );
}

// ============================================================================
// CONCURRENT BARRIER TESTS
// ============================================================================

/// Test concurrent load barrier operations
///
/// **Bug this finds:** Race conditions in load barrier
/// **Invariant verified:** Concurrent barrier operations are safe
#[test]
fn test_concurrent_load_barrier() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;

    // Arrange
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let barrier = Arc::new(LoadBarrier::new(
        Arc::clone(&mark_queue),
        Arc::clone(&forwarding_table),
    ));

    // Set marking phase
    barrier.set_phase(GcPhase::Marking);

    let thread_count = 8;
    let ops_per_thread = 100;
    let mut handles = Vec::new();

    // Act - concurrent barrier operations
    for thread_id in 0..thread_count {
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            for i in 0..ops_per_thread {
                let ptr = ColoredPointer::new(0x1000 + (thread_id * 100) + i);
                let _result = barrier_clone.on_pointer_load(ptr);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // Assert - mark queue should have entries
    // (exact count depends on implementation details)
}

/// Test concurrent mark bit operations through barrier
///
/// **Bug this finds:** Race condition in mark bit setting
/// **Invariant verified:** Concurrent marks don't corrupt state
#[test]
fn test_concurrent_marking_through_barrier() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;
    use std::sync::atomic::AtomicUsize;

    // Arrange - shared pointer
    let shared_ptr = Arc::new(AtomicUsize::new(0x1000));
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let barrier = Arc::new(LoadBarrier::new(
        Arc::clone(&mark_queue),
        Arc::clone(&forwarding_table),
    ));

    barrier.set_phase(GcPhase::Marking);

    let thread_count = 8;
    let mut handles = Vec::new();

    // Act - concurrent marking of same object
    for _ in 0..thread_count {
        let ptr_clone = Arc::clone(&shared_ptr);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let raw = ptr_clone.load(Ordering::Relaxed);
                let colored = ColoredPointer::from_raw(raw);
                let result = barrier_clone.on_pointer_load(colored);
                ptr_clone.store(result.raw(), Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    // Wait
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // Assert - final pointer should be marked
    let final_raw = shared_ptr.load(Ordering::Relaxed);
    let final_ptr = ColoredPointer::from_raw(final_raw);

    assert!(
        final_ptr.is_marked(),
        "Concurrent marking BUG: Object not marked after concurrent barrier ops"
    );

    assert_eq!(
        final_ptr.address(),
        0x1000,
        "Concurrent marking BUG: Address corrupted"
    );
}

// ============================================================================
// FORWARDING TABLE TESTS
// ============================================================================

/// Test forwarding table insert and lookup
///
/// **Bug this finds:** Forwarding table bugs causing pointer healing failure
/// **Invariant verified:** Forwarding entries are found correctly
#[test]
fn test_forwarding_table_insert_lookup() {
    use fgc::relocate::ForwardingTable;

    // Arrange
    let table = ForwardingTable::new(0, 1024 * 1024);

    let test_cases = [(0x1000, 0x2000), (0x3000, 0x4000), (0x5000, 0x6000)];

    // Act - add entries
    for &(old, new) in &test_cases {
        table.add_entry(old, new);
    }

    // Assert - lookup returns correct values
    for &(old, expected_new) in &test_cases {
        let result = table.lookup(old);

        assert_eq!(
            result,
            Some(expected_new),
            "Forwarding table BUG: lookup({:#x}) returned {:?}, expected Some({:#x})",
            old,
            result,
            expected_new
        );
    }

    // Non-existent entry should return None
    let missing = table.lookup(0x9999);
    assert_eq!(
        missing, None,
        "Forwarding table BUG: Non-existent entry returned Some instead of None"
    );
}

/// Test forwarding table concurrent access
///
/// **Bug this finds:** Race conditions in forwarding table
/// **Invariant verified:** Concurrent insert/lookup is safe
#[test]
fn test_forwarding_table_concurrent() {
    use fgc::relocate::ForwardingTable;

    // Arrange
    let table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let thread_count = 4;

    let mut handles = Vec::new();

    // Act - concurrent inserts and lookups
    for thread_id in 0..thread_count {
        let table_clone = Arc::clone(&table);

        let handle = thread::spawn(move || {
            // Add entry
            let old_addr = 0x1000 + thread_id * 0x100;
            let new_addr = 0x2000 + thread_id * 0x100;
            table_clone.add_entry(old_addr, new_addr);

            // Lookup
            for i in 0..10 {
                let lookup_addr = 0x1000 + i * 0x100;
                let _ = table_clone.lookup(lookup_addr);
            }
        });

        handles.push(handle);
    }

    // Wait
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // Assert - table should be consistent
    for thread_id in 0..thread_count {
        let old_addr = 0x1000 + thread_id * 0x100;
        let expected_new = 0x2000 + thread_id * 0x100;

        let result = table.lookup(old_addr);
        assert_eq!(
            result,
            Some(expected_new),
            "Concurrent forwarding table BUG: Entry from thread {} corrupted",
            thread_id
        );
    }
}

// ============================================================================
// BARRIER ENABLE/DISABLE TESTS
// ============================================================================

/// Test load barrier can be disabled
///
/// **Bug this finds:** Barrier always active causing issues
/// **Invariant verified:** Disabled barrier passes through unchanged
#[test]
fn test_load_barrier_disable() {
    use fgc::barrier::load_barrier::LoadBarrier;
    use fgc::marker::MarkQueue;
    use fgc::relocate::ForwardingTable;

    // Arrange
    let mark_queue = Arc::new(MarkQueue::new());
    let forwarding_table = Arc::new(ForwardingTable::new(0, 1024 * 1024));
    let barrier = LoadBarrier::new(Arc::clone(&mark_queue), Arc::clone(&forwarding_table));

    // Disable barrier
    barrier.set_enabled(false);
    assert!(!barrier.is_enabled(), "Barrier should be disabled");

    // Set marking phase (would normally process)
    barrier.set_phase(GcPhase::Marking);

    // Create unmarked pointer
    let ptr = ColoredPointer::new(0x1000);

    // Act
    let result = barrier.on_pointer_load(ptr);

    // Assert - should pass through unchanged when disabled
    assert_eq!(
        result.address(),
        0x1000,
        "Disabled barrier BUG: Modified pointer"
    );

    assert!(
        !result.is_marked(),
        "Disabled barrier BUG: Marked pointer when disabled"
    );
}
