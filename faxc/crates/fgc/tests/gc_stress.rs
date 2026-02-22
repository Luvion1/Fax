//! GC Stress Tests - High Load and Long Running Tests
//!
//! These tests verify GC behavior under stress:
//! - High allocation rates
//! - Many objects
//! - Long-running scenarios
//!
//! Note: These tests are marked as `#[ignore]` by default and should be
//! run explicitly with `cargo test --test gc_stress -- --ignored`

mod common;

use common::GcFixture;
use fgc::GcGeneration;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// ============================================================================
/// HIGH ALLOCATION RATE TESTS
/// ============================================================================

/// Stress test with high allocation rate
///
/// **Purpose:** Verify GC handles rapid allocations without crashes
/// **Duration:** ~5 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_high_allocation_rate() {
    let fixture = GcFixture::with_heap_size(128 * 1024 * 1024); // 128MB heap
    let allocations_per_second = 100_000;
    let duration_secs = 5;

    let start = std::time::Instant::now();
    let mut total_allocations = 0;
    let mut oom_count = 0;

    while start.elapsed() < Duration::from_secs(duration_secs) {
        match fixture.gc.heap().allocate_tlab_memory(64) {
            Ok(_) => total_allocations += 1,
            Err(_) => oom_count += 1,
        }

        // Trigger GC periodically
        if total_allocations % 10_000 == 0 {
            fixture.trigger_gc(GcGeneration::Young);
        }
    }

    println!(
        "Stress test completed: {} allocations, {} OOMs in {:?} (rate: {:.0} allocs/sec)",
        total_allocations,
        oom_count,
        start.elapsed(),
        total_allocations as f64 / start.elapsed().as_secs_f64()
    );

    assert!(total_allocations > 0, "Should have at least some successful allocations");
}

/// Stress test with many concurrent allocating threads
///
/// **Purpose:** Verify thread safety under high contention
/// **Duration:** ~3 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_concurrent_allocations() {
    let fixture = GcFixture::with_heap_size(256 * 1024 * 1024); // 256MB heap
    let thread_count = 16;
    let duration_secs = 3;

    let gc = Arc::clone(&fixture.gc);
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut handles = Vec::new();

    let start = std::time::Instant::now();

    for thread_id in 0..thread_count {
        let gc = Arc::clone(&gc);
        let stop = Arc::clone(&stop_flag);

        let handle = thread::spawn(move || {
            let mut local_allocs = 0;
            let mut local_ooms = 0;

            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                match gc.heap().allocate_tlab_memory(128) {
                    Ok(_) => local_allocs += 1,
                    Err(_) => local_ooms += 1,
                }

                // Small delay to prevent complete starvation
                if local_allocs % 1000 == 0 {
                    thread::yield_now();
                }
            }

            (thread_id, local_allocs, local_ooms)
        });

        handles.push(handle);
    }

    // Let it run
    thread::sleep(Duration::from_secs(duration_secs));
    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);

    // Collect results
    let mut total_allocs = 0;
    let mut total_ooms = 0;

    for handle in handles {
        let (tid, allocs, ooms) = handle.join().expect("Thread should not panic");
        println!("Thread {}: {} allocations, {} OOMs", tid, allocs, ooms);
        total_allocs += allocs;
        total_ooms += ooms;
    }

    let elapsed = start.elapsed();
    println!(
        "Concurrent stress test: {} total allocations, {} OOMs in {:?} (rate: {:.0} allocs/sec)",
        total_allocs,
        total_ooms,
        elapsed,
        total_allocs as f64 / elapsed.as_secs_f64()
    );

    assert!(total_allocs > 0, "Should have at least some successful allocations");
}

/// ============================================================================
/// MANY OBJECTS TESTS
/// ============================================================================

/// Stress test allocating many small objects
///
/// **Purpose:** Verify GC handles large numbers of objects
/// **Duration:** ~2 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_many_small_objects() {
    let fixture = GcFixture::with_heap_size(64 * 1024 * 1024); // 64MB heap
    let target_objects = 1_000_000;
    let object_size = 64;

    let mut addresses = Vec::with_capacity(target_objects);
    let mut oom_count = 0;

    let start = std::time::Instant::now();

    for i in 0..target_objects {
        match fixture.gc.heap().allocate_tlab_memory(object_size) {
            Ok(addr) => {
                if i % 100 == 0 {
                    addresses.push(addr);
                }
            }
            Err(_) => {
                oom_count += 1;
                // Trigger GC and continue
                fixture.trigger_gc(GcGeneration::Young);
            }
        }

        // Periodic GC
        if i % 10_000 == 0 {
            fixture.trigger_gc(GcGeneration::Young);
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Many objects test: {} objects tracked, {} OOMs in {:?}",
        addresses.len(),
        oom_count,
        elapsed
    );

    // Verify all tracked addresses are unique
    use std::collections::HashSet;
    let unique: HashSet<_> = addresses.iter().collect();
    assert_eq!(unique.len(), addresses.len(), "Should have unique addresses");
}

/// Stress test with varying object sizes
///
/// **Purpose:** Verify GC handles mixed allocation patterns
/// **Duration:** ~3 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_varying_object_sizes() {
    let fixture = GcFixture::with_heap_size(128 * 1024 * 1024);

    let sizes = [16, 64, 256, 1024, 4096, 16384, 65536];
    let mut allocation_counts = vec![0; sizes.len()];
    let mut oom_counts = vec![0; sizes.len()];

    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        for (i, &size) in sizes.iter().enumerate() {
            match fixture.gc.heap().allocate_tlab_memory(size) {
                Ok(_) => allocation_counts[i] += 1,
                Err(_) => oom_counts[i] += 1,
            }
        }

        // Periodic GC
        if allocation_counts.iter().sum::<usize>() % 1000 == 0 {
            fixture.trigger_gc(GcGeneration::Young);
        }
    }

    let elapsed = start.elapsed();
    println!("Varying sizes test completed in {:?}:", elapsed);
    for (i, &size) in sizes.iter().enumerate() {
        println!("  {} bytes: {} allocations, {} OOMs", size, allocation_counts[i], oom_counts[i]);
    }

    // Verify all sizes had at least some successful allocations
    for (i, &count) in allocation_counts.iter().enumerate() {
        assert!(count > 0, "Size {} should have at least some allocations", sizes[i]);
    }
}

/// ============================================================================
/// LONG-RUNNING TESTS
/// ============================================================================

/// Long-running GC test with continuous allocation and collection
///
/// **Purpose:** Verify GC stability over extended period
/// **Duration:** ~30 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_long_running() {
    let fixture = GcFixture::with_heap_size(256 * 1024 * 1024);
    let duration_secs = 30;

    let gc = Arc::clone(&fixture.gc);
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Allocator thread
    let alloc_stop = Arc::clone(&stop_flag);
    let alloc_gc = Arc::clone(&gc);
    let alloc_handle = thread::spawn(move || {
        let mut allocs = 0;
        while !alloc_stop.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = alloc_gc.heap().allocate_tlab_memory(256);
            allocs += 1;
        }
        allocs
    });

    // GC thread
    let gc_stop = Arc::clone(&stop_flag);
    let gc_clone = Arc::clone(&gc);
    let gc_handle = thread::spawn(move || {
        let mut collections = 0;
        while !gc_stop.load(std::sync::atomic::Ordering::Relaxed) {
            gc_clone.request_gc(GcGeneration::Young, fgc::GcReason::Explicit);
            collections += 1;
            thread::sleep(Duration::from_millis(100));
        }
        collections
    });

    // Let it run
    thread::sleep(Duration::from_secs(duration_secs));
    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);

    let total_allocs = alloc_handle.join().expect("Alloc thread should not panic");
    let total_collections = gc_handle.join().expect("GC thread should not panic");

    println!(
        "Long-running test: {} allocations, {} GCs in {} seconds",
        total_allocs,
        total_collections,
        duration_secs
    );

    assert!(total_allocs > 0, "Should have allocations");
    assert!(total_collections > 0, "Should have GCs");
}

/// ============================================================================
/// MEMORY PRESSURE TESTS
/// ============================================================================

/// Stress test with memory pressure (near-heap-limit allocations)
///
/// **Purpose:** Verify GC handles memory pressure correctly
/// **Duration:** ~5 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_memory_pressure() {
    let fixture = GcFixture::with_heap_size(16 * 1024 * 1024); // Small 16MB heap
    let duration_secs = 5;

    let mut successful_allocs = 0;
    let mut ooms = 0;
    let mut gc_count = 0;

    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(duration_secs) {
        match fixture.gc.heap().allocate_tlab_memory(1024) {
            Ok(_) => successful_allocs += 1,
            Err(_) => {
                ooms += 1;
                // Trigger GC on OOM
                fixture.trigger_gc(GcGeneration::Full);
                gc_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Memory pressure test: {} allocs, {} OOMs, {} GCs in {:?}",
        successful_allocs,
        ooms,
        gc_count,
        elapsed
    );

    assert!(successful_allocs > 0, "Should have some successful allocations");
    assert!(gc_count > 0, "Should have triggered some GCs");
}

/// ============================================================================
/// MIXED WORKLOAD TESTS
/// ============================================================================

/// Stress test with mixed allocation and GC patterns
///
/// **Purpose:** Verify GC handles realistic mixed workloads
/// **Duration:** ~10 seconds
#[test]
#[ignore = "Stress test - run explicitly"]
fn test_stress_mixed_workload() {
    let fixture = GcFixture::with_heap_size(128 * 1024 * 1024);
    let duration_secs = 10;

    let gc = Arc::clone(&fixture.gc);
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut handles = Vec::new();

    let start = std::time::Instant::now();

    // Thread 1: Small object allocator
    {
        let gc = Arc::clone(&gc);
        let stop = Arc::clone(&stop_flag);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = gc.heap().allocate_tlab_memory(64);
                count += 1;
            }
            ("small", count)
        }));
    }

    // Thread 2: Medium object allocator
    {
        let gc = Arc::clone(&gc);
        let stop = Arc::clone(&stop_flag);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = gc.heap().allocate_tlab_memory(1024);
                count += 1;
            }
            ("medium", count)
        }));
    }

    // Thread 3: Large object allocator
    {
        let gc = Arc::clone(&gc);
        let stop = Arc::clone(&stop_flag);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = gc.heap().allocate_tlab_memory(65536);
                count += 1;
            }
            ("large", count)
        }));
    }

    // Thread 4: GC trigger
    {
        let gc = Arc::clone(&gc);
        let stop = Arc::clone(&stop_flag);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                gc.request_gc(GcGeneration::Young, fgc::GcReason::Explicit);
                count += 1;
                thread::sleep(Duration::from_millis(50));
            }
            ("gc", count)
        }));
    }

    // Let it run
    thread::sleep(Duration::from_secs(duration_secs));
    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);

    // Collect results
    println!("Mixed workload test completed in {:?}:", start.elapsed());
    for handle in handles {
        let (name, count) = handle.join().expect("Thread should not panic");
        println!("  {}: {} operations", name, count);
    }
}
