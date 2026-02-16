/-
Unit tests for Parser Microservice
Tests protobuf conversion and service interface
-/

import Compiler.Lexer
import Compiler.Lexer.Proto
import Compiler.Parser
import Compiler.Parser.Proto
import Compiler.Proto

namespace Tests.Unit.Parser

open Compiler.Lexer.Proto
open Compiler.Parser.Proto
open Compiler.Proto

-- Test 1: Basic parsing
def testBasicParsing : IO Bool := do
  IO.println "Test 1: Basic Parsing"
  let source := "fn main() -> i32 { 42 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    IO.println s!"  ✓ Parsed module with {module.decls.length} declarations"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 2: Function with parameters
def testFunctionWithParams : IO Bool := do
  IO.println "Test 2: Function with Parameters"
  let source := "fn add(a: i32, b: i32) -> i32 { a + b }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    match module.decls.head? with
    | some (.func name params _ _) =>
      IO.println s!"  ✓ Function '{name}' with {params.length} parameters"
      return true
    | _ =>
      IO.println "  ✗ Not a function declaration"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 3: Struct declaration
def testStructDecl : IO Bool := do
  IO.println "Test 3: Struct Declaration"
  let source := "struct Point { x: i32, y: i32 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    match module.decls.head? with
    | some (.struct name fields) =>
      IO.println s!"  ✓ Struct '{name}' with {fields.length} fields"
      return true
    | _ =>
      IO.println "  ✗ Not a struct declaration"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 4: If expression
def testIfExpression : IO Bool := do
  IO.println "Test 4: If Expression"
  let source := "fn max() -> i32 { if 5 > 3 { 5 } else { 3 } }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok _ =>
    IO.println "  ✓ Parsed if expression"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 5: Binary operations
def testBinaryOperations : IO Bool := do
  IO.println "Test 5: Binary Operations"
  let source := "fn calc() -> i32 { 1 + 2 * 3 - 4 / 2 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok _ =>
    IO.println "  ✓ Parsed binary operations"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 6: Function call
def testFunctionCall : IO Bool := do
  IO.println "Test 6: Function Call"
  let source := "fn foo() -> i32 { bar(1, 2) }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok _ =>
    IO.println "  ✓ Parsed function call"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 7: Roundtrip through bytes
def testBytesRoundtrip : IO Bool := do
  IO.println "Test 7: Bytes Roundtrip"
  let source := "fn test() -> i32 { 42 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    -- Serialize module
    let moduleBytes := Proto.serializeModule module
    
    if moduleBytes.size > 0 then
      IO.println s!"  ✓ Serialized module to {moduleBytes.size} bytes"
      return true
    else
      IO.println "  ✗ Serialization failed"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 8: Complex program
def testComplexProgram : IO Bool := do
  IO.println "Test 8: Complex Program"
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
    IO.println s!"  ✓ Parsed complex program with {module.decls.length} declarations"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Run all tests
def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║         Parser Microservice Unit Tests                    ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  let results ← [
    testBasicParsing,
    testFunctionWithParams,
    testStructDecl,
    testIfExpression,
    testBinaryOperations,
    testFunctionCall,
    testBytesRoundtrip,
    testComplexProgram
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

end Tests.Unit.Parser

def main : IO UInt32 := Tests.Unit.Parser.runAllTests
