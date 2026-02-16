/-
Test utilities and examples for Fax compiler protobuf integration
-/

import Compiler.Proto

namespace Compiler.Proto.Test

-- Test source code examples
def testSource1 : String := "
fn main() {
    println(\"Hello, World!\")
}
"

def testSource2 : String := "
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = add(1, 2)
    println(result)
}
"

def testSource3 : String := "
struct Point {
    x: f64,
    y: f64,
}

fn distance(p1: Point, p2: Point) -> f64 {
    let dx = p1.x - p2.x
    let dy = p1.y - p2.y
    sqrt(dx * dx + dy * dy)
}
"

-- Test protobuf serialization roundtrip
def testTokenStreamRoundtrip (source : String) : IO Bool := do
  IO.println "Testing TokenStream serialization..."
  
  -- Lex and convert to protobuf
  let tokens := Compiler.Lexer.lex source
  let tokenStream := Converters.tokensToProto tokens "test.fax" source
  
  -- Serialize
  let bytes := Proto.serializeTokenStream tokenStream
  IO.println s!"Serialized to {bytes.size} bytes"
  
  -- Deserialize
  match Proto.deserializeTokenStream bytes with
  | some ts =>
    IO.println "Deserialization successful"
    -- Check if tokens match
    let tokens' := Converters.tokenStreamToLexer ts
    if tokens.length == tokens'.length then
      IO.println "✓ Token count matches"
      return true
    else
      IO.println s!"✗ Token count mismatch: {tokens.length} vs {tokens'.length}"
      return false
  | none =>
    IO.println "✗ Deserialization failed"
    return false

-- Test module serialization roundtrip
def testModuleRoundtrip (source : String) : IO Bool := do
  IO.println "\nTesting Module serialization..."
  
  -- Parse to module
  let tokens := Compiler.Lexer.lex source
  match Compiler.Parser.parseModule tokens with
  | Except.ok module =>
    let protoModule := Converters.AST.Module.toProto module
    
    -- Serialize
    let bytes := Proto.serializeModule protoModule
    IO.println s!"Serialized to {bytes.size} bytes"
    
    -- Deserialize
    match Proto.deserializeModule bytes with
    | some m =>
      IO.println "Deserialization successful"
      IO.println s!"Module name: {m.name}"
      IO.println s!"Declarations: {m.decls.length}"
      return true
    | none =>
      IO.println "✗ Deserialization failed"
      return false
  | Except.error e =>
    IO.println s!"✗ Parse error: {e}"
    return false

-- Test semantic analysis
def testSemanticAnalysis (source : String) : IO Bool := do
  IO.println "\nTesting semantic analysis..."
  
  let tokens := Compiler.Lexer.lex source
  match Compiler.Parser.parseModule tokens with
  | Except.ok module =>
    let protoModule := Converters.AST.Module.toProto module
    let result := Semantic.runSemanticAnalysis protoModule
    
    IO.println s!"Errors found: {result.errors.length}"
    for error in result.errors do
      IO.println s!"  - {repr error}"
    
    if result.isValid then
      IO.println "✓ Semantic analysis passed"
      return true
    else
      IO.println "✗ Semantic analysis failed"
      return false
  | Except.error e =>
    IO.println s!"✗ Parse error: {e}"
    return false

-- Test caching
def testCaching : IO Bool := do
  IO.println "\nTesting caching system..."
  
  let cache ← Cache.createCache {}
  let source := testSource1
  
  -- First access - cache miss
  let result1 ← CacheOps.getCachedTokenStream cache source
  match result1 with
  | some _ => 
    IO.println "✗ Unexpected cache hit"
    return false
  | none =>
    IO.println "✓ Cache miss (expected)"
  
  -- Cache the token stream
  let tokens := Compiler.Lexer.lex source
  let tokenStream := Converters.tokensToProto tokens "test.fax" source
  CacheOps.cacheTokenStream cache source tokenStream
  
  -- Second access - cache hit
  let result2 ← CacheOps.getCachedTokenStream cache source
  match result2 with
  | some ts =>
    IO.println s!"✓ Cache hit, got {ts.tokens.length} tokens"
    return true
  | none =>
    IO.println "✗ Cache miss (unexpected)"
    return false

-- Test full compilation pipeline
def testFullPipeline (source : String) : IO Bool := do
  IO.println "\nTesting full compilation pipeline..."
  
  match Driver.Proto.compileWithProtobuf source with
  | Except.ok ir =>
    IO.println "✓ Compilation successful"
    IO.println s!"Generated {ir.length} characters of LLVM IR"
    return true
  | Except.error e =>
    IO.println s!"✗ Compilation failed: {e}"
    return false

-- Run all tests
def runAllTests : IO Unit := do
  IO.println "========================================"
  IO.println "Fax Compiler Protobuf Integration Tests"
  IO.println "========================================"
  
  let mut passed := 0
  let mut failed := 0
  
  -- Test 1: TokenStream roundtrip
  if ← testTokenStreamRoundtrip testSource1 then
    passed := passed + 1
  else
    failed := failed + 1
  
  -- Test 2: Module roundtrip
  if ← testModuleRoundtrip testSource1 then
    passed := passed + 1
  else
    failed := failed + 1
  
  -- Test 3: Semantic analysis
  if ← testSemanticAnalysis testSource2 then
    passed := passed + 1
  else
    failed := failed + 1
  
  -- Test 4: Caching
  if ← testCaching then
    passed := passed + 1
  else
    failed := failed + 1
  
  -- Test 5: Full pipeline
  if ← testFullPipeline testSource1 then
    passed := passed + 1
  else
    failed := failed + 1
  
  -- Summary
  IO.println "\n========================================"
  IO.println s!"Tests passed: {passed}"
  IO.println s!"Tests failed: {failed}"
  IO.println "========================================"
  
  if failed == 0 then
    IO.println "All tests passed! ✓"
    IO.exit 0
  else
    IO.println "Some tests failed! ✗"
    IO.exit 1

-- Benchmark protobuf serialization
def benchmarkSerialization (source : String) (iterations : Nat := 100) : IO Unit := do
  IO.println s!"\nBenchmarking serialization ({iterations} iterations)..."
  
  let tokens := Compiler.Lexer.lex source
  let tokenStream := Converters.tokensToProto tokens "bench.fax" source
  
  let start ← IO.monoMsNow
  
  for _ in [:iterations] do
    let bytes := Proto.serializeTokenStream tokenStream
    let _ := Proto.deserializeTokenStream bytes
    pure ()
  
  let end ← IO.monoMsNow
  let elapsed := end - start
  let avg := elapsed.toFloat / iterations.toFloat
  
  IO.println s!"Total time: {elapsed}ms"
  IO.println s!"Average per iteration: {avg}ms"

-- Example: Using protobuf for IPC
def exampleIPC : IO Unit := do
  IO.println "\nExample: Inter-Process Communication"
  IO.println "====================================="
  
  -- Simulate compiler frontend
  let source := testSource2
  let tokens := Compiler.Lexer.lex source
  let tokenStream := Converters.tokensToProto tokens "example.fax" source
  let tokenBytes := Proto.serializeTokenStream tokenStream
  
  IO.println s!"Frontend: Serialized token stream ({tokenBytes.size} bytes)"
  
  -- Simulate sending over IPC
  -- In real scenario, this would be sent to another process
  
  -- Simulate compiler backend
  match Proto.deserializeTokenStream tokenBytes with
  | some ts =>
    IO.println "Backend: Deserialized token stream"
    let tokens' := Converters.tokenStreamToLexer ts
    IO.println s!"  Tokens: {tokens'.length}"
    
    match Compiler.Parser.parseModule tokens' with
    | Except.ok module =>
      IO.println "  Parsed successfully"
      let protoModule := Converters.AST.Module.toProto module
      let moduleBytes := Proto.serializeModule protoModule
      IO.println s!"  Serialized module ({moduleBytes.size} bytes)"
    | Except.error e =>
      IO.println s!"  Parse error: {e}"
  | none =>
    IO.println "Failed to deserialize"

-- Main entry for tests
def main (args : List String) : IO Unit :=
  match args with
  | ["test"] => runAllTests
  | ["benchmark"] => benchmarkSerialization testSource2
  | ["example"] => exampleIPC
  | _ =>
    IO.println "Usage: test-proto [test|benchmark|example]"
    IO.println "  test      - Run all tests"
    IO.println "  benchmark - Benchmark serialization"
    IO.println "  example   - Show IPC example"

end Compiler.Proto.Test
