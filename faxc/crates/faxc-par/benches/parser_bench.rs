//! Parser Benchmarks
//!
//! Benchmarks untuk mengukur performa parser.
//! Run dengan: `cargo bench --package faxc-par`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use faxc_lex::Lexer;
use faxc_par::Parser;
use faxc_util::Handler;

fn parse_source(source: &str) -> faxc_par::Ast {
    let mut handler = Handler::new();
    let tokens: Vec<_> = Lexer::new(source, &mut handler).collect();
    let mut parser = Parser::new(tokens, &mut handler);
    parser.parse()
}

fn bench_parser_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_simple");

    let source = "let x = 42;";
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("let_statement", |b| {
        b.iter(|| parse_source(black_box(source)))
    });

    group.finish();
}

fn bench_parser_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_functions");

    let source = r#"
        fn main() {
            let x = 42;
            let y = x + 1;
            return y;
        }
        
        fn fib(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
    "#;

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("functions", |b| b.iter(|| parse_source(black_box(source))));

    group.finish();
}

fn bench_parser_structs(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_structs");

    let source = r#"
        struct Point {
            x: i32,
            y: i32,
        }
        
        struct Rectangle {
            origin: Point,
            width: i32,
            height: i32,
        }
        
        impl Point {
            fn new(x: i32, y: i32) -> Point {
                return Point { x: x, y: y };
            }
            
            fn distance_to(self, other: Point) -> i32 {
                let dx = self.x - other.x;
                let dy = self.y - other.y;
                return dx * dx + dy * dy;
            }
        }
    "#;

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("structs_impls", |b| {
        b.iter(|| parse_source(black_box(source)))
    });

    group.finish();
}

fn bench_parser_enums(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_enums");

    let source = r#"
        enum Color {
            Red,
            Green,
            Blue,
            Custom(r: i32, g: i32, b: i32),
        }
        
        enum Option[T] {
            Some(T),
            None,
        }
        
        enum Result[T, E] {
            Ok(T),
            Err(E),
        }
    "#;

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("enums", |b| b.iter(|| parse_source(black_box(source))));

    group.finish();
}

fn bench_parser_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_control_flow");

    let source = r#"
        fn process(n: i32) -> i32 {
            if n < 0 {
                return -1;
            } else if n == 0 {
                return 0;
            } else {
                match n {
                    1 => return 1,
                    2 => return 2,
                    _ => {
                        let mut sum = 0;
                        let mut i = 0;
                        while i < n {
                            sum = sum + i;
                            i = i + 1;
                        }
                        return sum;
                    }
                }
            }
        }
    "#;

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("control_flow", |b| {
        b.iter(|| parse_source(black_box(source)))
    });

    group.finish();
}

fn bench_parser_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_complex");

    let source = r#"
        trait Drawable {
            fn draw(&self);
            fn bounding_box(&self) -> Rectangle;
        }
        
        struct Point {
            x: i32,
            y: i32,
        }
        
        struct Rectangle {
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        }
        
        impl Point {
            fn new(x: i32, y: i32) -> Point {
                return Point { x: x, y: y };
            }
        }
        
        impl Rectangle {
            fn new(x: i32, y: i32, w: i32, h: i32) -> Rectangle {
                return Rectangle { x: x, y: y, width: w, height: h };
            }
        }
        
        impl Drawable for Point {
            fn draw(&self) {
                println!("Point at ({}, {})", self.x, self.y);
            }
            
            fn bounding_box(&self) -> Rectangle {
                return Rectangle::new(self.x, self.y, 1, 1);
            }
        }
        
        fn main() {
            let p = Point::new(10, 20);
            p.draw();
            
            let shapes: Vec[Drawable] = [];
            shapes.push(p);
        }
    "#;

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("complex_source", |b| {
        b.iter(|| parse_source(black_box(source)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_simple,
    bench_parser_functions,
    bench_parser_structs,
    bench_parser_enums,
    bench_parser_control_flow,
    bench_parser_complex
);
criterion_main!(benches);
