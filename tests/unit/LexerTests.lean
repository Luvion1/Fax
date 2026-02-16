/-
Unit tests for Lexer Microservice
Tests protobuf conversion and service interface
-/

import Compiler.Lexer
import Compiler.Lexer.Proto
import Compiler.Proto

namespace Tests.Unit.Lexer

open Compiler.Lexer
open Compiler.Lexer.Proto
open Compiler.Proto

-- Test 1: Basic tokenization
def testBasicTokenization : IO Bool := do
  IO.println "Test 1: Basic Tokenization"
  let source := "fn main() { 42 }"
  let tokens := Lexer.lex source
  
  if tokens.length > 0 then
    IO.println s!"  ✓ Tokenized {tokens.length} tokens"
    return true
  else
    IO.println "  ✗ No tokens generated"
    return false

-- Test 2: Protobuf conversion
def testProtobufConversion : IO Bool := do
  IO.println "Test 2: Protobuf Conversion"
  let source := "let x = 42"
  let tokenStream := lexToProtobuf source "test.fax"
  
  if tokenStream.tokens.length > 0 then
    IO.println s!"  ✓ Converted to protobuf: {tokenStream.tokens.length} tokens"
    return true
  else
    IO.println "  ✗ Protobuf conversion failed"
    return false

-- Test 3: Roundtrip conversion
def testRoundtrip : IO Bool := do
  IO.println "Test 3: Roundtrip Conversion"
  let source := "fn add(a: i32, b: i32) -> i32 { a + b }"
  
  -- Convert to protobuf
  let tokenStream := lexToProtobuf source "test.fax"
  
  -- Convert back
  let tokens := parseFromProtobuf tokenStream
  
  if tokens.length > 0 then
    IO.println s!"  ✓ Roundtrip successful: {tokens.length} tokens"
    return true
  else
    IO.println "  ✗ Roundtrip failed"
    return false

-- Test 4: Serialization
def testSerialization : IO Bool := do
  IO.println "Test 4: Byte Serialization"
  let source := "let mut x = 10"
  let bytes := lexToBytes source "test.fax"
  
  if bytes.size > 0 then
    IO.println s!"  ✓ Serialized to {bytes.size} bytes"
    return true
  else
    IO.println "  ✗ Serialization failed"
    return false

-- Test 5: Keywords
def testKeywords : IO Bool := do
  IO.println "Test 5: Keywords Recognition"
  let source := "fn let mut if else match struct enum return"
  let tokens := Lexer.lex source
  
  let keywordCount := tokens.filter (fun t =>
    match t with
    | .keyword _ => true
    | _ => false
  ) |>.length
  
  if keywordCount >= 9 then
    IO.println s!"  ✓ Recognized {keywordCount} keywords"
    return true
  else
    IO.println s!"  ✗ Only recognized {keywordCount} keywords"
    return false

-- Test 6: Literals
def testLiterals : IO Bool := do
  IO.println "Test 6: Literals Recognition"
  let source := "42 3.14 \"hello\" 'x' true false"
  let tokens := Lexer.lex source
  
  let literalCount := tokens.filter (fun t =>
    match t with
    | .literal _ => true
    | _ => false
  ) |>.length
  
  if literalCount >= 5 then
    IO.println s!"  ✓ Recognized {literalCount} literals"
    return true
  else
    IO.println s!"  ✗ Only recognized {literalCount} literals"
    return false

-- Run all tests
def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║         Lexer Microservice Unit Tests                     ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  let results ← [
    testBasicTokenization,
    testProtobufConversion,
    testRoundtrip,
    testSerialization,
    testKeywords,
    testLiterals
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

end Tests.Unit.Lexer

def main : IO UInt32 := Tests.Unit.Lexer.runAllTests
