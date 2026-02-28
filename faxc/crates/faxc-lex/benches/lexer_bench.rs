//! Lexer Benchmarks
//!
//! Benchmarks untuk mengukur performa lexical analyzer.
//! Run dengan: `cargo bench --package faxc-lex`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use faxc_lex::Lexer;
use faxc_util::Handler;

fn create_handler() -> Handler {
    Handler::new()
}

fn lexer_token_count(source: &str) -> usize {
    let mut handler = create_handler();
    let lexer = Lexer::new(source, &mut handler);
    // Lexer implements Iterator, so we can use it directly
    lexer.count()
}

fn bench_lexer_keywords(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    let source = "let x = 42; fn main() { let y = x + 1; return y; }";
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("simple_let", |b| {
        b.iter(|| lexer_token_count(black_box("let x = 42;")))
    });

    group.bench_function("function_with_body", |b| {
        b.iter(|| lexer_token_count(black_box(source)))
    });

    group.finish();
}

fn bench_lexer_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_complex");

    // Complex source code with many tokens
    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        struct Point {
            x: i32,
            y: i32,
        }

        enum Color {
            Red,
            Green,
            Blue,
        }

        trait Drawable {
            fn draw(&self);
        }

        impl Drawable for Point {
            fn draw(&self) {
                println!("Point at ({}, {})", self.x, self.y);
            }
        }
    "#;

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("complex_source", |b| {
        b.iter(|| lexer_token_count(black_box(source)))
    });

    group.finish();
}

fn bench_lexer_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_strings");

    group.bench_function("short_string", |b| {
        b.iter(|| lexer_token_count(black_box("let s = \"hello\";")))
    });

    group.bench_function("long_string", |b| {
        let source = "let s = \"This is a longer string that contains some text for benchmarking purposes.\";";
        b.iter(|| {
            lexer_token_count(black_box(source))
        })
    });

    group.finish();
}

fn bench_lexer_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_numbers");

    group.bench_function("integer", |b| {
        b.iter(|| lexer_token_count(black_box("let x = 123456;")))
    });

    group.bench_function("float", |b| {
        b.iter(|| lexer_token_count(black_box("let x = 3.14159;")))
    });

    group.bench_function("hex", |b| {
        b.iter(|| lexer_token_count(black_box("let x = 0xDEADBEEF;")))
    });

    group.finish();
}

fn bench_lexer_identifiers(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_identifiers");

    group.bench_function("short_ident", |b| {
        b.iter(|| lexer_token_count(black_box("let x = 42;")))
    });

    group.bench_function("long_ident", |b| {
        b.iter(|| lexer_token_count(black_box("let very_long_variable_name = 42;")))
    });

    group.bench_function("many_ident", |b| {
        b.iter(|| {
            lexer_token_count(black_box(
                "let a = 1; let b = 2; let c = 3; let d = 4; let e = 5;",
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_keywords,
    bench_lexer_complex,
    bench_lexer_strings,
    bench_lexer_numbers,
    bench_lexer_identifiers
);
criterion_main!(benches);
