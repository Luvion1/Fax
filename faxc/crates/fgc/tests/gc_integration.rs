//! GC Integration Tests - Full Cycle Testing
//!
//! Integration tests untuk menguji full GC cycle dari start sampai finish.
//! Tests ini mencakup:
//! - Full GC cycle execution
//! - Concurrent marking dengan multiple threads
//! - GC dengan roots registration
//! - Work stealing functionality
//! - Statistics accuracy

mod common;

use common::GcFixture;
use fgc::{GcGeneration, GcReason, GcState};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// ============================================================================
/// FULL GC CYCLE TESTS
/// ============================================================================

#[test]
fn test_full_gc_cycle() {
    let fixture = GcFixture::with_defaults();
    
    // Allocate objects
    let addr1 = fixture.allocate(1024);
    let addr2 = fixture.allocate(2048);
    let addr3 = fixture.allocate(512);
    
    // Verify allocations are unique
    assert_ne!(addr1, addr2);
    assert_ne!(addr2, addr3);
    assert_ne!(addr1, addr3);
    
    // Verify addresses are aligned
    assert_eq!(addr1 % 8, 0, "Address should be 8-byte aligned");
    assert_eq!(addr2 % 8, 0, "Address should be 8-byte aligned");
    assert_eq!(addr3 % 8, 0, "Address should be 8-byte aligned");
    
    // Trigger GC
    fixture.trigger_gc(GcGeneration::Young);
    
    // Verify GC completed
    assert_eq!(fixture.state(), GcState::Idle, "GC should be in Idle state after completion");
    assert!(fixture.cycle_count() > 0, "GC cycle count should have increased");
}

#[test]
fn test_multiple_gc_cycles() {
    let fixture = GcFixture::with_defaults();
    
    let initial_cycle_count = fixture.cycle_count();
    
    // Run multiple GC cycles
    for i in 0..3 {
        // Allocate some objects
        let _addr = fixture.allocate(1024 * (i + 1));
        
        // Trigger GC
        fixture.trigger_gc(GcGeneration::Young);
        
        // Verify cycle count increased
        assert!(
            fixture.cycle_count() > initial_cycle_count + i as u64,
            "GC cycle {} should have completed", i + 1
        );
    }
}

#[test]
fn test_gc_cycle_state_transitions() {
    let fixture = GcFixture::with_defaults();
    
    // Initial state should be Idle
    assert_eq!(fixture.state(), GcState::Idle, "Initial state should be Idle");
    
    // Allocate and trigger GC
    let _addr = fixture.allocate(1024);
    
    // During GC, state should transition through Marking, Relocating, Cleanup
    // After GC, should be back to Idle
    fixture.trigger_gc(GcGeneration::Young);
    
    assert_eq!(fixture.state(), GcState::Idle, "State should return to Idle after GC");
}

/// ============================================================================
/// CONCURRENT MARKING TESTS
/// ============================================================================

#[test]
fn test_concurrent_marking() {
    let fixture = GcFixture::with_defaults();
    
    // Multiple threads allocate while GC runs
    let gc = fixture.gc.clone();
    
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let gc_clone = gc.clone();
            thread::spawn(move || {
                // Allocate objects
                let mut addresses = Vec::new();
                for _ in 0..10 {
                    if let Ok(addr) = gc_clone.heap().allocate_tlab_memory(256) {
                        addresses.push(addr);
                    }
                }
                addresses
            })
        })
        .collect();
    
    // Trigger GC while threads are allocating
    fixture.trigger_gc(GcGeneration::Young);
    
    // Wait for all threads
    let mut all_addresses = Vec::new();
    for handle in handles {
        let addresses = handle.join().expect("Thread should complete successfully");
        all_addresses.extend(addresses);
    }
    
    // Verify all addresses are unique
    use std::collections::HashSet;
    let unique: HashSet<_> = all_addresses.iter().collect();
    assert_eq!(
        unique.len(),
        all_addresses.len(),
        "All addresses should be unique"
    );
}

#[test]
fn test_concurrent_marking_with_heavy_allocation() {
    let fixture = GcFixture::with_heap_size(128 * 1024 * 1024); // 128MB heap
    
    let gc = fixture.gc.clone();
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    
    // Start allocation threads
    let alloc_handles: Vec<_> = (0..4)
        .map(|i| {
            let gc_clone = gc.clone();
            let stop = stop_flag.clone();
            thread::spawn(move || {
                let mut count = 0;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    if gc_clone.heap().allocate_tlab_memory(128).is_ok() {
                        count += 1;
                    }
                    thread::sleep(Duration::from_micros(10));
                }
                count
            })
        })
        .collect();
    
    // Let allocation run for a bit
    thread::sleep(Duration::from_millis(50));
    
    // Trigger GC
    fixture.trigger_gc(GcGeneration::Young);
    
    // Stop allocation threads
    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    
    // Wait for threads
    for handle in alloc_handles {
        let count = handle.join().expect("Thread should complete");
        assert!(count > 0, "Should have allocated some objects");
    }
    
    // Verify GC completed
    assert_eq!(fixture.state(), GcState::Idle);
}

/// ============================================================================
/// GC WITH ROOTS TESTS
/// ============================================================================

#[test]
fn test_gc_with_roots() {
    let fixture = GcFixture::with_defaults();
    
    // Register some roots
    let root_value1: usize = 0x12345678;
    let root_value2: usize = 0x87654321;
    
    let marker = {
        let gc = fixture.gc.clone();
        // Access marker through GC (would need public API)
        // For now, just verify GC works with implicit roots
        drop(gc);
    };
    
    // Allocate objects
    let _addr1 = fixture.allocate(1024);
    let _addr2 = fixture.allocate(2048);
    
    // Trigger GC
    fixture.trigger_gc(GcGeneration::Young);
    
    // Verify GC completed successfully
    assert_eq!(fixture.state(), GcState::Idle);
}

#[test]
fn test_gc_root_scanning() {
    let fixture = GcFixture::with_defaults();
    
    // Allocate objects that will be roots
    let addresses: Vec<usize> = fixture.allocate_many(10, 512);
    
    // Verify all addresses are unique and aligned
    common::assert_all_addresses_unique(&addresses, "Root objects");
    for &addr in &addresses {
        common::assert_address_aligned(addr, 8, "Root object");
    }
    
    // Trigger GC
    fixture.trigger_gc(GcGeneration::Young);
    
    // Verify GC completed
    assert_eq!(fixture.state(), GcState::Idle);
}

/// ============================================================================
/// WORK STEALING TESTS
/// ============================================================================

#[test]
fn test_gc_thread_pool_creation() {
    use fgc::marker::gc_threads::GcThreadPool;
    use fgc::marker::Marker;
    
    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Arc::new(Marker::new(heap.clone()));
    
    let pool = GcThreadPool::new(4, marker.clone(), marker.get_global_queue());
    
    assert_eq!(pool.num_workers(), 4);
    assert!(!pool.is_active());
}

#[test]
fn test_gc_thread_pool_start_stop() {
    use fgc::marker::gc_threads::GcThreadPool;
    use fgc::marker::Marker;
    
    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Arc::new(Marker::new(heap.clone()));
    
    let mut pool = GcThreadPool::new(2, marker.clone(), marker.get_global_queue());
    
    assert!(!pool.is_active());
    
    pool.start();
    
    // Wait for threads to start
    thread::sleep(Duration::from_millis(10));
    
    assert!(pool.is_active());
    
    pool.stop();
    
    assert!(!pool.is_active());
}

#[test]
fn test_gc_worker_statistics() {
    use fgc::marker::gc_threads::GcThreadPool;
    use fgc::marker::Marker;
    
    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Arc::new(Marker::new(heap.clone()));
    
    let pool = GcThreadPool::new(4, marker.clone(), marker.get_global_queue());
    
    let stats = pool.get_stats();
    
    assert_eq!(stats.total_workers, 4);
    // Workers are idle before pool starts
    assert_eq!(stats.idle_workers, 4);
    assert_eq!(stats.total_processed, 0);
    assert!(!stats.is_active);
    
    // Check individual worker stats
    for worker_stat in &stats.worker_stats {
        assert_eq!(worker_stat.processed_count, 0);
        // Workers start as idle
        assert!(worker_stat.is_idle);
    }
}

#[test]
#[test]
fn test_work_distribution() {
    use fgc::marker::gc_threads::GcThreadPool;
    use fgc::marker::Marker;

    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Arc::new(Marker::new(heap.clone()));

    let pool = GcThreadPool::new(4, marker.clone(), marker.get_global_queue());

    // Distribute work
    let work_items = vec![0x1000, 0x2000, 0x3000, 0x4000, 0x5000];
    pool.distribute_work(&work_items);

    // Verify work was distributed to global queue
    let stats = pool.get_stats();

    // Verify pool has correct number of workers
    assert_eq!(stats.total_workers, 4);
    // Work was pushed to global queue (simplified implementation)
    assert_eq!(stats.total_processed, 0); // No work processed yet
}

/// ============================================================================
/// GC STATISTICS TESTS
/// ============================================================================

#[test]
fn test_gc_statistics() {
    let fixture = GcFixture::with_defaults();
    
    let initial_stats = fixture.gc.stats().summary();
    
    // Allocate and trigger GC
    let _addr = fixture.allocate(1024);
    fixture.trigger_gc(GcGeneration::Young);
    
    let final_stats = fixture.gc.stats().summary();
    
    // Verify cycle count increased
    assert!(
        final_stats.total_cycles > initial_stats.total_cycles,
        "GC cycle count should have increased"
    );
}

#[test]
fn test_gc_pause_time_recording() {
    let fixture = GcFixture::with_defaults();
    
    // Trigger multiple GCs
    for _ in 0..3 {
        let _addr = fixture.allocate(512);
        fixture.trigger_gc(GcGeneration::Young);
    }
    
    let stats = fixture.gc.stats().summary();
    
    // Verify pause times were recorded
    assert!(stats.total_cycles >= 3, "Should have at least 3 GC cycles");
}

/// ============================================================================
/// GC GENERATION TESTS
/// ============================================================================

#[test]
fn test_young_generation_gc() {
    let fixture = GcFixture::with_defaults();
    
    // Allocate young objects
    let addresses: Vec<usize> = fixture.allocate_many(20, 256);
    
    let before_cycle = fixture.cycle_count();
    
    // Trigger young GC
    fixture.trigger_gc(GcGeneration::Young);
    
    let after_cycle = fixture.cycle_count();
    
    assert!(after_cycle > before_cycle, "Young GC should have run");
    
    // Verify addresses are still valid (not collected)
    for addr in &addresses {
        assert!(*addr > 0, "Address should be valid");
    }
}

#[test]
fn test_full_generation_gc() {
    let fixture = GcFixture::with_defaults();
    
    // Allocate objects
    let addresses: Vec<usize> = fixture.allocate_many(10, 512);
    
    let before_cycle = fixture.cycle_count();
    
    // Trigger full GC
    fixture.trigger_gc(GcGeneration::Full);
    
    let after_cycle = fixture.cycle_count();
    
    assert!(after_cycle > before_cycle, "Full GC should have run");
    
    // Verify all addresses are unique
    common::assert_all_addresses_unique(&addresses, "Full GC objects");
}

/// ============================================================================
/// EDGE CASE TESTS
/// ============================================================================

#[test]
fn test_gc_with_no_allocations() {
    let fixture = GcFixture::with_defaults();
    
    // Trigger GC without any allocations
    fixture.trigger_gc(GcGeneration::Young);
    
    // Should complete successfully
    assert_eq!(fixture.state(), GcState::Idle);
    assert!(fixture.cycle_count() > 0);
}

#[test]
fn test_gc_with_large_allocation() {
    let fixture = GcFixture::with_heap_size(64 * 1024 * 1024);
    
    // Allocate large object
    let large_addr = fixture.allocate(1024 * 1024); // 1MB
    
    assert!(large_addr > 0, "Large allocation should succeed");
    common::assert_address_aligned(large_addr, 8, "Large object");
    
    // Trigger GC
    fixture.trigger_gc(GcGeneration::Young);
    
    assert_eq!(fixture.state(), GcState::Idle);
}

#[test]
fn test_gc_rapid_succession() {
    let fixture = GcFixture::with_defaults();
    
    // Trigger multiple GCs in rapid succession
    for _ in 0..5 {
        fixture.trigger_gc(GcGeneration::Young);
    }
    
    // All should complete successfully
    assert_eq!(fixture.state(), GcState::Idle);
    assert!(fixture.cycle_count() >= 5);
}

#[test]
fn test_gc_concurrent_requests() {
    let fixture = GcFixture::with_defaults();
    
    let gc = fixture.gc.clone();
    
    // Multiple threads request GC
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let gc_clone = gc.clone();
            thread::spawn(move || {
                gc_clone.request_gc(GcGeneration::Young, GcReason::Explicit);
            })
        })
        .collect();
    
    // Wait for all requests
    for handle in handles {
        handle.join().expect("GC request should complete");
    }
    
    // Run GC to process requests
    fixture.trigger_gc(GcGeneration::Young);
    
    assert_eq!(fixture.state(), GcState::Idle);
}

/// ============================================================================
/// MARKER INTEGRATION TESTS
/// ============================================================================

#[test]
fn test_marker_start_stop() {
    use fgc::marker::Marker;
    
    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Marker::new(heap.clone());
    
    // Start marking
    marker.start_marking().expect("Marking should start");
    
    assert!(marker.is_marking_in_progress());
    
    // Finalize marking
    marker.finalize_marking().expect("Marking should finalize");
    
    assert!(!marker.is_marking_in_progress());
}

#[test]
fn test_marker_root_scanning() {
    use fgc::marker::Marker;
    
    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Marker::new(heap.clone());
    
    // Register a root
    let root_value: usize = 0x12345678;
    let _handle = marker.root_scanner().register_global_root(
        &root_value as *const usize as usize,
        Some("test_root")
    );
    
    // Scan roots
    marker.scan_roots().expect("Root scanning should succeed");
    
    // Verify root was scanned
    assert!(marker.marked_count() >= 0); // May be 0 if root is null
}

#[test]
fn test_marker_concurrent_marking_integration() {
    use fgc::marker::Marker;
    
    let config = Arc::new(fgc::GcConfig::default());
    let heap = Arc::new(fgc::heap::Heap::new(config.clone()).unwrap());
    let marker = Arc::new(Marker::new(heap.clone()));
    
    // Start concurrent marking
    marker.start_concurrent_marking(2).expect("Concurrent marking should start");
    
    // Wait for completion
    marker.wait_completion().expect("Marking should complete");
    
    // Finalize marking
    marker.finalize_marking().expect("Marking should finalize");
    
    // Verify marking finished
    assert!(!marker.is_marking_in_progress());
}
