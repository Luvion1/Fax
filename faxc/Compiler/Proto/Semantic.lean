/-
Semantic Analysis Layer for Fax Compiler
Analyzes AST and produces typed, validated AST via protobuf
-/

import Compiler.Proto.Messages
import Compiler.Proto.Codec

namespace Compiler.Proto.Semantic

open Messages

-- Semantic error types
inductive SemanticError
  | typeMismatch (expected : Ty) (actual : Ty) (location : String)
  | undefinedVariable (name : String) (location : String)
  | undefinedFunction (name : String) (location : String)
  | undefinedType (name : String) (location : String)
  | duplicateDefinition (name : String) (location : String)
  | invalidPattern (pat : Pattern) (ty : Ty) (location : String)
  | missingReturn (function : String)
  | unreachableCode (location : String)
  deriving Repr

-- Symbol table entry
inductive Symbol
  | variable (name : String) (ty : Ty) (mutable : Bool)
  | function (name : String) (params : List Ty) (ret : Ty)
  | type_ (name : String) (def : Ty)
  deriving Repr

-- Symbol table
structure SymbolTable where
  scopes : List (List Symbol) := [[]]
  deriving Repr

def SymbolTable.empty : SymbolTable := {}

def SymbolTable.enterScope (st : SymbolTable) : SymbolTable :=
  { scopes := [] :: st.scopes }

def SymbolTable.exitScope (st : SymbolTable) : SymbolTable :=
  match st.scopes with
  | [] => st
  | _ :: rest => { scopes := rest }

def SymbolTable.add (st : SymbolTable) (sym : Symbol) : SymbolTable :=
  match st.scopes with
  | [] => { scopes := [[sym]] }
  | current :: rest => { scopes := (sym :: current) :: rest }

def SymbolTable.lookup (st : SymbolTable) (name : String) : Option Symbol :=
  let rec searchScopes (scopes : List (List Symbol)) : Option Symbol :=
    match scopes with
    | [] => none
    | current :: rest =>
      match current.find? (λ s =>
        match s with
        | .variable n _ _ => n == name
        | .function n _ _ => n == name
        | .type_ n _ => n == name) with
      | some s => some s
      | none => searchScopes rest
  searchScopes st.scopes

-- Semantic analysis environment
structure SemEnv where
  symbols : SymbolTable
  errors : List SemanticError
  currentFunction : Option String
  currentReturnType : Option Ty
  deriving Repr

def SemEnv.empty : SemEnv :=
  { symbols := SymbolTable.empty, errors := [], currentFunction := none, currentReturnType := none }

def SemEnv.addError (env : SemEnv) (err : SemanticError) : SemEnv :=
  { env with errors := err :: env.errors }

-- Type checking
partial def inferType (env : SemEnv) (expr : Expr) : Ty × SemEnv :=
  match expr with
  | .lit l =>
    match l with
    | .int _ => (.primitive .i32, env)
    | .float _ => (.primitive .f64, env)
    | .bool _ => (.primitive .bool, env)
    | .string _ => (.primitive .string, env)
    | .char _ => (.primitive .char, env)
  | .var name =>
    match env.symbols.lookup name with
    | some (.variable _ ty _) => (ty, env)
    | _ =>
      let env' := env.addError (.undefinedVariable name "")
      (.primitive .inferred, env')
  | .tuple elems =>
    let (types, env') := elems.foldl (λ (acc, e) elem =>
      let (ty, e') := inferType e elem
      (ty :: acc, e')) ([], env)
    (.tuple types.reverse, env')
  | .binary op e1 e2 =>
    let (t1, env1) := inferType env e1
    let (t2, env2) := inferType env1 e2
    -- Check type compatibility
    match op with
    | .add | .sub | .mul | .div | .mod =>
      if t1 == t2 then
        (t1, env2)
      else
        let env' := env2.addError (.typeMismatch t1 t2 "")
        (.primitive .inferred, env')
    | .eq | .ne | .lt | .le | .gt | .ge =>
      (.primitive .bool, env2)
    | .and | .or =>
      (.primitive .bool, env2)
    | _ =>
      (t1, env2)
  | .call fnName args =>
    match env.symbols.lookup fnName with
    | some (.function _ _ ret) => (ret, env)
    | _ =>
      let env' := env.addError (.undefinedFunction fnName "")
      (.primitive .inferred, env')
  | _ =>
    (.primitive .inferred, env)

-- Check statement
def checkStmt (env : SemEnv) (stmt : Stmt) : SemEnv :=
  match stmt with
  | .decl isMut pat value =>
    let (valTy, env') := inferType env value
    match pat with
    | .var name =>
      let sym := Symbol.variable name valTy isMut
      { env' with symbols := env'.symbols.add sym }
    | _ => env'
  | .assign lhs rhs =>
    let (lhsTy, env1) := inferType env lhs
    let (rhsTy, env2) := inferType env1 rhs
    if lhsTy == rhsTy then
      env2
    else
      env2.addError (.typeMismatch lhsTy rhsTy "")
  | .return e =>
    match env.currentReturnType with
    | some expected =>
      let (actual, env') := inferType env e
      if expected == actual then
        env'
      else
        env'.addError (.typeMismatch expected actual "")
    | none =>
      env.addError (.missingReturn "")
  | _ => env

-- Analyze declaration
def analyzeDecl (env : SemEnv) (decl : Decl) : SemEnv :=
  match decl.declKind with
  | .funDecl f =>
    -- Add function to symbol table
    let paramTypes := f.params.map (λ p => p.paramType)
    let fnSym := Symbol.function f.name paramTypes f.returnType
    let env' := { env with symbols := env.symbols.add fnSym }
    
    -- Enter function scope
    let env'' := { env' with
      symbols := env'.symbols.enterScope
      currentFunction := some f.name
      currentReturnType := some f.returnType
    }
    
    -- Add parameters to scope
    let env''' := f.params.foldl (λ e p =>
      { e with symbols := e.symbols.add (Symbol.variable p.name p.paramType false) }) env''
    
    -- Analyze body
    let env'''' := checkStmt env''' (.exprStmt f.body)
    
    -- Exit scope
    { env'''' with
      symbols := env''''.symbols.exitScope
      currentFunction := env.currentFunction
      currentReturnType := env.currentReturnType
    }
  | .structDecl s =>
    let tySym := Symbol.type_ s.name (.structTy s.name (s.fields.map (λ f => (f.name, f.fieldType))))
    { env with symbols := env.symbols.add tySym }
  | .enumDecl e =>
    let tySym := Symbol.type_ e.name (.enumTy e.name (e.variants.map (λ v => (v.name, v.payloadTypes))))
    { env with symbols := env.symbols.add tySym }

-- Analyze module
def analyzeModule (m : Module) : SemEnv :=
  m.decls.foldl analyzeDecl SemEnv.empty

-- Semantic analysis result
structure AnalysisResult where
  module : Module
  errors : List SemanticError
  symbols : SymbolTable
  deriving Repr

-- Run full semantic analysis
def runSemanticAnalysis (m : Module) : AnalysisResult :=
  let env := analyzeModule m
  { module := m, errors := env.errors.reverse, symbols := env.symbols }

-- Check if analysis succeeded
def AnalysisResult.isValid (r : AnalysisResult) : Bool :=
  r.errors.isEmpty

-- Semantic analysis via protobuf
def analyzeModuleProtobuf (data : ByteArray) : Except String AnalysisResult := do
  let m ← Codec.deserializeModule data
  return runSemanticAnalysis m

-- Serialize analysis result
def AnalysisResult.serialize (r : AnalysisResult) : ByteArray :=
  -- Serialize module and errors
  Codec.serializeModule r.module

end Compiler.Proto.Semantic
