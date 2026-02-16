/-
Unit tests for Semantic Analysis Microservice
Tests type checking and protobuf service interface
-/

import Compiler.Semantic.Proto
import Compiler.Lexer.Proto
import Compiler.Parser.Proto
import Compiler.Proto

namespace Tests.Unit.Semantic

open Compiler.Semantic.Proto
open Compiler.Lexer.Proto
open Compiler.Parser.Proto
open Compiler.Proto

-- Test 1: Valid module analysis
def testValidModule : IO Bool := do
  IO.println "Test 1: Valid Module Analysis"
  let source := "fn main() -> i32 { 42 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let result := analyzeProtobuf module
    if result.errors.isEmpty then
      IO.println "  ✓ Valid module passed analysis"
      return true
    else
      IO.println s!"  ✗ Unexpected errors: {result.errors}"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 2: Symbol table construction
def testSymbolTable : IO Bool := do
  IO.println "Test 2: Symbol Table Construction"
  let source := "fn add(a: i32, b: i32) -> i32 { a + b }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let result := analyzeProtobuf module
    let symCount := result.symbolTable.symbols.length
    IO.println s!"  ✓ Symbol table has {symCount} symbols"
    return true
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 3: Service response
def testServiceResponse : IO Bool := do
  IO.println "Test 3: Service Response"
  let source := "fn test() -> i32 { 0 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let response := analyzeWithResponse module
    if response.valid then
      IO.println "  ✓ Service response indicates valid module"
      return true
    else
      IO.println "  ✗ Service response indicates invalid module"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 4: Bytes analysis
def testBytesAnalysis : IO Bool := do
  IO.println "Test 4: Bytes Analysis"
  let source := "fn main() -> i32 { 0 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let moduleBytes := Proto.serializeModule module
    
    match analyzeFromBytes moduleBytes with
    | Except.ok result =>
      IO.println s!"  ✓ Analyzed from bytes, {result.errors.length} errors"
      return true
    | Except.error err =>
      IO.println s!"  ✗ Analysis error: {err}"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 5: Multiple declarations
def testMultipleDecls : IO Bool := do
  IO.println "Test 5: Multiple Declarations"
  let source := "
fn foo() -> i32 { 1 }
fn bar() -> i32 { 2 }
fn baz() -> i32 { 3 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let result := analyzeProtobuf module
    let funcCount := result.symbolTable.symbols.filter (fun s =>
      match s.kind with | .function => true | _ => false
    ) |>.length
    
    if funcCount == 3 then
      IO.println s!"  ✓ Found all 3 functions in symbol table"
      return true
    else
      IO.println s!"  ✗ Only found {funcCount} functions"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 6: Struct and enum symbols
def testStructEnumSymbols : IO Bool := do
  IO.println "Test 6: Struct and Enum Symbols"
  let source := "
struct Point { x: i32, y: i32 }
enum Color { Red, Green, Blue }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let result := analyzeProtobuf module
    let typeCount := result.symbolTable.symbols.filter (fun s =>
      match s.kind with | .struct => true | .enum => true | _ => false
    ) |>.length
    
    if typeCount == 2 then
      IO.println s!"  ✓ Found {typeCount} type definitions"
      return true
    else
      IO.println s!"  ✗ Only found {typeCount} type definitions"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 7: Type info extraction
def testTypeInfo : IO Bool := do
  IO.println "Test 7: Type Info Extraction"
  let source := "fn calc(x: i32, y: f64) -> bool { true }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let result := analyzeProtobuf module
    if result.typeInfo.types.length > 0 then
      IO.println s!"  ✓ Extracted {result.typeInfo.types.length} type annotations"
      return true
    else
      IO.println "  ✗ No type info extracted"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Test 8: Validation
def testValidation : IO Bool := do
  IO.println "Test 8: Module Validation"
  let source := "fn dup() -> i32 { 1 }
fn dup() -> i32 { 2 }"
  let tokens := lexToProtobuf source "test.fax"
  
  match parseFromProtobuf tokens with
  | Except.ok module =>
    let errors := validateModule module
    if errors.any (fun e => e.contains "Duplicate") then
      IO.println "  ✓ Detected duplicate function"
      return true
    else
      IO.println "  ✗ Failed to detect duplicate"
      return false
  | Except.error err =>
    IO.println s!"  ✗ Parse error: {err}"
    return false

-- Run all tests
def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║       Semantic Analysis Microservice Unit Tests           ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  let results ← [
    testValidModule,
    testSymbolTable,
    testServiceResponse,
    testBytesAnalysis,
    testMultipleDecls,
    testStructEnumSymbols,
    testTypeInfo,
    testValidation
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

end Tests.Unit.Semantic

def main : IO UInt32 := Tests.Unit.Semantic.runAllTests
