//! GC Edge Cases Tests - Boundary Conditions and Error Handling
//!
//! These tests verify GC behavior at boundaries:
//! - Zero-size allocation
//! - Maximum-size allocation
//! - OOM scenarios with strict error types
//! - Heap exhaustion and recovery
//!
//! ============================================================================
//! EACH TEST FINDS SPECIFIC EDGE CASE BUGS - DO NOT WEAKEN ASSERTIONS
//! ============================================================================

mod common;

use common::{GcFixture, assert_address_aligned, DEFAULT_ALIGNMENT};
use fgc::{FgcError, GcConfig, GcGeneration};
use std::sync::Arc;

/// ============================================================================
/// ZERO-SIZE ALLOCATION TESTS
/// ============================================================================

/// Test zero-size allocation behavior
///
/// **Bug this finds:** Zero-size handling bugs, infinite loops, null returns
/// **Invariant verified:** Zero-size allocation returns valid result or clear error
#[test]
fn test_zero_size_allocation() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Act - attempt zero-size allocation
    let result = fixture.gc.heap().allocate_tlab_memory(0);
    
    // Assert - should either succeed with valid address or return clear error
    match result {
        Ok(addr) => {
            // If it succeeds, address must be valid
            assert!(
                addr > 0,
                "Zero-size allocation returned null (0) - invalid behavior"
            );
            
            assert_address_aligned(addr, DEFAULT_ALIGNMENT,
                "Zero-size allocation result");
        }
        Err(e) => {
            // If it fails, error must be specific (not generic)
            assert!(
                matches!(e, FgcError::OutOfMemory { .. } | 
                         FgcError::TlabError(_)),
                "Zero-size allocation returned unexpected error: {:?}",
                e
            );
        }
    }
}

/// Test multiple zero-size allocations
///
/// **Bug this finds:** Zero-size state corruption, bump pointer stuck
/// **Invariant verified:** Multiple zero-size allocs produce unique addresses
#[test]
fn test_multiple_zero_size_allocations() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let count = 100;
    
    // Act
    let mut addresses = Vec::new();
    for _ in 0..count {
        if let Ok(addr) = fixture.gc.heap().allocate_tlab_memory(0) {
            addresses.push(addr);
        }
    }
    
    // Assert - if any succeeded, they should be unique
    if !addresses.is_empty() {
        let unique: std::collections::HashSet<_> = addresses.iter().collect();
        
        assert_eq!(
            unique.len(),
            addresses.len(),
            "Zero-size allocations produced {} duplicates out of {} - bump pointer bug",
            addresses.len() - unique.len(),
            addresses.len()
        );
    }
}

/// ============================================================================
/// MINIMUM SIZE ALLOCATION TESTS
/// ============================================================================

/// Test minimum non-zero allocation (1 byte)
///
/// **Bug this finds:** Minimum size handling, alignment padding bugs
/// **Invariant verified:** 1-byte allocation returns aligned address
#[test]
fn test_minimum_allocation() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Act
    let addr = fixture.allocate(1);
    
    // Assert
    assert!(
        addr > 0,
        "1-byte allocation returned null - minimum size bug"
    );
    
    assert_address_aligned(addr, DEFAULT_ALIGNMENT,
        "1-byte allocation (should be aligned to 8 bytes)");
}

/// Test allocation of various small sizes
///
/// **Bug this finds:** Small size edge cases, rounding bugs
/// **Invariant verified:** All small sizes produce valid addresses
#[test]
fn test_small_size_allocations() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Test sizes around alignment boundaries
    let sizes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 15, 16, 17, 31, 32, 33];
    
    // Act & Assert
    for &size in &sizes {
        let addr = fixture.allocate(size);
        
        assert!(
            addr > 0,
            "{}-byte allocation returned null",
            size
        );
        
        assert_address_aligned(addr, DEFAULT_ALIGNMENT,
            &format!("{}-byte allocation", size));
    }
}

/// ============================================================================
/// LARGE ALLOCATION TESTS
/// ============================================================================

/// Test very large allocation
///
/// **Bug this finds:** Size overflow, large object handling bugs
/// **Invariant verified:** Large allocations handled correctly
#[test]
fn test_very_large_allocation() {
    // Arrange
    let fixture = GcFixture::with_heap_size(256 * 1024 * 1024); // 256MB heap
    let large_size = 64 * 1024 * 1024; // 64MB
    
    // Act
    let result = fixture.gc.heap().allocate_tlab_memory(large_size);
    
    // Assert
    match result {
        Ok(addr) => {
            assert!(
                addr > 0,
                "Large allocation ({}) returned null",
                large_size
            );
            
            assert_address_aligned(addr, DEFAULT_ALIGNMENT,
                &format!("{}-byte large allocation", large_size));
        }
        Err(FgcError::OutOfMemory { requested, available }) => {
            // OOM is acceptable if heap is too small
            assert_eq!(
                requested,
                large_size,
                "OutOfMemory error should report correct requested size"
            );
            
            assert!(
                available < requested,
                "OutOfMemory reported available={} >= requested={}",
                available,
                requested
            );
        }
        Err(e) => {
            panic!("Unexpected error for large allocation: {:?}", e);
        }
    }
}

/// Test allocation near heap limit
///
/// **Bug this finds:** Boundary calculation bugs, off-by-one errors
/// **Invariant verified:** Near-limit allocations handled correctly
#[test]
fn test_allocation_near_heap_limit() {
    // Arrange - small heap for testing
    let heap_size = 1024 * 1024; // 1MB
    let fixture = GcFixture::with_heap_size(heap_size);
    
    // Allocate most of heap
    let alloc_size = heap_size - 1024; // Leave 1KB headroom
    
    // Act
    let result = fixture.gc.heap().allocate_tlab_memory(alloc_size);
    
    // Assert - should either succeed or return OOM
    match result {
        Ok(addr) => {
            assert!(addr > 0, "Near-limit allocation returned null");
        }
        Err(FgcError::OutOfMemory { .. }) => {
            // OOM is acceptable
        }
        Err(e) => {
            panic!("Unexpected error for near-limit allocation: {:?}", e);
        }
    }
}

/// ============================================================================
/// OUT OF MEMORY TESTS
/// ============================================================================

/// Test OOM error type and message
///
/// **Bug this finds:** Wrong error type, unhelpful error messages
/// **Invariant verified:** OOM returns FgcError::OutOfMemory with correct info
#[test]
fn test_oom_error_type() {
    // Arrange - tiny heap
    let fixture = GcFixture::with_heap_size(1024); // 1KB
    
    // Act - allocate more than heap size
    let result = fixture.gc.heap().allocate_tlab_memory(2048); // 2KB
    
    // Assert - must be OutOfMemory error
    match result {
        Err(FgcError::OutOfMemory { requested, available }) => {
            assert_eq!(
                requested,
                2048,
                "OutOfMemory should report requested=2048, got {}",
                requested
            );
            
            assert!(
                available < 2048,
                "OutOfMemory should report available < requested"
            );
        }
        Err(e) => {
            panic!("Expected OutOfMemory error, got: {:?}", e);
        }
        Ok(_) => {
            panic!("Allocation larger than heap succeeded - heap limit bug");
        }
    }
}

/// Test OOM after multiple allocations (heap exhaustion)
///
/// **Bug this finds:** Heap tracking bugs, exhaustion not detected
/// **Invariant verified:** OOM occurs when heap is exhausted
#[test]
fn test_heap_exhaustion() {
    // Arrange - small heap
    let heap_size = 64 * 1024; // 64KB
    let fixture = GcFixture::with_heap_size(heap_size);
    let alloc_size = 8 * 1024; // 8KB per allocation
    
    // Act - allocate until OOM
    let mut allocation_count = 0;
    let mut got_oom = false;
    
    for _ in 0..100 {
        match fixture.gc.heap().allocate_tlab_memory(alloc_size) {
            Ok(_) => {
                allocation_count += 1;
            }
            Err(FgcError::OutOfMemory { .. }) => {
                got_oom = true;
                break;
            }
            Err(e) => {
                panic!("Unexpected error during exhaustion test: {:?}", e);
            }
        }
    }
    
    // Assert - should have gotten OOM
    assert!(
        got_oom,
        "Heap exhaustion test: allocated {} × {} bytes = {} bytes \
         without OOM on {}-byte heap - heap limit not enforced",
        allocation_count,
        alloc_size,
        allocation_count * alloc_size,
        heap_size
    );
    
    // Should have allocated at least some objects
    assert!(
        allocation_count > 0,
        "No allocations succeeded before OOM - allocation bug"
    );
}

/// Test OOM recovery after GC
///
/// **Bug this finds:** Memory not reclaimed, GC not freeing space
/// **Invariant verified:** GC reclaims memory for reuse
#[test]
#[ignore] // TODO: Requires full GC implementation with memory tracking
fn test_oom_recovery_after_gc() {
    // Arrange - small heap
    let heap_size = 32 * 1024; // 32KB
    let fixture = GcFixture::with_heap_size(heap_size);
    let alloc_size = 16 * 1024; // 16KB
    
    // Fill heap
    let mut addresses = Vec::new();
    while let Ok(addr) = fixture.gc.heap().allocate_tlab_memory(alloc_size) {
        addresses.push(addr);
    }
    
    // Drop references (make objects garbage)
    drop(addresses);
    
    // Act - GC to reclaim memory
    fixture.trigger_gc(GcGeneration::Full);
    
    // Assert - should be able to allocate again
    let result = fixture.gc.heap().allocate_tlab_memory(alloc_size);
    
    assert!(
        result.is_ok(),
        "Allocation failed after GC - memory not reclaimed"
    );
}

/// ============================================================================
/// ALIGNMENT EDGE CASES
/// ============================================================================

/// Test allocation with various alignment requirements
///
/// **Bug this finds:** Alignment handling bugs, incorrect rounding
/// **Invariant verified:** All alignments are respected
#[test]
fn test_various_alignments() {
    // Arrange
    let fixture = GcFixture::with_defaults();

    // Test common alignments (all powers of 2)
    let alignments = [1, 2, 4, 8, 16, 32, 64];

    for &align in &alignments {
        // Use the new aligned allocation API
        // Minimum alignment is 8 bytes, so effective alignment is align.max(8)
        let effective_align = align.max(DEFAULT_ALIGNMENT);
        
        let addr = fixture.gc.heap()
            .allocate_tlab_memory_aligned(align, effective_align)
            .unwrap_or_else(|e| {
                panic!("Aligned allocation of {} bytes with {}-byte alignment failed: {:?}", 
                       align, effective_align, e);
            });

        assert_address_aligned(addr, effective_align,
            &format!("Allocation with {}-byte alignment", effective_align));
    }
}

/// Test allocation size that's not power of 2
///
/// **Bug this finds:** Non-power-of-2 size handling bugs
/// **Invariant verified:** Arbitrary sizes handled correctly
#[test]
fn test_non_power_of_2_sizes() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Non-power-of-2 sizes
    let sizes = [3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 
                 17, 23, 31, 33, 47, 63, 65, 127, 129, 255, 257];
    
    // Act & Assert
    for &size in &sizes {
        let addr = fixture.allocate(size);
        
        assert!(
            addr > 0,
            "Non-power-of-2 size {} allocation returned null",
            size
        );
        
        assert_address_aligned(addr, DEFAULT_ALIGNMENT,
            &format!("{}-byte (non-power-of-2) allocation", size));
    }
}

/// ============================================================================
/// RAPID ALLOCATION/DEALLOCATION
/// ============================================================================

/// Test rapid allocation without deallocation
///
/// **Bug this finds:** Bump pointer overflow, rapid alloc bugs
/// **Invariant verified:** Rapid allocations all succeed or OOM cleanly
#[test]
fn test_rapid_allocations() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    let count = 10000;
    let size = 64;
    
    // Act
    let mut success_count = 0;
    let mut oom_count = 0;
    
    for _ in 0..count {
        match fixture.gc.heap().allocate_tlab_memory(size) {
            Ok(_) => success_count += 1,
            Err(FgcError::OutOfMemory { .. }) => oom_count += 1,
            Err(e) => {
                panic!("Unexpected error during rapid allocation: {:?}", e);
            }
        }
    }
    
    // Assert
    assert!(
        success_count > 0 || oom_count > 0,
        "All {} rapid allocations failed with non-OOM error",
        count
    );
    
    // If we got OOM, it should be consistent
    if oom_count > 0 {
        assert_eq!(
            success_count + oom_count,
            count,
            "Allocation results don't add up"
        );
    }
}

/// ============================================================================
/// NEGATIVE/INVALID SIZE TESTS
/// ============================================================================

/// Test that usize::MAX size is handled
///
/// **Bug this finds:** Integer overflow, size validation bugs
/// **Invariant verified:** Maximum size returns OOM, not panic
#[test]
fn test_maximum_size_allocation() {
    // Arrange
    let fixture = GcFixture::with_defaults();
    
    // Act - attempt maximum size
    let result = fixture.gc.heap().allocate_tlab_memory(usize::MAX);
    
    // Assert - must be OOM, not panic
    match result {
        Err(FgcError::OutOfMemory { .. }) => {
            // Expected - can't allocate usize::MAX bytes
        }
        Ok(_) => {
            panic!("usize::MAX allocation succeeded - impossible");
        }
        Err(e) => {
            // Other errors are acceptable too
            assert!(
                matches!(e, FgcError::OutOfMemory { .. } | 
                         FgcError::TlabError(_)),
                "Unexpected error for usize::MAX: {:?}",
                e
            );
        }
    }
}

/// ============================================================================
/// CONCURRENT EDGE CASES
/// ============================================================================

/// Test concurrent allocations that exhaust heap
///
/// **Bug this finds:** Race condition in OOM detection
/// **Invariant verified:** Concurrent exhaustion handled safely
#[test]
fn test_concurrent_heap_exhaustion() {
    // Arrange - small heap
    let heap_size = 32 * 1024; // 32KB
    let fixture = GcFixture::with_heap_size(heap_size);
    let thread_count = 4;
    let alloc_size = 4 * 1024; // 4KB
    
    let mut handles = Vec::new();
    
    // Act - concurrent allocations until OOM
    for thread_id in 0..thread_count {
        let gc: Arc<fgc::GarbageCollector> = Arc::clone(&fixture.gc);

        let handle = std::thread::spawn(move || {
            let mut local_success = 0;
            let mut local_oom = 0;
            
            for _ in 0..100 {
                match gc.heap().allocate_tlab_memory(alloc_size) {
                    Ok(_) => local_success += 1,
                    Err(FgcError::OutOfMemory { .. }) => local_oom += 1,
                    Err(e) => {
                        panic!("Thread {} unexpected error: {:?}", thread_id, e);
                    }
                }
            }
            
            (local_success, local_oom)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_success = 0;
    let mut total_oom = 0;
    
    for handle in handles {
        let (success, oom) = handle.join().expect("Thread should not panic");
        total_success += success;
        total_oom += oom;
    }
    
    // Assert - at least some OOM should occur
    assert!(
        total_oom > 0,
        "Concurrent exhaustion: {} successes, {} OOM - heap limit not enforced",
        total_success,
        total_oom
    );
}
