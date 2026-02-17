//! FGC Performance Benchmarks
//!
//! Benchmarks untuk mengukur performa FGC.

use fgc::{GcConfig, GarbageCollector};
use test::Bencher;

#[bench]
fn bench_gc_creation(b: &mut Bencher) {
    b.iter(|| {
        let config = GcConfig::default();
        let _gc = GarbageCollector::new(config).unwrap();
    });
}

#[bench]
fn bench_allocation_small(b: &mut Bencher) {
    let config = GcConfig::default();
    let gc = GarbageCollector::new(config).unwrap();
    let heap = gc.heap();

    b.iter(|| {
        let _ = heap.allocate_tlab_memory(64);
    });
}

#[bench]
fn bench_allocation_medium(b: &mut Bencher) {
    let config = GcConfig::default();
    let gc = GarbageCollector::new(config).unwrap();
    let heap = gc.heap();

    b.iter(|| {
        let _ = heap.allocate_tlab_memory(512);
    });
}

#[bench]
fn bench_allocation_large(b: &mut Bencher) {
    let config = GcConfig::default();
    let gc = GarbageCollector::new(config).unwrap();
    let heap = gc.heap();

    b.iter(|| {
        let _ = heap.allocate_tlab_memory(4096);
    });
}

#[bench]
fn bench_gc_cycle_young(b: &mut Bencher) {
    let config = GcConfig {
        target_pause_time_ms: 1,
        ..Default::default()
    };

    let gc = GarbageCollector::new(config).unwrap();

    b.iter(|| {
        gc.request_gc(fgc::GcGeneration::Young, fgc::GcReason::Explicit);
    });
}

#[bench]
fn bench_gc_cycle_full(b: &mut Bencher) {
    let config = GcConfig {
        target_pause_time_ms: 10,
        ..Default::default()
    };

    let gc = GarbageCollector::new(config).unwrap();

    b.iter(|| {
        gc.request_gc(fgc::GcGeneration::Full, fgc::GcReason::Explicit);
    });
}
