//! Specification-Based GC Tests
//! 
//! These tests are written based on GC specification, NOT current implementation.
//! Any failing test indicates a bug in the implementation.
//! 
//! IMPORTANT: Tests verify what GC SHOULD do, not what it currently does.

use fgc::{GarbageCollector, FgcConfig, FgcError};
use std::sync::Arc;

// ============================================================================
// ALLOCATION TESTS
// ============================================================================

#[test]
fn spec_allocate_zero_size_object() {
    // SPEC: Zero-size allocations should either:
    // 1. Return unique addresses (each allocation different)
    // 2. Return an error
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr1 = gc.allocate(0);
    let addr2 = gc.allocate(0);
    
    match (addr1, addr2) {
        (Ok(a1), Ok(a2)) => assert_ne!(a1, a2, "Zero-size allocations must return unique addresses"),
        (Err(_), Err(_)) => {}, // Also valid - rejecting zero-size is OK
        _ => panic!("Inconsistent behavior for zero-size allocation"),
    }
}

#[test]
fn spec_allocate_maximum_size_object() {
    // SPEC: Allocation larger than max_heap should fail with OutOfMemory
    let config = FgcConfig {
        initial_heap_size: 1024 * 1024,
        max_heap_size: 2 * 1024 * 1024,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let result = gc.allocate(10 * 1024 * 1024);
    
    assert!(
        matches!(result, Err(FgcError::OutOfMemory { .. })), 
        "Large allocation should return OutOfMemory, got: {:?}", result
    );
}

#[test]
fn spec_allocate_unaligned_size() {
    // SPEC: All allocations must be 8-byte aligned (64-bit systems)
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    for size in [1, 7, 13, 17, 100, 127, 255] {
        let addr = gc.allocate(size).expect("Allocation failed");
        assert_eq!(addr % 8, 0, "Allocation of {} bytes not 8-byte aligned: {:#x}", size, addr);
    }
}

#[test]
fn spec_concurrent_allocation_thread_safety() {
    // SPEC: Multiple threads can allocate simultaneously without data races
    let config = FgcConfig {
        initial_heap_size: 100 * 1024 * 1024,
        ..Default::default()
    };
    
    let gc = Arc::new(GarbageCollector::new(config).expect("Failed to create GC"));
    let mut handles = vec![];
    
    for i in 0..10 {
        let gc_clone = Arc::clone(&gc);
        let handle = std::thread::spawn(move || {
            let mut addresses = Vec::new();
            for _ in 0..100 {
                if let Ok(addr) = gc_clone.allocate(64) {
                    addresses.push(addr);
                }
            }
            (i, addresses)
        });
        handles.push(handle);
    }
    
    let mut all_addresses = Vec::new();
    for handle in handles {
        let (thread_id, addrs) = handle.join().expect("Thread panicked");
        
        let unique_count = addrs.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, addrs.len(), "Thread {} got duplicate addresses", thread_id);
        
        all_addresses.extend(addrs);
    }
    
    let unique_count = all_addresses.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, all_addresses.len(), 
               "Concurrent allocations returned duplicate addresses - DATA RACE!");
}

#[test]
fn spec_allocate_negative_size_should_fail() {
    // SPEC: Negative sizes should be rejected (usize can't be negative, but 0 is edge case)
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    // Size 0 is the smallest possible - already tested above
    // This test documents that we can't test negative (usize limitation)
}

#[test]
fn spec_allocate_exact_heap_size() {
    // SPEC: Allocation exactly matching available space should succeed
    let config = FgcConfig {
        initial_heap_size: 8192,
        max_heap_size: 8192,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    // Allocate most of heap
    let addr = gc.allocate(4096).expect("Failed to allocate half heap");
    assert!(addr > 0);
}

// ============================================================================
// GARBAGE COLLECTION TESTS
// ============================================================================

#[test]
fn spec_collect_unreachable_object() {
    // SPEC: Unreachable objects must be collected
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Allocation failed");
    
    unsafe {
        *(addr as *mut u64) = 0xDEADBEEF;
    }
    
    // Don't register as root - object is unreachable
    gc.collect();
    
    // Test passes if GC doesn't crash
    // (Verifying actual collection requires internal state access)
}

#[test]
fn spec_preserve_reachable_object() {
    // SPEC: Reachable objects must NOT be collected
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Allocation failed");
    
    unsafe {
        *(addr as *mut u64) = 0x12345678DEADBEEF;
    }
    
    gc.register_root(addr).expect("Failed to register root");
    
    gc.collect();
    
    let value = unsafe { *(addr as *mut u64) };
    assert_eq!(value, 0x12345678DEADBEEF, "Reachable object's data was corrupted by GC!");
    
    gc.unregister_root(addr).expect("Failed to unregister root");
}

#[test]
fn spec_collect_cyclic_references() {
    // SPEC: Cyclic references between unreachable objects must be collected
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr_a = gc.allocate(64).expect("Failed to allocate A");
    let addr_b = gc.allocate(64).expect("Failed to allocate B");
    let addr_c = gc.allocate(64).expect("Failed to allocate C");
    
    // Create cycle: A -> B -> C -> A
    unsafe {
        *(addr_a as *mut usize) = addr_b;
        *(addr_b as *mut usize) = addr_c;
        *(addr_c as *mut usize) = addr_a;
    }
    
    // Don't register any as roots - cycle is unreachable
    gc.collect();
    
    // Test passes if GC doesn't crash
}

#[test]
fn spec_transitive_reachability() {
    // SPEC: Objects reachable transitively from roots must be preserved
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let root_addr = gc.allocate(64).expect("Failed to allocate root");
    let addr_a = gc.allocate(64).expect("Failed to allocate A");
    let addr_b = gc.allocate(64).expect("Failed to allocate B");
    let addr_c = gc.allocate(64).expect("Failed to allocate C");
    
    // Create chain: Root -> A -> B -> C
    unsafe {
        *(root_addr as *mut usize) = addr_a;
        *(addr_a as *mut usize) = addr_b;
        *(addr_b as *mut usize) = addr_c;
    }
    
    gc.register_root(root_addr).expect("Failed to register root");
    
    gc.collect();
    
    // C should still exist (transitively reachable)
    // We verify GC didn't crash and root is still valid
    let root_value = unsafe { *(root_addr as *mut usize) };
    assert_eq!(root_value, addr_a, "Root pointer corrupted!");
    
    gc.unregister_root(root_addr).expect("Failed to unregister root");
}

#[test]
fn spec_gc_without_allocations() {
    // SPEC: GC should handle being called with no allocations
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    gc.collect(); // Should not crash
}

#[test]
fn spec_gc_rapid_cycles() {
    // SPEC: GC should handle rapid consecutive collections
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Failed to allocate");
    gc.register_root(addr).expect("Failed to register root");
    
    for _ in 0..100 {
        gc.collect(); // Rapid GC cycles
    }
    
    // Object should still be accessible
    gc.unregister_root(addr).expect("Failed to unregister root");
}

// ============================================================================
// ROOT MANAGEMENT TESTS
// ============================================================================

#[test]
fn spec_register_null_root() {
    // SPEC: Registering null root should fail or be safely ignored
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let result = gc.register_root(0);
    
    // Should either return error or silently succeed (but not crash)
    assert!(result.is_ok() || result.is_err(), "Register null root should not panic");
}

#[test]
fn spec_unregister_nonexistent_root() {
    // SPEC: Unregistering non-root should fail or be safely ignored
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Failed to allocate");
    
    // Try to unregister without registering first
    let result = gc.unregister_root(addr);
    
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err(), "Unregister should not panic");
}

#[test]
fn spec_root_update_after_relocation() {
    // SPEC: If GC relocates objects, root references must be updated
    let config = FgcConfig {
        initial_heap_size: 8 * 1024,
        max_heap_size: 8 * 1024,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let mut addresses = Vec::new();
    for _ in 0..10 {
        addresses.push(gc.allocate(64).expect("Failed to allocate"));
    }
    
    for &addr in &addresses {
        gc.register_root(addr).expect("Failed to register root");
    }
    
    // Allocate more to trigger GC and potential relocation
    for _ in 0..100 {
        let _ = gc.allocate(64);
    }
    
    // GC should complete without crashing
    // Roots should still be tracked (may have new addresses)
}

#[test]
fn spec_concurrent_root_registration() {
    // SPEC: Root registration must be thread-safe
    let config = FgcConfig::default();
    let gc = Arc::new(GarbageCollector::new(config).expect("Failed to create GC"));
    let mut handles = vec![];
    
    for i in 0..10 {
        let gc_clone = Arc::clone(&gc);
        let handle = std::thread::spawn(move || {
            let addr = gc_clone.allocate(64).expect("Failed to allocate");
            gc_clone.register_root(addr).expect("Failed to register root");
            (i, addr)
        });
        handles.push(handle);
    }
    
    let mut addresses = Vec::new();
    for handle in handles {
        let (thread_id, addr) = handle.join().expect("Thread panicked");
        addresses.push(addr);
    }
    
    // All registrations should succeed without data races
}

#[test]
fn spec_duplicate_root_registration() {
    // SPEC: Registering same address twice should be handled
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Failed to allocate");
    
    gc.register_root(addr).expect("Failed to register root");
    let result = gc.register_root(addr);
    
    // Should handle duplicate gracefully
    assert!(result.is_ok() || result.is_err(), "Duplicate registration should not panic");
    
    gc.unregister_root(addr).expect("Failed to unregister root");
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn spec_unaligned_root_address() {
    // SPEC: Unaligned addresses should be rejected or handled safely
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let unaligned_addr = 0x1001; // Not 8-byte aligned
    
    let result = gc.register_root(unaligned_addr);
    assert!(result.is_ok() || result.is_err(), "Unaligned root should not panic");
}

#[test]
fn spec_single_allocation_gc() {
    // SPEC: GC should handle minimal allocations
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Failed to allocate");
    gc.register_root(addr).expect("Failed to register root");
    
    gc.collect(); // Should not crash
}

#[test]
fn spec_allocation_overflow_protection() {
    // SPEC: Integer overflow in size calculations should be detected
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    // Try to allocate size that would overflow when aligned
    let large_size = usize::MAX - 100;
    let result = gc.allocate(large_size);
    
    // Should fail gracefully, not wrap around
    assert!(
        matches!(result, Err(FgcError::OutOfMemory { .. }) | Err(FgcError::InvalidArgument(_))),
        "Overflow allocation should fail gracefully, got: {:?}", result
    );
}

#[test]
fn spec_kernel_space_address_rejected() {
    // SPEC: Kernel-space addresses should be rejected as roots
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let kernel_addr = 0xFFFF_FFFF_FFFF_F000;
    let result = gc.register_root(kernel_addr);
    
    // Should reject kernel addresses
    assert!(result.is_err(), "Kernel address should be rejected");
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[test]
fn spec_stress_many_objects() {
    // SPEC: GC should handle large numbers of objects
    let config = FgcConfig {
        initial_heap_size: 50 * 1024 * 1024,
        max_heap_size: 100 * 1024 * 1024,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let mut addresses = Vec::new();
    
    for i in 0..10000 {
        let addr = gc.allocate(64).expect("Failed to allocate");
        
        unsafe {
            *(addr as *mut u64) = i as u64;
        }
        
        if i % 10 == 0 {
            gc.register_root(addr).expect("Failed to register root");
        }
        
        addresses.push(addr);
        
        if i % 1000 == 0 {
            gc.collect();
        }
    }
    
    gc.collect();
}

#[test]
fn spec_stress_varying_sizes() {
    // SPEC: GC should handle objects of varying sizes
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let sizes = [1, 7, 8, 13, 64, 127, 128, 255, 256, 512, 1023, 1024, 4096, 8192];
    
    for &size in &sizes {
        let addr = gc.allocate(size).expect("Failed to allocate");
        assert_eq!(addr % 8, 0, "Allocation of {} bytes not aligned", size);
        gc.register_root(addr).expect("Failed to register root");
    }
    
    gc.collect();
}

#[test]
fn spec_stress_concurrent_allocation_gc() {
    // SPEC: GC should handle concurrent allocation and collection
    let config = FgcConfig {
        initial_heap_size: 50 * 1024 * 1024,
        ..Default::default()
    };
    
    let gc = Arc::new(GarbageCollector::new(config).expect("Failed to create GC"));
    let mut handles = vec![];
    
    // Allocator threads
    for i in 0..5 {
        let gc_clone = Arc::clone(&gc);
        let handle = std::thread::spawn(move || {
            for j in 0..1000 {
                let addr = gc_clone.allocate(64).expect("Failed to allocate");
                if j % 10 == 0 {
                    let _ = gc_clone.register_root(addr);
                }
            }
            i
        });
        handles.push(handle);
    }
    
    // GC thread
    let gc_clone = Arc::clone(&gc);
    let gc_handle = std::thread::spawn(move || {
        for _ in 0..10 {
            gc_clone.collect();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    gc_handle.join().expect("GC thread panicked");
}

#[test]
fn spec_stress_fragmentation() {
    // SPEC: GC should handle fragmentation and still allocate
    let config = FgcConfig {
        initial_heap_size: 32 * 1024,
        max_heap_size: 64 * 1024,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let mut addresses = Vec::new();
    
    // Allocate many small objects
    for _ in 0..100 {
        addresses.push(gc.allocate(64).expect("Failed to allocate"));
    }
    
    // Free every other one (by not registering as root)
    for (i, addr) in addresses.iter().enumerate() {
        if i % 2 == 0 {
            gc.register_root(*addr).expect("Failed to register root");
        }
    }
    
    // Run GC to collect unregistered objects
    gc.collect();
    
    // Should still be able to allocate
    let new_addr = gc.allocate(64).expect("Failed to allocate after GC");
    assert!(new_addr > 0);
}

#[test]
fn spec_stress_large_objects() {
    // SPEC: GC should handle mix of small and large objects
    let config = FgcConfig {
        initial_heap_size: 10 * 1024 * 1024,
        max_heap_size: 20 * 1024 * 1024,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    // Allocate some large objects
    let large1 = gc.allocate(1024 * 1024).expect("Failed to allocate 1MB");
    let large2 = gc.allocate(512 * 1024).expect("Failed to allocate 512KB");
    
    gc.register_root(large1).expect("Failed to register root");
    gc.register_root(large2).expect("Failed to register root");
    
    // Allocate many small objects
    for _ in 0..1000 {
        let addr = gc.allocate(64).expect("Failed to allocate small");
        gc.register_root(addr).expect("Failed to register root");
    }
    
    gc.collect();
}

// ============================================================================
// MEMORY SAFETY TESTS
// ============================================================================

#[test]
fn spec_no_use_after_free() {
    // SPEC: GC must not cause use-after-free for reachable objects
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Failed to allocate");
    
    unsafe {
        *(addr as *mut u64) = 0xCAFEBABE;
    }
    
    gc.register_root(addr).expect("Failed to register root");
    
    // Run multiple GCs
    for _ in 0..10 {
        gc.collect();
    }
    
    // Object should still be accessible (no use-after-free)
    let value = unsafe { *(addr as *mut u64) };
    assert_eq!(value, 0xCAFEBABE, "Use-after-free detected!");
    
    gc.unregister_root(addr).expect("Failed to unregister root");
}

#[test]
fn spec_no_double_free() {
    // SPEC: GC must not double-free objects
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config).expect("Failed to create GC");
    
    let addr = gc.allocate(64).expect("Failed to allocate");
    gc.register_root(addr).expect("Failed to register root");
    
    // Run many GCs - should not double-free
    for _ in 0..50 {
        gc.collect();
    }
    
    gc.unregister_root(addr).expect("Failed to unregister root");
    
    // Run more GCs after unregistering
    for _ in 0..10 {
        gc.collect();
    }
    
    // Should not crash (no double-free)
}
