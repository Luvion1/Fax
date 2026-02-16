/-
Integration tests for complete Microservices Pipeline
Tests end-to-end flow through all services
-/

import Compiler.Driver
import Compiler.Proto.Grpc.Codegen
import Compiler.Runtime

namespace Tests.Integration.Pipeline

open Compiler.Driver
open Compiler.Proto.Grpc.Codegen
open Compiler.Runtime

-- Test 1: Complete compilation pipeline
def testCompletePipeline : IO Bool := do
  IO.println "Test 1: Complete Compilation Pipeline"
  let source := "fn main() -> i32 { 42 }"
  
  match ← compile source false with
  | Except.ok ir =>
    if ir.contains "define i32 @main" && ir.contains "ret i32" then
      IO.println "  ✓ Complete pipeline successful"
      return true
    else
      IO.println "  ✗ IR missing expected components"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 2: Pipeline with arithmetic
def testPipelineWithArithmetic : IO Bool := do
  IO.println "Test 2: Pipeline with Arithmetic"
  let source := "fn calc() -> i32 { 1 + 2 * 3 }"
  
  match ← compile source false with
  | Except.ok ir =>
    IO.println "  ✓ Pipeline with arithmetic successful"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 3: Pipeline with control flow
def testPipelineWithControlFlow : IO Bool := do
  IO.println "Test 3: Pipeline with Control Flow"
  let source := "fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }"
  
  match ← compile source false with
  | Except.ok ir =>
    if ir.contains "br i1" && ir.contains "phi" then
      IO.println "  ✓ Pipeline with control flow successful"
      return true
    else
      IO.println "  ✗ IR missing control flow"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 4: Pipeline with function calls
def testPipelineWithFunctionCalls : IO Bool := do
  IO.println "Test 4: Pipeline with Function Calls"
  let source := "
fn double(x: i32) -> i32 { x * 2 }
fn main() -> i32 { double(21) }"
  
  match ← compile source false with
  | Except.ok ir =>
    if ir.contains "@double" && ir.contains "call" then
      IO.println "  ✓ Pipeline with function calls successful"
      return true
    else
      IO.println "  ✗ IR missing function calls"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 5: Complex program pipeline
def testComplexProgram : IO Bool := do
  IO.println "Test 5: Complex Program Pipeline"
  let source := "
fn factorial(n: i32) -> i32 {
  if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn main() -> i32 {
  factorial(5)
}"
  
  match ← compile source false with
  | Except.ok ir =>
    IO.println s!"  ✓ Complex program compiled ({ir.length} chars)"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 6: Multiple functions
def testMultipleFunctions : IO Bool := do
  IO.println "Test 6: Multiple Functions"
  let source := "
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
fn mul(a: i32, b: i32) -> i32 { a * b }
fn main() -> i32 { add(1, 2) }"
  
  match ← compile source false with
  | Except.ok ir =>
    let funcCount := ["@add", "@sub", "@mul", "@main"].count (fun f => ir.contains f)
    if funcCount == 4 then
      IO.println "  ✓ All functions compiled"
      return true
    else
      IO.println s!"  ✗ Only {funcCount}/4 functions found"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 7: Standard library integration
def testStdlibIntegration : IO Bool := do
  IO.println "Test 7: Standard Library Integration"
  let source := "fn main() -> i32 { 0 }"
  
  match ← compile source false with
  | Except.ok ir =>
    let hasPrintln := ir.contains "@println"
    let hasStdlibHeader := ir.contains "Standard Library Runtime"
    
    if hasPrintln && hasStdlibHeader then
      IO.println "  ✓ Standard library integrated"
      return true
    else
      IO.println "  ✗ Standard library missing"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Compilation failed: {err}"
    return false

-- Test 8: Batch compilation
def testBatchCompilation : IO Bool := do
  IO.println "Test 8: Batch Compilation"
  let sources := [
    "fn main() -> i32 { 1 }",
    "fn main() -> i32 { 2 }",
    "fn main() -> i32 { 3 }"
  ]
  
  let results ← sources.mapM (fun source => compile source false)
  
  let successCount := results.filter (fun r =>
    match r with | Except.ok _ => true | _ => false
  ) |>.length
  
  if successCount == sources.length then
    IO.println s!"  ✓ All {successCount} programs compiled successfully"
    return true
  else
    IO.println s!"  ✗ Only {successCount}/{sources.length} programs compiled"
    return false

-- Run all tests
def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║     Microservices Pipeline Integration Tests              ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  let results ← [
    testCompletePipeline,
    testPipelineWithArithmetic,
    testPipelineWithControlFlow,
    testPipelineWithFunctionCalls,
    testComplexProgram,
    testMultipleFunctions,
    testStdlibIntegration,
    testBatchCompilation
  ].mapM id
  
  let passed := results.filter id |>.length
  let total := results.length
  
  IO.println ""
  IO.println s!"Results: {passed}/{total} tests passed"
  
  if passed == total then
    IO.println "✓ All integration tests passed!"
    return 0
  else
    IO.println s!"✗ {total - passed} test(s) failed"
    return 1

end Tests.Integration.Pipeline

def main : IO UInt32 := Tests.Integration.Pipeline.runAllTests
