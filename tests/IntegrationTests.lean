-- Integration test for Fax compiler
import Compiler.Driver.IO
import Compiler.Lexer
import Compiler.Parser
import Compiler.Codegen

namespace Tests

-- Test 1: Lexer test
def testLexer : IO Unit := do
  let source := "fn main() { 42 }"
  let tokens := Lexer.lex source
  IO.println "=== Test 1: Lexer ==="
  IO.println s!"Source: {source}"
  IO.println s!"Tokens: {tokens.length}"
  for token in tokens do
    IO.println s!"  {repr token}"
  IO.println ""

-- Test 2: Parser test
def testParser : IO Unit := do
  let source := "fn main() -> i32 { 42 }"
  let tokens := Lexer.lex source
  IO.println "=== Test 2: Parser ==="
  IO.println s!"Source: {source}"
  match Parser.parseModule tokens with
  | Except.ok module => 
    IO.println "Parse successful!"
    IO.println s!"Declarations: {module.decls.length}"
  | Except.error err => 
    IO.println s!"Parse error: {err}"
  IO.println ""

-- Test 3: Codegen test - simple function
def testCodegenSimple : IO Unit := do
  let source := "fn main() -> i32 { 42 }"
  IO.println "=== Test 3: Simple Codegen ==="
  IO.println s!"Source: {source}"
  
  let tokens := Lexer.lex source
  match Parser.parseModule tokens with
  | Except.ok module =>
    let ir := Compiler.Codegen.generateIR module
    IO.println "Generated LLVM IR:"
    IO.println "---"
    IO.println ir
    IO.println "---"
  | Except.error err =>
    IO.println s!"Error: {err}"
  IO.println ""

-- Test 4: Codegen test - arithmetic
def testCodegenArithmetic : IO Unit := do
  let source := "fn add() -> i32 { 1 + 2 }"
  IO.println "=== Test 4: Arithmetic Codegen ==="
  IO.println s!"Source: {source}"
  
  let tokens := Lexer.lex source
  match Parser.parseModule tokens with
  | Except.ok module =>
    let ir := Compiler.Codegen.generateIR module
    IO.println "Generated LLVM IR:"
    IO.println "---"
    IO.println ir
    IO.println "---"
  | Except.error err =>
    IO.println s!"Error: {err}"
  IO.println ""

-- Test 5: Codegen test - if expression
def testCodegenIf : IO Unit := do
  let source := "fn max() -> i32 { if 5 > 3 { 5 } else { 3 } }"
  IO.println "=== Test 5: If Expression Codegen ==="
  IO.println s!"Source: {source}"
  
  let tokens := Lexer.lex source
  match Parser.parseModule tokens with
  | Except.ok module =>
    let ir := Compiler.Codegen.generateIR module
    IO.println "Generated LLVM IR:"
    IO.println "---"
    IO.println ir
    IO.println "---"
  | Except.error err =>
    IO.println s!"Error: {err}"
  IO.println ""

-- Test 6: Codegen test - let binding
def testCodegenLet : IO Unit := do
  let source := "fn foo() -> i32 { let x = 10; x + 5 }"
  IO.println "=== Test 6: Let Binding Codegen ==="
  IO.println s!"Source: {source}"
  
  let tokens := Lexer.lex source
  match Parser.parseModule tokens with
  | Except.ok module =>
    let ir := Compiler.Codegen.generateIR module
    IO.println "Generated LLVM IR:"
    IO.println "---"
    IO.println ir
    IO.println "---"
  | Except.error err =>
    IO.println s!"Error: {err}"
  IO.println ""

-- Test 7: Codegen test - function call
def testCodegenCall : IO Unit := do
  let source := "fn add(a: i32, b: i32) -> i32 { a + b }
fn main() -> i32 { add(1, 2) }"
  IO.println "=== Test 7: Function Call Codegen ==="
  IO.println s!"Source: {source}"
  
  let tokens := Lexer.lex source
  match Parser.parseModule tokens with
  | Except.ok module =>
    let ir := Compiler.Codegen.generateIR module
    IO.println "Generated LLVM IR:"
    IO.println "---"
    IO.println ir
    IO.println "---"
  | Except.error err =>
    IO.println s!"Error: {err}"
  IO.println ""

-- Run all tests
def runAllTests : IO Unit := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║           Fax Compiler Integration Tests                  ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  testLexer
  testParser
  testCodegenSimple
  testCodegenArithmetic
  testCodegenIf
  testCodegenLet
  testCodegenCall
  
  IO.println "═══════════════════════════════════════════════════════════"
  IO.println "                     Tests Complete                         "
  IO.println "═══════════════════════════════════════════════════════════"

end Tests

def main : IO Unit := Tests.runAllTests
