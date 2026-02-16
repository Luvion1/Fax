/-
Simple Compiler Driver for Testing
Direct compilation without protobuf overhead
-/

import Compiler.Lexer
import Compiler.Parser
import Compiler.Codegen
import Compiler.Semantic
import Fax.StdLib

namespace Compiler.Driver.Simple

open Compiler.AST

-- Simple compile function
def compile (source : String) : Except String String := do
  -- Step 1: Lexical analysis
  let tokens := Lexer.lex source
  
  -- Step 2: Parsing
  let module ← Parser.parseModule tokens
  
  -- Step 3: Semantic analysis (optional, for type checking)
  -- let typeState := Semantic.checkModule module
  -- if !typeState.errors.isEmpty then
  --   Except.error (String.intercalate "\n" typeState.errors)
  -- else
  --   pure ()
  
  -- Step 4: Add standard library
  let moduleWithStd := Fax.StdLib.addStdLibToModule module
  
  -- Step 5: Code generation
  let ir := Compiler.Codegen.generateIR moduleWithStd
  
  -- Add standard library runtime
  let fullIR := Fax.StdLib.generateStdLibRuntime ++ ir
  
  return fullIR

-- Compile with detailed output
def compileDetailed (source : String) : Except String (List Token × Module × String) := do
  let tokens := Lexer.lex source
  let module ← Parser.parseModule tokens
  let ir ← compile source
  return (tokens, module, ir)

-- Compile file
def compileFile (filepath : String) : IO (Except String String) := do
  try
    let source ← IO.readFile filepath
    return compile source
  catch e =>
    return Except.error s!"Failed to read file: {e}"

-- Write compiled output
def writeIR (ir : String) (filepath : String) : IO Unit := do
  IO.writeFile filepath ir

-- Main entry point
def main (args : List String) : IO UInt32 := do
  if args.isEmpty then
    IO.println "Fax Compiler v0.2.0 (Simple Driver)"
    IO.println ""
    IO.println "Usage: faxc-simple <input.fax> [output.ll]"
    IO.println ""
    IO.println "Options:"
    IO.println "  -v, --version    Show version"
    IO.println "  -h, --help       Show this help"
    return 1
  
  let inputFile := args.head!
  
  if inputFile == "-v" || inputFile == "--version" then
    IO.println "Fax Compiler v0.2.0"
    return 0
  
  if inputFile == "-h" || inputFile == "--help" then
    IO.println "Fax Compiler v0.2.0 (Simple Driver)"
    IO.println ""
    IO.println "Usage: faxc-simple <input.fax> [output.ll]"
    return 0
  
  let outputFile := if args.length > 1 then args.get! 1 else inputFile.replace ".fax" ".ll"
  
  IO.println s!"Compiling {inputFile}..."
  
  match ← compileFile inputFile with
  | Except.error err =>
    IO.println s!"Error: {err}"
    return 1
  | Except.ok ir =>
    writeIR ir outputFile
    IO.println s!"Generated: {outputFile}"
    return 0

end Compiler.Driver.Simple
