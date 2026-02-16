/-
End-to-End Tests for Fax Compiler
Tests complete compilation workflows from source to LLVM IR
-/

import Compiler.Driver
import Compiler.Lexer
import Compiler.Parser
import Compiler.Codegen
import Compiler.Semantic

namespace Tests.E2E

open Compiler.Driver
open Compiler.Lexer
open Compiler.Parser
open Compiler.Codegen
open Compiler.Semantic

-- ============================================================================
-- Test Helpers
-- ============================================================================

def compileSource (source : String) : Except String String := do
  -- Step 1: Lexing
  let tokens := Lexer.lex source
  
  -- Step 2: Parsing
  let module ← Parser.parseModule tokens
  
  -- Step 3: Semantic Analysis
  let semanticResult := Semantic.typeCheckModule module
  if !semanticResult.isValid then
    let errors := semanticResult.errors.map (·.message) |> String.intercalate "\n"
    Except.error s!"Semantic errors:\n{errors}"
  else
    -- Step 4: Code Generation
    let ir := Codegen.generateIR module
    Except.ok ir

def expectSuccess (source : String) (testName : String) : IO Bool := do
  match compileSource source with
  | Except.ok ir =>
    IO.println s!"  ✓ {testName}"
    return true
  | Except.error err =>
    IO.println s!"  ✗ {testName}"
    IO.println s!"    Error: {err}"
    return false

def expectError (source : String) (expectedError : String) (testName : String) : IO Bool := do
  match compileSource source with
  | Except.ok _ =>
    IO.println s!"  ✗ {testName} (expected error but succeeded)"
    return false
  | Except.error err =>
    if err.contains expectedError then
      IO.println s!"  ✓ {testName}"
      return true
    else
      IO.println s!"  ✗ {testName}"
      IO.println s!"    Expected error containing: {expectedError}"
      IO.println s!"    Got: {err}"
      return false

-- ============================================================================
-- Basic Programs
-- ============================================================================

def testHelloWorld : IO Bool := do
  IO.println "\nTest: Hello World"
  let source := "fn main() -> i32 { 0 }"
  expectSuccess source "Hello world compiles"

def testSimpleArithmetic : IO Bool := do
  IO.println "\nTest: Simple Arithmetic"
  let source := "fn calc() -> i32 { 1 + 2 * 3 }"
  expectSuccess source "Arithmetic expressions"

def testVariableDeclaration : IO Bool := do
  IO.println "\nTest: Variable Declaration"
  let source := "
fn main() -> i32 {
  let x = 42
  let y = x + 8
  y
}"
  expectSuccess source "Variable declarations"

def testMutableVariable : IO Bool := do
  IO.println "\nTest: Mutable Variable"
  let source := "
fn main() -> i32 {
  let mut x = 10
  x = 20
  x
}"
  expectSuccess source "Mutable variables"

-- ============================================================================
-- Control Flow
-- ============================================================================

def testIfExpression : IO Bool := do
  IO.println "\nTest: If Expression"
  let source := "
fn max(a: i32, b: i32) -> i32 {
  if a > b { a } else { b }
}"
  expectSuccess source "If expressions"

def testNestedIf : IO Bool := do
  IO.println "\nTest: Nested If"
  let source := "
fn sign(x: i32) -> i32 {
  if x > 0 { 1 } else if x < 0 { -1 } else { 0 }
}"
  expectSuccess source "Nested if expressions"

def testWhileLoop : IO Bool := do
  IO.println "\nTest: While Loop"
  let source := "
fn sum(n: i32) -> i32 {
  let mut i = 0
  let mut s = 0
  while i < n {
    s = s + i
    i = i + 1
  }
  s
}"
  expectSuccess source "While loops"

def testBreakContinue : IO Bool := do
  IO.println "\nTest: Break and Continue"
  let source := "
fn test() -> i32 {
  let mut i = 0
  while true {
    if i > 10 { break }
    i = i + 1
    if i % 2 == 0 { continue }
  }
  i
}"
  expectSuccess source "Break and continue"

-- ============================================================================
-- Functions
-- ============================================================================

def testFunctionDefinition : IO Bool := do
  IO.println "\nTest: Function Definition"
  let source := "
fn add(a: i32, b: i32) -> i32 {
  a + b
}

fn main() -> i32 {
  add(3, 4)
}"
  expectSuccess source "Function definitions"

def testRecursiveFunction : IO Bool := do
  IO.println "\nTest: Recursive Function"
  let source := "
fn factorial(n: i32) -> i32 {
  if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn main() -> i32 {
  factorial(5)
}"
  expectSuccess source "Recursive functions"

def testMultipleFunctions : IO Bool := do
  IO.println "\nTest: Multiple Functions"
  let source := "
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
fn mul(a: i32, b: i32) -> i32 { a * b }
fn div(a: i32, b: i32) -> i32 { a / b }

fn main() -> i32 {
  add(mul(2, 3), sub(10, div(8, 2)))
}"
  expectSuccess source "Multiple functions"

def testHigherOrderFunction : IO Bool := do
  IO.println "\nTest: Higher-Order Function"
  let source := "
fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
  f(x)
}

fn double(x: i32) -> i32 {
  x * 2
}

fn main() -> i32 {
  apply(double, 21)
}"
  expectSuccess source "Higher-order functions"

-- ============================================================================
-- Data Types
-- ============================================================================

def testStructDefinition : IO Bool := do
  IO.println "\nTest: Struct Definition"
  let source := "
struct Point {
  x: i32,
  y: i32
}

fn main() -> i32 {
  0
}"
  expectSuccess source "Struct definitions"

def testStructConstruction : IO Bool := do
  IO.println "\nTest: Struct Construction"
  let source := "
struct Point {
  x: i32,
  y: i32
}

fn main() -> i32 {
  let p = Point { x: 10, y: 20 }
  p.x
}"
  expectSuccess source "Struct construction"

def testTuple : IO Bool := do
  IO.println "\nTest: Tuple"
  let source := "
fn swap(a: i32, b: i32) -> (i32, i32) {
  (b, a)
}

fn main() -> i32 {
  let (x, y) = swap(1, 2)
  x + y
}"
  expectSuccess source "Tuples"

def testEnumDefinition : IO Bool := do
  IO.println "\nTest: Enum Definition"
  let source := "
enum Color {
  Red,
  Green,
  Blue
}

fn main() -> i32 {
  0
}"
  expectSuccess source "Enum definitions"

def testEnumWithData : IO Bool := do
  IO.println "\nTest: Enum With Data"
  let source := "
enum Result {
  Ok(i32),
  Err(str)
}

fn main() -> i32 {
  0
}"
  expectSuccess source "Enums with data"

-- ============================================================================
-- Type Checking Errors
-- ============================================================================

def testTypeMismatch : IO Bool := do
  IO.println "\nTest: Type Mismatch Error"
  let source := "
fn main() -> i32 {
  let x: str = 42
  0
}"
  expectError source "type mismatch" "Type mismatch detection"

def testUndefinedVariable : IO Bool := do
  IO.println "\nTest: Undefined Variable Error"
  let source := "
fn main() -> i32 {
  x + 1
}"
  expectError source "Undefined variable" "Undefined variable detection"

def testUndefinedFunction : IO Bool := do
  IO.println "\nTest: Undefined Function Error"
  let source := "
fn main() -> i32 {
  unknownFunc(1, 2)
}"
  expectError source "Undefined function" "Undefined function detection"

def testWrongArity : IO Bool := do
  IO.println "\nTest: Wrong Arity Error"
  let source := "
fn add(a: i32, b: i32) -> i32 { a + b }

fn main() -> i32 {
  add(1)
}"
  expectError source "arity mismatch" "Arity mismatch detection"

def testDuplicateDefinition : IO Bool := do
  IO.println "\nTest: Duplicate Definition Error"
  let source := "
fn foo() -> i32 { 1 }
fn foo() -> i32 { 2 }

fn main() -> i32 { 0 }"
  expectError source "duplicate" "Duplicate definition detection"

-- ============================================================================
-- Complex Programs
-- ============================================================================

def testFibonacci : IO Bool := do
  IO.println "\nTest: Fibonacci Program"
  let source := "
fn fib(n: i32) -> i32 {
  if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() -> i32 {
  fib(10)
}"
  expectSuccess source "Fibonacci implementation"

def testFactorial : IO Bool := do
  IO.println "\nTest: Factorial Program"
  let source := "
fn factorial(n: i32) -> i32 {
  if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn main() -> i32 {
  factorial(5)
}"
  expectSuccess source "Factorial implementation"

def testGCD : IO Bool := do
  IO.println "\nTest: GCD Program"
  let source := "
fn gcd(a: i32, b: i32) -> i32 {
  if b == 0 { a } else { gcd(b, a % b) }
}

fn main() -> i32 {
  gcd(48, 18)
}"
  expectSuccess source "GCD implementation"

def testBinarySearch : IO Bool := do
  IO.println "\nTest: Binary Search Program"
  let source := "
fn binarySearch(arr: [i32; 10], target: i32) -> i32 {
  let mut left = 0
  let mut right = 9
  
  while left <= right {
    let mid = (left + right) / 2
    if arr[mid] == target {
      return mid
    }
    if arr[mid] < target {
      left = mid + 1
    } else {
      right = mid - 1
    }
  }
  -1
}

fn main() -> i32 {
  0
}"
  expectSuccess source "Binary search implementation"

-- ============================================================================
-- Edge Cases
-- ============================================================================

def testEmptyFunction : IO Bool := do
  IO.println "\nTest: Empty Function"
  let source := "fn main() -> i32 { 0 }"
  expectSuccess source "Empty function"

def testDeepNesting : IO Bool := do
  IO.println "\nTest: Deep Nesting"
  let source := "
fn test() -> i32 {
  if true {
    if true {
      if true {
        if true {
          42
        } else { 0 }
      } else { 0 }
    } else { 0 }
  } else { 0 }
}"
  expectSuccess source "Deep nesting"

def testLongExpression : IO Bool := do
  IO.println "\nTest: Long Expression"
  let source := "
fn test() -> i32 {
  1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10
}"
  expectSuccess source "Long expression"

def testManyFunctions : IO Bool := do
  IO.println "\nTest: Many Functions"
  let functions := List.range 20 |>.map (λ i => 
    s!"fn f{i}() -> i32 {{ {i} }}"
  ) |> String.intercalate "\n"
  
  let source := functions ++ "\n\nfn main() -> i32 { f0() }"
  expectSuccess source "Many functions"

-- ============================================================================
-- Test Runner
-- ============================================================================

def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║           Fax Compiler E2E Tests                          ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  
  let tests := [
    -- Basic Programs
    testHelloWorld,
    testSimpleArithmetic,
    testVariableDeclaration,
    testMutableVariable,
    
    -- Control Flow
    testIfExpression,
    testNestedIf,
    testWhileLoop,
    testBreakContinue,
    
    -- Functions
    testFunctionDefinition,
    testRecursiveFunction,
    testMultipleFunctions,
    testHigherOrderFunction,
    
    -- Data Types
    testStructDefinition,
    testStructConstruction,
    testTuple,
    testEnumDefinition,
    testEnumWithData,
    
    -- Type Checking Errors
    testTypeMismatch,
    testUndefinedVariable,
    testUndefinedFunction,
    testWrongArity,
    testDuplicateDefinition,
    
    -- Complex Programs
    testFibonacci,
    testFactorial,
    testGCD,
    testBinarySearch,
    
    -- Edge Cases
    testEmptyFunction,
    testDeepNesting,
    testLongExpression,
    testManyFunctions
  ]
  
  let results ← tests.mapM id
  let passed := results.filter id |>.length
  let total := results.length
  
  IO.println ""
  IO.println "═══════════════════════════════════════════════════════════"
  IO.println s!"              Results: {passed}/{total} tests passed"
  IO.println "═══════════════════════════════════════════════════════════"
  
  if passed == total then
    IO.println ""
    IO.println "✓ All E2E tests passed!"
    return 0
  else
    IO.println ""
    IO.println s!"✗ {total - passed} test(s) failed"
    return 1

end Tests.E2E

def main : IO UInt32 := Tests.E2E.runAllTests
