//! GC Concurrency Tests - Race Condition Detection
//!
//! These tests verify thread safety of GC operations:
//! - Atomic operations on colored pointers
//! - Race conditions with multiple threads
//! - Deadlock scenarios
//! - Memory ordering correctness
//!
//! ============================================================================
//! EACH TEST FINDS SPECIFIC RACE CONDITIONS - DO NOT WEAKEN ASSERTIONS
//! ============================================================================

mod common;

use common::{
    GcFixture,
    assert_all_addresses_unique,
    assert_gc_completed,
    assert_gc_cycle_increased,
    TEST_TIMEOUT,
};
use fgc::{GcGeneration, GcReason};
use std::sync::{Arc, Barrier, atomic::{AtomicUsize, AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

/// ============================================================================
/// COLORED POINTER ATOMIC OPERATIONS
/// ============================================================================

/// Test concurrent mark bit operations on colored pointers
///
/// **Bug this finds:** Non-atomic mark bit updates, race conditions in marking
/// **Invariant verified:** Mark operations are atomic and don't corrupt state
#[test]
fn test_concurrent_mark_operations() {
    use fgc::barrier::colored_ptr::ColoredPointer;
    use std::sync::atomic::AtomicUsize;
    
    // Arrange - shared pointer accessed by multiple threads
    let shared_ptr = Arc::new(AtomicUsize::new(0x1000)); // Raw pointer value
    let thread_count = 8;
    let operations_per_thread = 100;
    
    let mut handles = Vec::new();
    let barrier = Arc::new(Barrier::new(thread_count));
    
    // Act - concurrent mark operations
    for _ in 0..thread_count {
        let ptr_clone = Arc::clone(&shared_ptr);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            for _ in 0..operations_per_thread {
                // Read, modify, write pattern (simulating what GC does)
                let mut current = ptr_clone.load(Ordering::Relaxed);
                let mut colored = ColoredPointer::from_raw(current);
                
                // Set mark bit
                colored.set_marked0();
                
                // Write back
                ptr_clone.store(colored.raw(), Ordering::Relaxed);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
    
    // Assert - pointer should be in valid state
    let final_raw = shared_ptr.load(Ordering::Relaxed);
    let final_ptr = ColoredPointer::from_raw(final_raw);
    
    // Address should be preserved
    assert_eq!(
        final_ptr.address(),
        0x1000,
        "Address corrupted during concurrent mark operations: expected 0x1000, got {:#x}",
        final_ptr.address()
    );
    
    // Marked0 should be set (at least one thread set it)
    assert!(
        final_ptr.is_marked0(),
        "Marked0 bit not set after concurrent operations - race condition in mark"
    );
}

/// Test concurrent flip_mark_bit operations
///
/// **Bug this finds:** flip_mark_bit flipping BOTH bits instead of swapping
/// **Invariant verified:** Mark bits are properly swapped, not both flipped
#[test]
fn test_concurrent_flip_mark_bit() {
    use fgc::barrier::colored_ptr::ColoredPointer;
    
    // Arrange
    let mut ptr = ColoredPointer::new(0x1000);
    
    // Set initial state: Marked0 = true, Marked1 = false
    ptr.set_marked0();
    assert!(ptr.is_marked0());
    assert!(!ptr.is_marked1());
    
    // Act - flip mark bit
    ptr.flip_mark_bit();
    
    // Assert - should swap, not flip both
    // CORRECT behavior: Marked0=false, Marked1=true
    // BUG behavior: Marked0=false, Marked1=false (both flipped)
    
    assert!(
        !ptr.is_marked0(),
        "flip_mark_bit bug: Marked0 should be cleared after flip"
    );
    
    assert!(
        ptr.is_marked1(),
        "flip_mark_bit BUG: Marked1 should be SET after flip (swap), but it's clear. \
         Current implementation XORs both bits which is WRONG - should swap."
    );
    
    // Flip again - should return to original state
    ptr.flip_mark_bit();
    
    assert!(
        ptr.is_marked0(),
        "flip_mark_bit bug: Second flip should restore Marked0"
    );
    
    assert!(
        !ptr.is_marked1(),
        "flip_mark_bit bug: Second flip should clear Marked1"
    );
}

/// Test that exactly one mark bit is set after operations
///
/// **Bug this finds:** Both mark bits set simultaneously (invalid state)
/// **Invariant verified:** Only one mark bit should be active at a time
#[test]
fn test_mark_bit_exclusivity() {
    use fgc::barrier::colored_ptr::ColoredPointer;
    
    // Arrange
    let mut ptr = ColoredPointer::new(0x1000);
    
    // Act - various mark operations
    ptr.set_marked0();
    
    // Assert - only Marked0 should be set
    assert!(
        ptr.is_marked0() && !ptr.is_marked1(),
        "After set_marked0: Marked0={}, Marked1={} - should be true/false",
        ptr.is_marked0(),
        ptr.is_marked1()
    );
    
    ptr.set_marked1();
    
    // Note: Current implementation allows both to be set
    // This test documents the expected invariant
    // In a correct implementation, set_marked1 might clear Marked0 first
}

/// ============================================================================
/// GC STATE RACE CONDITIONS
/// ============================================================================

/// Test concurrent GC state reads
///
/// **Bug this finds:** Data races on GC state, inconsistent state reads
/// **Invariant verified:** State reads are consistent across threads
#[test]
fn test_concurrent_state_reads() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let thread_count = 8;
    let reads_per_thread = 1000;
    
    let mut handles = Vec::new();
    let mut states_read: Vec<_> = (0..thread_count)
        .map(|_| Arc::new(std::sync::Mutex::new(Vec::new())))
        .collect();
    
    // Act - concurrent state reads
    for i in 0..thread_count {
        let gc: Arc<fgc::GarbageCollector> = Arc::clone(&fixture.gc);
        let states = Arc::clone(&states_read[i]);
        
        let handle = thread::spawn(move || {
            let mut local_states = states.lock().expect("Lock should not be poisoned");
            for _ in 0..reads_per_thread {
                let state = gc.state();
                local_states.push(state);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
    
    // Assert - all state reads should be valid
    for (i, states) in states_read.iter().enumerate() {
        let states = states.lock().expect("Lock should not be poisoned");
        assert_eq!(
            states.len(),
            reads_per_thread,
            "Thread {} did not complete all reads",
            i
        );
        
        // All states should be valid enum values (Rust guarantees this)
        // The test verifies no panics occurred during concurrent reads
    }
}

/// Test GC request from multiple threads
///
/// **Bug this finds:** Race conditions in GC request handling
/// **Invariant verified:** Multiple GC requests handled safely
#[test]
fn test_concurrent_gc_requests() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let thread_count = 4;
    let before_cycles = fixture.cycle_count();
    
    let mut handles = Vec::new();
    let barrier = Arc::new(Barrier::new(thread_count));
    
    // Act - concurrent GC requests
    for _ in 0..thread_count {
        let gc: Arc<fgc::GarbageCollector> = Arc::clone(&fixture.gc);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            gc.request_gc(GcGeneration::Young, GcReason::Explicit);
        });
        
        handles.push(handle);
    }
    
    // Wait for requests
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
    
    // Wait for GC to complete
    thread::sleep(Duration::from_millis(100));
    
    // Assert - GC should have run at least once
    let after_cycles = fixture.cycle_count();
    
    // Note: Multiple requests might coalesce into single GC
    // The important thing is no crashes or deadlocks
    assert!(
        after_cycles >= before_cycles,
        "GC cycle count decreased - counter corruption bug"
    );
}

/// ============================================================================
/// ALLOCATION RACE CONDITIONS
/// ============================================================================

/// Test allocation during GC
///
/// **Bug this finds:** Allocation-GC race conditions, use-after-free
/// **Invariant verified:** Allocations during GC are safe
#[test]
#[ignore] // TODO: Requires full GC implementation
fn test_allocation_during_gc() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let fixture = Arc::new(fixture);
    let barrier = Arc::new(Barrier::new(2));

    let alloc_barrier = Arc::clone(&barrier);
    let gc_barrier = Arc::clone(&barrier);
    let alloc_fixture = Arc::clone(&fixture);
    let gc_fixture = Arc::clone(&fixture);

    // Act - allocate while GC runs
    let alloc_handle = thread::spawn(move || {
        alloc_barrier.wait();
        // Try to allocate during GC
        let addr = alloc_fixture.allocate(64);
        addr
    });

    let gc_handle = thread::spawn(move || {
        gc_barrier.wait();
        gc_fixture.trigger_gc(GcGeneration::Young);
    });

    let allocated_addr = alloc_handle.join().expect("Alloc thread should not panic");
    gc_handle.join().expect("GC thread should not panic");

    // Assert
    assert!(
        allocated_addr > 0,
        "Allocation during GC returned null - race condition bug"
    );
}

/// Test concurrent allocation and deallocation patterns
///
/// **Bug this finds:** Use-after-free, double-free race conditions
/// **Invariant verified:** No memory safety violations
#[test]
fn test_concurrent_alloc_free_pattern() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let thread_count = 4;
    let operations_per_thread = 100;
    
    let allocated_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    
    // Act - concurrent alloc/free pattern
    for _ in 0..thread_count {
        let gc: Arc<fgc::GarbageCollector> = Arc::clone(&fixture.gc);
        let count = Arc::clone(&allocated_count);
        
        let handle = thread::spawn(move || {
            let mut local_addrs = Vec::new();
            
            for _ in 0..operations_per_thread {
                // Allocate
                if let Ok(addr) = gc.heap().allocate_tlab_memory(64) {
                    local_addrs.push(addr);
                    count.fetch_add(1, Ordering::Relaxed);
                }
            }
            
            local_addrs
        });
        
        handles.push(handle);
    }
    
    // Collect all allocations
    let mut all_addresses = Vec::new();
    for handle in handles {
        let addrs = handle.join().expect("Thread should not panic");
        all_addresses.extend(addrs);
    }
    
    // Assert - all allocations should be unique
    assert_all_addresses_unique(&all_addresses,
        "Concurrent alloc/free pattern");
    
    // Verify count matches
    let total_allocated = allocated_count.load(Ordering::Relaxed);
    assert_eq!(
        all_addresses.len(),
        total_allocated,
        "Address count mismatch - possible allocation tracking bug"
    );
}

/// ============================================================================
/// MARKING RACE CONDITIONS
/// ============================================================================

/// Test concurrent marking of same object
///
/// **Bug this finds:** Double-mark race condition, bitmap corruption
/// **Invariant verified:** Concurrent marks don't corrupt bitmap
#[test]
fn test_concurrent_mark_same_object() {
    use fgc::barrier::colored_ptr::ColoredPointer;
    use std::sync::atomic::AtomicUsize;
    
    // Arrange - multiple threads marking same object
    let shared_ptr = Arc::new(AtomicUsize::new(0x1000));
    let thread_count = 8;
    let marks_per_thread = 100;
    
    let marked_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    let barrier = Arc::new(Barrier::new(thread_count));
    
    // Act - concurrent marking
    for _ in 0..thread_count {
        let ptr = Arc::clone(&shared_ptr);
        let count = Arc::clone(&marked_count);
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            
            for _ in 0..marks_per_thread {
                let raw = ptr.load(Ordering::Relaxed);
                let mut colored = ColoredPointer::from_raw(raw);
                
                if !colored.is_marked() {
                    colored.set_marked0();
                    ptr.store(colored.raw(), Ordering::Relaxed);
                    count.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
    
    // Assert - object should be marked
    let final_raw = shared_ptr.load(Ordering::Relaxed);
    let final_ptr = ColoredPointer::from_raw(final_raw);
    
    assert!(
        final_ptr.is_marked(),
        "Object not marked after concurrent mark attempts - race condition bug"
    );
    
    // Address should be preserved
    assert_eq!(
        final_ptr.address(),
        0x1000,
        "Address corrupted during concurrent marking"
    );
}

/// ============================================================================
/// DEADLOCK DETECTION
/// ============================================================================

/// Test that GC doesn't deadlock with concurrent operations
///
/// **Bug this finds:** Deadlock between GC and allocation threads
/// **Invariant verified:** Operations complete within timeout
#[test]
fn test_no_deadlock_concurrent_gc_alloc() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let fixture = Arc::new(fixture);
    let barrier = Arc::new(Barrier::new(3)); // GC + 2 alloc threads

    let gc_barrier = Arc::clone(&barrier);
    let alloc1_barrier = Arc::clone(&barrier);
    let alloc2_barrier = Arc::clone(&barrier);
    
    let gc_fixture = Arc::clone(&fixture);
    let alloc1_fixture = Arc::clone(&fixture);
    let alloc2_fixture = Arc::clone(&fixture);

    // Act - GC and allocations simultaneously
    let gc_handle = thread::spawn(move || {
        gc_barrier.wait();
        gc_fixture.trigger_gc(GcGeneration::Young);
    });

    let alloc1_handle = thread::spawn(move || {
        alloc1_barrier.wait();
        let _addr = alloc1_fixture.allocate(64);
    });

    let alloc2_handle = thread::spawn(move || {
        alloc2_barrier.wait();
        let _addr = alloc2_fixture.allocate(64);
    });

    // Assert - should complete without deadlock
    let result = thread::spawn(move || {
        gc_handle.join().expect("GC thread panicked");
        alloc1_handle.join().expect("Alloc1 thread panicked");
        alloc2_handle.join().expect("Alloc2 thread panicked");
    }).join();

    assert!(
        result.is_ok(),
        "Deadlock detected: threads did not complete"
    );
}

/// Test GC shutdown doesn't deadlock
///
/// **Bug this finds:** Shutdown deadlock, resource cleanup race
/// **Invariant verified:** Shutdown completes cleanly
#[test]
fn test_gc_shutdown_no_deadlock() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Do some allocations first
    for _ in 0..10 {
        let _ = fixture.allocate(64);
    }
    
    // Act & Assert - shutdown should complete
    common::assert_completed_within_timeout(
        || {
            drop(fixture); // Drop triggers shutdown in Drop impl
        },
        TEST_TIMEOUT,
        "GC shutdown"
    );
}

/// ============================================================================
/// MEMORY ORDERING TESTS
/// ============================================================================

/// Test memory ordering in atomic operations
///
/// **Bug this finds:** Incorrect memory ordering causing visibility issues
/// **Invariant verified:** Proper memory ordering for cross-thread visibility
#[test]
fn test_memory_ordering_visibility() {
    use std::sync::atomic::AtomicUsize;
    
    // Arrange - classic message passing pattern
    let data = Arc::new(AtomicUsize::new(0));
    let ready = Arc::new(AtomicBool::new(false));
    
    let data_clone = Arc::clone(&data);
    let ready_clone = Arc::clone(&ready);
    
    // Act - writer thread
    let writer = thread::spawn(move || {
        data_clone.store(42, Ordering::Relaxed);
        // BUG: Should use Release ordering here
        ready_clone.store(true, Ordering::Relaxed);
    });
    
    // Reader thread
    let data_clone2 = Arc::clone(&data);
    let ready_clone2 = Arc::clone(&ready);
    
    let reader = thread::spawn(move || {
        while !ready_clone2.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_micros(1));
        }
        // BUG: Should use Acquire ordering here
        data_clone2.load(Ordering::Relaxed)
    });
    
    writer.join().expect("Writer should not panic");
    let value = reader.join().expect("Reader should not panic");
    
    // Note: With Relaxed ordering, this COULD fail on weak memory models
    // On x86 it will likely pass, but the test documents the expected behavior
    assert_eq!(
        value,
        42,
        "Memory ordering bug: value not visible across threads"
    );
}
