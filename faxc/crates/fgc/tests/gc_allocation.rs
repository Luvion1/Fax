//! GC Allocation Tests - Strict Invariants for Memory Allocation
//!
//! These tests verify that the allocator maintains critical invariants:
//! - Unique addresses (ZERO tolerance for duplicates)
//! - Proper alignment
//! - Heap bounds
//! - Monotonic allocation (for bump allocator)
//!
//! ============================================================================
//! EACH TEST FINDS SPECIFIC BUGS - DO NOT WEAKEN ASSERTIONS
//! ============================================================================

mod common;

use common::{
    GcFixture,
    assert_all_addresses_unique,
    assert_address_aligned,
    assert_address_in_bounds,
    assert_addresses_monotonic,
    DEFAULT_ALIGNMENT,
};
use std::collections::HashSet;

/// ============================================================================
/// BASIC ALLOCATION TESTS
/// ============================================================================

/// Test that allocation returns valid, non-null addresses
///
/// **Bug this finds:** Null pointer returns, allocation failure without error
/// **Invariant verified:** Allocated address must be > 0
#[test]
fn test_allocation_returns_valid_address() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Act
    let addr = fixture.allocate(64);
    
    // Assert - strict verification
    assert!(
        addr > 0,
        "Null pointer (0) returned for 64-byte allocation - allocator returned invalid address"
    );
    
    // Address should be reasonable (not absurdly large)
    assert!(
        addr < usize::MAX / 2,
        "Suspicious address {:#x} - possible integer overflow or corruption",
        addr
    );
}

/// Test that allocation respects alignment requirements
///
/// **Bug this finds:** Alignment bugs that cause SIGBUS or performance issues
/// **Invariant verified:** All addresses must be 8-byte aligned (minimum)
#[test]
fn test_allocation_respects_alignment() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let sizes = [1, 7, 8, 15, 16, 31, 32, 64, 127, 128, 255, 256];
    
    // Act & Assert
    for &size in &sizes {
        let addr = fixture.allocate(size);
        
        assert_address_aligned(addr, DEFAULT_ALIGNMENT, 
            &format!("Allocation of {} bytes", size));
    }
}

/// Test that allocated addresses are within heap bounds
///
/// **Bug this finds:** Heap overflow, address calculation bugs
/// **Invariant verified:** All addresses must be within [base, base+size)
#[test]
fn test_allocation_within_heap_bounds() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let heap = fixture.gc.heap();
    let heap_base = heap.base_address();
    let heap_size = heap.max_size();
    
    // Act - allocate multiple objects
    let addresses = fixture.allocate_many(100, 64);
    
    // Assert - every address must be in bounds
    for (i, &addr) in addresses.iter().enumerate() {
        assert_address_in_bounds(addr, heap_base, heap_size,
            &format!("Allocation #{} ({} bytes)", i, 64));
    }
}

/// Test that sequential allocations return unique addresses
///
/// **Bug this finds:** Bump pointer not advancing, address reuse bug
/// **Invariant verified:** ZERO duplicates allowed
#[test]
fn test_sequential_allocations_unique() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let allocation_count = 1000;
    let allocation_size = 64;
    
    // Act
    let addresses = fixture.allocate_many(allocation_count, allocation_size);
    
    // Assert - strict uniqueness check
    assert_all_addresses_unique(&addresses, 
        "Sequential allocations");
    
    // Additional: verify we got exactly the expected count
    assert_eq!(
        addresses.len(),
        allocation_count,
        "Expected {} allocations, got {}",
        allocation_count,
        addresses.len()
    );
}

/// Test that addresses increase monotonically (bump allocator property)
///
/// **Bug this finds:** Bump pointer regression, allocation order corruption
/// **Invariant verified:** Each allocation address >= previous
#[test]
fn test_bump_allocator_monotonic() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let allocation_count = 100;
    let allocation_size = 64;
    
    // Act
    let addresses = fixture.allocate_many(allocation_count, allocation_size);
    
    // Assert - monotonic increase
    assert_addresses_monotonic(&addresses, 
        "Bump allocator should produce monotonically increasing addresses");
    
    // Additional: verify addresses actually increase (not all same)
    let unique: HashSet<_> = addresses.iter().collect();
    assert!(
        unique.len() > 1,
        "All addresses are identical ({:#x}) - bump pointer not advancing",
        addresses[0]
    );
}

/// ============================================================================
/// SIZE VARIATION TESTS
/// ============================================================================

/// Test allocation with various sizes
///
/// **Bug this finds:** Size-dependent bugs, edge case handling
/// **Invariant verified:** All sizes produce valid, unique addresses
#[test]
fn test_various_allocation_sizes() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Test sizes: edge cases and common sizes
    let test_sizes = [
        1,      // Minimum
        8,      // Word size
        16,     // Common small alloc
        64,     // Cache line
        128,    // Small object
        256,    // Threshold
        512,    // Medium
        1024,   // 1KB
        4096,   // Page size
        8192,   // 2 pages
        65536,  // 64KB
    ];
    
    // Act & Assert
    let mut all_addresses = Vec::new();
    for &size in &test_sizes {
        let addr = fixture.allocate(size);
        
        assert_address_aligned(addr, DEFAULT_ALIGNMENT,
            &format!("{}-byte allocation", size));
        
        assert!(addr > 0, 
            "Null pointer for {}-byte allocation", size);
        
        all_addresses.push(addr);
    }
    
    // All addresses should be unique
    assert_all_addresses_unique(&all_addresses, 
        "Various size allocations");
}

/// Test that large allocations still work correctly
///
/// **Bug this finds:** Size overflow, large object handling bugs
/// **Invariant verified:** Large allocations produce valid addresses
#[test]
fn test_large_allocation() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let large_size = 1024 * 1024; // 1MB
    
    // Act
    let addr = fixture.allocate(large_size);
    
    // Assert
    assert!(
        addr > 0,
        "Null pointer for {}-byte (1MB) allocation",
        large_size
    );
    
    assert_address_aligned(addr, DEFAULT_ALIGNMENT,
        &format!("{}-byte (1MB) allocation", large_size));
}

/// ============================================================================
/// CONCURRENT ALLOCATION TESTS
/// ============================================================================

/// Test concurrent allocations from multiple threads
///
/// **Bug this finds:** Race conditions in allocator, thread safety issues
/// **Invariant verified:** ZERO duplicate addresses across all threads
#[test]
fn test_concurrent_allocations_unique() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let thread_count = 8;
    let allocations_per_thread = 100;
    let allocation_size = 64;
    
    // Act - run concurrent allocations
    common::run_concurrent_allocations(
        &fixture,
        thread_count,
        allocations_per_thread,
        allocation_size,
        move |all_addresses: Vec<usize>| {
            // Assert - strict uniqueness across ALL threads
            assert_all_addresses_unique(&all_addresses,
                &format!("Concurrent allocations ({} threads × {} allocs)",
                    thread_count, allocations_per_thread));
        },
    );
}

/// Test concurrent allocations maintain alignment
///
/// **Bug this finds:** Race conditions causing misalignment
/// **Invariant verified:** All concurrent allocation results are aligned
#[test]
fn test_concurrent_allocations_aligned() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let thread_count = 4;
    let allocations_per_thread = 50;
    let allocation_size = 32;
    
    // Act
    common::run_concurrent_allocations(
        &fixture,
        thread_count,
        allocations_per_thread,
        allocation_size,
        |all_addresses: Vec<usize>| {
            // Assert - every address must be aligned
            for (i, &addr) in all_addresses.iter().enumerate() {
                assert_address_aligned(addr, DEFAULT_ALIGNMENT,
                    &format!("Concurrent allocation #{}", i));
            }
        },
    );
}

/// Test concurrent allocations stay within heap bounds
///
/// **Bug this finds:** Race conditions causing heap overflow
/// **Invariant verified:** All addresses within heap bounds
#[test]
fn test_concurrent_allocations_in_bounds() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let heap = fixture.gc.heap();
    let heap_base = heap.base_address();
    let heap_size = heap.max_size();
    let thread_count = 4;
    let allocations_per_thread = 50;
    
    // Act
    common::run_concurrent_allocations(
        &fixture,
        thread_count,
        allocations_per_thread,
        64,
        move |all_addresses: Vec<usize>| {
            // Assert - every address in bounds
            for (i, &addr) in all_addresses.iter().enumerate() {
                assert_address_in_bounds(addr, heap_base, heap_size,
                    &format!("Concurrent allocation #{}", i));
            }
        },
    );
}

/// Test high-contention concurrent allocation
///
/// **Bug this finds:** Lock contention bugs, starvation, deadlock
/// **Invariant verified:** All allocations complete successfully
#[test]
fn test_high_contention_allocation() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let thread_count = 16; // High contention
    let allocations_per_thread = 50;
    let allocation_size = 16; // Small, fast allocations
    
    // Act & Assert - should complete without deadlock
    common::assert_completed_within_timeout(
        || {
            common::run_concurrent_allocations(
                &fixture,
                thread_count,
                allocations_per_thread,
                allocation_size,
                |all_addresses| {
                    assert_all_addresses_unique(&all_addresses,
                        "High-contention allocations");
                },
            );
        },
        common::TEST_TIMEOUT,
        "High-contention allocation test"
    );
}

/// ============================================================================
/// ALLOCATION PATTERN TESTS
/// ============================================================================

/// Test interleaved small and large allocations
///
/// **Bug this finds:** Size-dependent state corruption
/// **Invariant verified:** All allocations valid regardless of pattern
#[test]
fn test_interleaved_size_allocations() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Act - interleave small and large
    let mut addresses = Vec::new();
    for i in 0..50 {
        let size = if i % 2 == 0 { 16 } else { 4096 };
        addresses.push(fixture.allocate(size));
    }
    
    // Assert
    assert_all_addresses_unique(&addresses, 
        "Interleaved size allocations");
    
    for (i, &addr) in addresses.iter().enumerate() {
        assert_address_aligned(addr, DEFAULT_ALIGNMENT,
            &format!("Interleaved allocation #{}", i));
    }
}

/// Test repeated same-size allocations
///
/// **Bug this finds:** Size caching bugs, bump pointer stuck
/// **Invariant verified:** Same-size allocations produce unique addresses
#[test]
fn test_repeated_same_size_allocations() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let allocation_size = 128;
    let count = 500;
    
    // Act
    let addresses = fixture.allocate_many(count, allocation_size);
    
    // Assert
    assert_all_addresses_unique(&addresses,
        &format!("Repeated {}-byte allocations", allocation_size));
    
    // Verify bump pointer advanced
    let unique: HashSet<_> = addresses.iter().collect();
    assert_eq!(unique.len(), count,
        "Bump pointer stuck: only {} unique addresses from {} allocations",
        unique.len(), count);
}
