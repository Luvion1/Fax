-- Example Fax programs for testing
namespace Examples

-- Hello World equivalent
example
fn main() -> i32 {
    0
}

-- Simple arithmetic
example
fn add(a: i32, b: i32) -> i32 {
    a + b
}

-- Factorial function
example
fn factorial(n: i32) -> i32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

-- Fibonacci
example
fn fib(n: i32) -> i32 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

-- Using let bindings
example
fn compute() -> i32 {
    let x = 10;
    let y = 20;
    let z = x + y;
    z * 2
}

-- Comparison operators
example
fn compare(a: i32, b: i32) -> i32 {
    if a > b { 1 } else { 0 }
}

-- Boolean logic
example
fn logic(x: bool, y: bool) -> bool {
    x && y || !x
}

-- Simple struct (requires codegen support)
example
struct Point {
    x: i32,
    y: i32
}

-- Using tuples
example
fn swap(a: i32, b: i32) -> (i32, i32) {
    (b, a)
}

-- Higher order function pattern (conceptual)
example
fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

end Examples
