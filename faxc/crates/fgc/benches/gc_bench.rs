//! FGC Comprehensive Benchmarks
//!
//! Comprehensive benchmarks untuk mengukur performa FGC dalam berbagai skenario.
//! Run dengan: `cargo bench --package fgc`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use fgc::{GarbageCollector, GcConfig, GcGeneration, GcReason, Runtime};
use std::sync::Arc;

fn create_gc() -> GarbageCollector {
    let config = GcConfig::default();
    GarbageCollector::new(config).unwrap()
}

fn create_runtime() -> Runtime {
    let config = GcConfig {
        max_heap_size: 512 * 1024 * 1024,
        min_heap_size: 64 * 1024 * 1024,
        soft_max_heap_size: 512 * 1024 * 1024,
        ..Default::default()
    };
    Runtime::new(config).unwrap()
}

fn bench_gc_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_creation");

    group.bench_function("default_config", |b| {
        b.iter(|| {
            let config = GcConfig::default();
            black_box(GarbageCollector::new(config).unwrap())
        })
    });

    group.bench_function("large_heap", |b| {
        b.iter(|| {
            let config = GcConfig {
                max_heap_size: 2 * 1024 * 1024 * 1024,
                min_heap_size: 256 * 1024 * 1024,
                soft_max_heap_size: 2 * 1024 * 1024 * 1024,
                ..Default::default()
            };
            black_box(GarbageCollector::new(config).unwrap())
        })
    });

    group.bench_function("generational_disabled", |b| {
        b.iter(|| {
            let config = GcConfig {
                generational: false,
                ..Default::default()
            };
            black_box(GarbageCollector::new(config).unwrap())
        })
    });

    group.finish();
}

fn bench_allocation_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_small");

    let gc = create_gc();
    let heap = gc.heap();

    let sizes = [8, 16, 32, 64, 128, 256];
    for &size in &sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| {
                let _ = black_box(heap.allocate_tlab_memory(size));
            })
        });
    }

    group.finish();
}

fn bench_allocation_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_medium");

    let gc = create_gc();
    let heap = gc.heap();

    let sizes = [512, 1024, 2048, 4096, 8192];
    for &size in &sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| {
                let _ = black_box(heap.allocate_tlab_memory(size));
            })
        });
    }

    group.finish();
}

fn bench_allocation_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_large");

    let gc = create_gc();
    let heap = gc.heap();

    let sizes = [16384, 32768, 65536, 131072, 262144];
    for &size in &sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| {
                let _ = black_box(heap.allocate_tlab_memory(size));
            })
        });
    }

    group.finish();
}

fn bench_allocation_aligned(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_aligned");

    let gc = create_gc();
    let heap = gc.heap();

    let alignments = [8, 16, 32, 64, 128, 256];
    for &align in &alignments {
        group.bench_function(format!("align_{}", align), |b| {
            b.iter(|| {
                let _ = black_box(heap.allocate_tlab_memory_aligned(64, align));
            })
        });
    }

    group.finish();
}

fn bench_gc_cycle_young(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_cycle_young");

    let gc = create_gc();

    group.bench_function("minor_gc", |b| {
        b.iter(|| {
            gc.request_gc(GcGeneration::Young, GcReason::Explicit);
        })
    });

    group.finish();
}

fn bench_gc_cycle_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_cycle_full");

    let gc = create_gc();

    group.bench_function("major_gc", |b| {
        b.iter(|| {
            gc.request_gc(GcGeneration::Full, GcReason::Explicit);
        })
    });

    group.finish();
}

fn bench_root_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("root_registration");

    let gc = create_gc();
    let heap = gc.heap();
    let addr = heap.allocate_tlab_memory(64).unwrap();

    group.bench_function("register_single", |b| {
        b.iter(|| {
            black_box(gc.register_root(addr));
        })
    });

    gc.register_root(addr).unwrap();

    group.bench_function("unregister_single", |b| {
        b.iter(|| {
            black_box(gc.unregister_root(addr));
        })
    });

    group.finish();
}

fn bench_multi_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_allocation");

    let gc = create_gc();
    let heap = gc.heap();

    group.bench_function("10_objects", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = heap.allocate_tlab_memory(64);
            }
        })
    });

    group.bench_function("100_objects", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = heap.allocate_tlab_memory(64);
            }
        })
    });

    group.bench_function("1000_objects", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _ = heap.allocate_tlab_memory(64);
            }
        })
    });

    group.finish();
}

fn bench_mixed_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_sizes");

    let gc = create_gc();
    let heap = gc.heap();

    group.bench_function("mixed_workload", |b| {
        b.iter(|| {
            let sizes = [16, 32, 64, 128, 256, 512, 1024, 2048];
            for &size in sizes.iter().cycle().take(100) {
                let _ = heap.allocate_tlab_memory(size);
            }
        })
    });

    group.finish();
}

fn bench_gc_with_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_with_allocation");

    let gc = Arc::new(create_gc());

    group.bench_function("alloc_then_young_gc", |b| {
        b.iter(|| {
            let heap = gc.heap();
            for _ in 0..1000 {
                let _ = heap.allocate_tlab_memory(64);
            }
            gc.request_gc(GcGeneration::Young, GcReason::Explicit);
        })
    });

    group.finish();
}

fn bench_concurrent_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_allocation");

    let gc = Arc::new(create_gc());

    group.bench_function("2_threads", |b| {
        b.iter(|| {
            let gc1 = Arc::clone(&gc);
            let gc2 = Arc::clone(&gc);

            std::thread::spawn(move || {
                let heap = gc1.heap();
                for _ in 0..500 {
                    let _ = heap.allocate_tlab_memory(64);
                }
            });

            let heap = gc2.heap();
            for _ in 0..500 {
                let _ = heap.allocate_tlab_memory(64);
            }
        })
    });

    group.finish();
}

fn bench_gc_config_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_validation");

    group.bench_function("default", |b| {
        b.iter(|| {
            let config = GcConfig::default();
            black_box(config.validate())
        })
    });

    group.bench_function("custom_valid", |b| {
        b.iter(|| {
            let config = GcConfig {
                max_heap_size: 1024 * 1024 * 1024,
                min_heap_size: 128 * 1024 * 1024,
                target_pause_time_ms: 5,
                generational: true,
                ..Default::default()
            };
            black_box(config.validate())
        })
    });

    group.finish();
}

fn bench_heap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("heap_operations");

    let gc = create_gc();
    let heap = gc.heap();

    group.bench_function("region_allocation", |b| {
        b.iter(|| {
            black_box(heap.allocate_region(1024 * 1024, fgc::heap::Generation::Young));
        })
    });

    group.finish();
}

fn bench_object_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_graph");

    let gc = Arc::new(create_gc());
    let heap = gc.heap();

    let mut root_addrs = Vec::new();
    for _ in 0..10 {
        let addr = heap.allocate_tlab_memory(64).unwrap();
        root_addrs.push(addr);
        gc.register_root(addr).unwrap();
    }

    for _ in 0..100 {
        let addr = heap.allocate_tlab_memory(64).unwrap();
    }

    group.bench_function("traverse_graph", |b| {
        b.iter(|| {
            gc.request_gc(GcGeneration::Full, GcReason::Explicit);
        })
    });

    for addr in root_addrs {
        gc.unregister_root(addr).unwrap();
    }

    group.finish();
}

fn bench_stress_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_allocation");

    let gc = Arc::new(create_gc());

    group.bench_function("continuous_allocation", |b| {
        b.iter(|| {
            let heap = gc.heap();
            for _ in 0..10000 {
                let size = ((std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    % 4096) as usize)
                    .max(8);
                let _ = heap.allocate_tlab_memory(size);
            }
        })
    });

    group.finish();
}

fn bench_tlab_refill(c: &mut Criterion) {
    let mut group = c.benchmark_group("tlab_refill");

    let config = GcConfig {
        min_heap_size: 128 * 1024 * 1024,
        initial_heap_size: 128 * 1024 * 1024,
        max_heap_size: 512 * 1024 * 1024,
        soft_max_heap_size: 256 * 1024 * 1024,
        tlab_size: 1024,
        tlab_min_size: 512,
        ..Default::default()
    };
    let gc = GarbageCollector::new(config).unwrap();
    let heap = gc.heap();

    group.bench_function("force_refill", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = heap.allocate_tlab_memory(64);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_gc_creation,
    bench_allocation_small,
    bench_allocation_medium,
    bench_allocation_large,
    bench_allocation_aligned,
    bench_gc_cycle_young,
    bench_gc_cycle_full,
    bench_root_registration,
    bench_multi_allocation,
    bench_mixed_sizes,
    bench_gc_with_allocation,
    bench_concurrent_allocation,
    bench_gc_config_validation,
    bench_heap_operations,
    bench_object_graph,
    bench_stress_allocation,
    bench_tlab_refill
);
criterion_main!(benches);
