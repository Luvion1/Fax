//! Symbol module benchmarks
//!
//! These benchmarks measure the performance of symbol interning operations.
//! Run with: `cargo bench --bench symbol_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use faxc_util::symbol::Symbol;

/// Benchmark basic symbol interning
fn bench_intern(c: &mut Criterion) {
    let mut group = c.benchmark_group("intern");
    group.throughput(Throughput::Elements(1));

    // Benchmark interning a new string (miss)
    group.bench_function("intern_new_string", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            Symbol::intern(&format!("new_string_{}", counter))
        })
    });

    // Benchmark interning an existing string (hit)
    group.bench_function("intern_existing_string", |b| {
        let _sym = Symbol::intern("existing_string");
        b.iter(|| {
            black_box(Symbol::intern("existing_string"))
        })
    });

    // Benchmark interning known keywords
    group.bench_function("intern_known_keyword", |b| {
        b.iter(|| {
            black_box(Symbol::intern_known("fn"))
        })
    });

    group.finish();
}

/// Benchmark symbol comparison
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");
    group.throughput(Throughput::Elements(1));

    let sym1 = Symbol::intern("hello");
    let sym2 = Symbol::intern("hello");
    let sym3 = Symbol::intern("world");

    // Benchmark symbol-to-symbol comparison
    group.bench_function("symbol_eq_symbol", |b| {
        b.iter(|| {
            black_box(sym1 == sym2);
            black_box(sym1 == sym3);
        })
    });

    // Benchmark symbol-to-string comparison
    group.bench_function("symbol_eq_str", |b| {
        b.iter(|| {
            black_box(sym1.eq_str("hello"));
            black_box(sym1.eq_str("world"));
        })
    });

    // Benchmark string-to-string comparison (baseline)
    group.bench_function("str_eq_str", |b| {
        let s1 = "hello";
        let s2 = "world";
        b.iter(|| {
            black_box(s1 == "hello");
            black_box(s1 == s2);
        })
    });

    group.finish();
}

/// Benchmark string retrieval
fn bench_string_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_retrieval");
    group.throughput(Throughput::Elements(1));

    let sym = Symbol::intern("test_string");

    group.bench_function("as_str", |b| {
        b.iter(|| {
            black_box(sym.as_str())
        })
    });

    group.bench_function("to_string", |b| {
        b.iter(|| {
            black_box(sym.to_string())
        })
    });

    group.finish();
}

/// Benchmark utility methods
fn bench_utilities(c: &mut Criterion) {
    let mut group = c.benchmark_group("utilities");
    group.throughput(Throughput::Elements(1));

    let empty = Symbol::intern("");
    let short = Symbol::intern("a");
    let medium = Symbol::intern("hello_world");
    let long = Symbol::intern(&"a".repeat(1000));

    group.bench_function("is_empty", |b| {
        b.iter(|| {
            black_box(empty.is_empty());
            black_box(short.is_empty());
            black_box(medium.is_empty());
            black_box(long.is_empty());
        })
    });

    group.bench_function("len", |b| {
        b.iter(|| {
            black_box(empty.len());
            black_box(short.len());
            black_box(medium.len());
            black_box(long.len());
        })
    });

    group.bench_function("starts_with", |b| {
        b.iter(|| {
            black_box(medium.starts_with("hello"));
            black_box(medium.starts_with("world"));
        })
    });

    group.bench_function("ends_with", |b| {
        b.iter(|| {
            black_box(medium.ends_with("world"));
            black_box(medium.ends_with("hello"));
        })
    });

    group.finish();
}

/// Benchmark statistics retrieval
fn bench_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");
    group.throughput(Throughput::Elements(1));

    // Pre-populate the interner
    for i in 0..1000 {
        let _ = Symbol::intern(&format!("bench_{}", i));
    }

    group.bench_function("stats_struct", |b| {
        b.iter(|| {
            black_box(Symbol::stats_struct())
        })
    });

    group.bench_function("stats_tuple", |b| {
        b.iter(|| {
            black_box(Symbol::stats())
        })
    });

    group.finish();
}

/// Benchmark with varying string sizes
fn bench_varying_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("varying_sizes");

    let sizes = [1, 10, 100, 1000, 10000];

    for &size in &sizes {
        let string = "a".repeat(size);
        group.bench_with_input(
            BenchmarkId::new("intern", size),
            &string,
            |b, s| {
                b.iter(|| {
                    black_box(Symbol::intern(s))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent interning
fn bench_concurrent(c: &mut Criterion) {
    use std::thread;

    let mut group = c.benchmark_group("concurrent");

    let thread_counts = [1, 2, 4, 8];

    for &num_threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_intern", num_threads),
            &num_threads,
            |b, &n| {
                b.iter(|| {
                    let handles: Vec<_> = (0..n)
                        .map(|i| {
                            thread::spawn(move || {
                                for j in 0..100 {
                                    let _ = Symbol::intern(&format!("thread_{}_{}", i, j));
                                }
                            })
                        })
                        .collect();

                    for h in handles {
                        h.join().unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_intern,
    bench_comparison,
    bench_string_retrieval,
    bench_utilities,
    bench_statistics,
    bench_varying_sizes,
    bench_concurrent,
);

criterion_main!(benches);
