/-
Unit tests for Codegen Microservice
Tests IR generation and protobuf service interface
-/

import Compiler.Codegen.Proto
import Compiler.Lexer.Proto
import Compiler.Parser.Proto
import Compiler.Proto

namespace Tests.Unit.Codegen

open Compiler.Codegen.Proto
open Compiler.Lexer.Proto
open Compiler.Parser.Proto
open Compiler.Proto

-- Test 1: Basic IR generation
def testBasicIR : IO Bool := do
  IO.println "Test 1: Basic IR Generation"
  let source := "fn main() -> i32 { 42 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let ir := generateFromProtobuf module
    if ir.contains "define i32 @main" then
      IO.println "  ✓ Generated IR with main function"
      return true
    else
      IO.println "  ✗ IR missing main function"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 2: IR with arithmetic
def testArithmeticIR : IO Bool := do
  IO.println "Test 2: IR with Arithmetic"
  let source := "fn add() -> i32 { 1 + 2 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let ir := generateFromProtobuf module
    if ir.contains "add" || ir.contains "+" then
      IO.println "  ✓ Generated IR with arithmetic"
      return true
    else
      IO.println "  ✗ IR missing arithmetic"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 3: IR with function call
def testFunctionCallIR : IO Bool := do
  IO.println "Test 3: IR with Function Call"
  let source := "fn double(x: i32) -> i32 { x * 2 }
fn main() -> i32 { double(21) }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let ir := generateFromProtobuf module
    if ir.contains "@double" && ir.contains "@main" then
      IO.println "  ✓ Generated IR with function calls"
      return true
    else
      IO.println "  ✗ IR missing function calls"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 4: IR with if expression
def testIfExpressionIR : IO Bool := do
  IO.println "Test 4: IR with If Expression"
  let source := "fn max() -> i32 { if 5 > 3 { 5 } else { 3 } }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let ir := generateFromProtobuf module
    if ir.contains "br i1" && ir.contains "phi" then
      IO.println "  ✓ Generated IR with conditional branches"
      return true
    else
      IO.println "  ✗ IR missing conditional logic"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 5: Standard library inclusion
def testStdlib : IO Bool := do
  IO.println "Test 5: Standard Library"
  let source := "fn main() -> i32 { 0 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let ir := generateFromProtobuf module
    if ir.contains "@println" then
      IO.println "  ✓ Generated IR includes standard library"
      return true
    else
      IO.println "  ✗ IR missing standard library"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 6: Service response generation
def testServiceResponse : IO Bool := do
  IO.println "Test 6: Service Response"
  let source := "fn test() -> i32 { 42 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let opts : CodegenOptions := {
      targetTriple := "x86_64-unknown-linux-gnu",
      optLevel := 2,
      emitDebug := false
    }
    let response := generateWithResponse module opts
    
    match response.llvmIR with
    | some ir =>
      IO.println s!"  ✓ Generated service response with {ir.length} chars"
      return true
    | none =>
      IO.println "  ✗ Service response missing IR"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 7: Bytes generation
def testBytesGeneration : IO Bool := do
  IO.println "Test 7: Bytes Generation"
  let source := "fn main() -> i32 { 0 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let moduleBytes := Proto.serializeModule module
    
    match generateFromBytes moduleBytes with
    | Except.ok ir =>
      IO.println s!"  ✓ Generated IR from bytes ({ir.length} chars)"
      return true
    | Except.error err =>
      IO.println s!"  ✗ Generation error: {err}"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 8: Complex program IR
def testComplexProgramIR : IO Bool := do
  IO.println "Test 8: Complex Program IR"
  let source := "
fn factorial(n: i32) -> i32 {
  if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn main() -> i32 {
  factorial(5)
}"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let ir := generateFromProtobuf module
    if ir.contains "@factorial" && ir.contains "@main" && ir.contains "call" then
      IO.println s!"  ✓ Generated IR for complex program ({ir.length} chars)"
      return true
    else
      IO.println "  ✗ IR missing expected components"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Run all tests
def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║         Codegen Microservice Unit Tests                   ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  let results ← [
    testBasicIR,
    testArithmeticIR,
    testFunctionCallIR,
    testIfExpressionIR,
    testStdlib,
    testServiceResponse,
    testBytesGeneration,
    testComplexProgramIR
  ].mapM id
  
  let passed := results.filter id |>.length
  let total := results.length
  
  IO.println ""
  IO.println s!"Results: {passed}/{total} tests passed"
  
  if passed == total then
    IO.println "✓ All tests passed!"
    return 0
  else
    IO.println s!"✗ {total - passed} test(s) failed"
    return 1

end Tests.Unit.Codegen

def main : IO UInt32 := Tests.Unit.Codegen.runAllTests
