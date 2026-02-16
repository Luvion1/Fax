/-
Semantic Analysis with Protobuf input/output
Accepts Module in protobuf format and returns analysis results
Follows same pattern as Lexer.Proto, Parser.Proto, and Codegen.Proto
-/

import Compiler.Semantic
import Compiler.Proto
import Compiler.Proto.Messages
import Compiler.Proto.Services

namespace Compiler.Semantic.Proto

open Proto
open Proto.Messages
open Proto.Services

-- ============================================================================
-- Protobuf Input/Output Functions (Following the same pattern)
-- ============================================================================

-- Analyze protobuf Module and return analysis result
def analyzeProtobuf (module : Proto.Messages.Module) : SemanticResult :=
  let ast := Proto.Converters.Module.toAST module
  Semantic.checkModule ast

-- Analyze protobuf Module and return detailed response
def analyzeWithResponse (module : Proto.Messages.Module) : AnalyzeResponse :=
  let result := analyzeProtobuf module
  {
    valid := result.errors.isEmpty,
    errors := result.errors,
    symbolTable := result.symbolTable,
    typeInfo := result.typeInfo
  }

-- Analyze from serialized protobuf bytes
def analyzeFromBytes (data : ByteArray) : Except String SemanticResult := do
  let module ← match Proto.deserializeModule data with
    | some m => Except.ok m
    | none => Except.error "Failed to deserialize Module"
  return analyzeProtobuf module

-- Convert AST Module to analysis result
def analyzeFromAST (module : AST.Module) : SemanticResult :=
  Semantic.checkModule module

-- ============================================================================
-- Semantic Result Structure
-- ============================================================================

structure SemanticResult where
  errors : List String
  symbolTable : SymbolTable
  typeInfo : TypeInfo
  deriving Repr

def SemanticResult.isValid (r : SemanticResult) : Bool :=
  r.errors.isEmpty

structure SymbolTable where
  symbols : List (String × SymbolInfo)
  deriving Repr

structure SymbolInfo where
  name : String
  kind : SymbolKind
  ty : Ty
  scope : Nat
  deriving Repr

inductive SymbolKind
  | variable
  | function
  | struct
  | enum
  deriving Repr

structure TypeInfo where
  types : List (String × Ty)
  deriving Repr

structure AnalyzeResponse where
  valid : Bool
  errors : List String
  symbolTable : SymbolTable
  typeInfo : TypeInfo
  deriving Repr

-- ============================================================================
-- Service Handler (For gRPC Integration)
-- ============================================================================

def handleAnalyzeService (request : AnalyzeRequest) : IO AnalyzeResponse := do
  try
    let result := analyzeProtobuf request.module
    return {
      valid := result.errors.isEmpty,
      errors := result.errors,
      symbolTable := result.symbolTable,
      typeInfo := result.typeInfo
    }
  catch e =>
    return {
      valid := false,
      errors := [s!"Analysis failed: {e}"],
      symbolTable := { symbols := [] },
      typeInfo := { types := [] }
    }

-- Request structure for analysis service
structure AnalyzeRequest where
  module : Proto.Messages.Module
  options : AnalyzeOptions
  deriving Repr

structure AnalyzeOptions where
  strictMode : Bool
  warnUnused : Bool
  deriving Repr

def AnalyzeOptions.default : AnalyzeOptions :=
  { strictMode := false, warnUnused := true }

-- ============================================================================
-- Symbol Table Construction from Protobuf Module
-- ============================================================================

def buildSymbolTable (module : Proto.Messages.Module) : SymbolTable :=
  let symbols := module.decls.foldl (fun acc decl =>
    match decl with
    | .func name params ret _ =>
      acc ++ [{
        name := name,
        kind := .function,
        ty := .fun (params.map (·.2)) ret,
        scope := 0
      }]
    | .struct name fields =>
      acc ++ [{
        name := name,
        kind := .struct,
        ty := .structTy name fields,
        scope := 0
      }]
    | .enum name variants =>
      acc ++ [{
        name := name,
        kind := .enum,
        ty := .enumTy name variants,
        scope := 0
      }]
  ) []
  { symbols := symbols }

-- ============================================================================
-- Type Info Extraction
-- ============================================================================

def extractTypeInfo (module : Proto.Messages.Module) : TypeInfo :=
  let types := module.decls.foldl (fun acc decl =>
    match decl with
    | .func name params ret _ =>
      acc ++ [(s!"{name}_ret", ret)] ++ params.map (fun (n, t) => (s!"{name}_{n}", t))
    | _ => acc
  ) []
  { types := types }

-- ============================================================================
-- Validation Functions
-- ============================================================================

def validateModule (module : Proto.Messages.Module) : List String :=
  let mut errors := []
  
  -- Check for duplicate function names
  let funcNames := module.decls.filterMap (fun d =>
    match d with | .func n _ _ _ => some n | _ => none
  )
  let duplicates := findDuplicates funcNames
  errors := errors ++ duplicates.map (fun name => s!"Duplicate function: {name}")
  
  -- Check for main function if needed
  let hasMain := funcNames.contains "main"
  if !hasMain then
    errors := errors ++ ["Warning: No 'main' function defined"]
  
  errors

private def findDuplicates (list : List String) : List String :=
  let rec go (seen : List String) (dups : List String) (remaining : List String) : List String :=
    match remaining with
    | [] => dups
    | x :: xs =>
      if seen.contains x then
        if dups.contains x then
          go seen dups xs
        else
          go seen (x :: dups) xs
      else
        go (x :: seen) dups xs
  go [] [] list

-- ============================================================================
-- Batch Analysis
-- ============================================================================

def analyzeBatch (modules : List Proto.Messages.Module) : List AnalyzeResponse :=
  modules.map analyzeWithResponse

-- ============================================================================
-- Incremental Analysis
-- ============================================================================

def analyzeIncremental (oldModule : Proto.Messages.Module) 
    (newModule : Proto.Messages.Module) 
    (changedDecls : List String) : AnalyzeResponse :=
  -- Only re-analyze changed declarations
  let filteredDecls := newModule.decls.filter (fun d =>
    match d with
    | .func n _ _ _ => changedDecls.contains n
    | .struct n _ => changedDecls.contains n
    | .enum n _ => changedDecls.contains n
  )
  
  let partialModule := { newModule with decls := filteredDecls }
  analyzeWithResponse partialModule

end Compiler.Semantic.Proto
