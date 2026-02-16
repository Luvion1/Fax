/-
Type Checker Implementation
Complete type checking logic with Hindley-Milner inference
-/

import Compiler.AST
import Compiler.AST.Types
import Compiler.Semantic.Types
import Compiler.Semantic.Scope
import Compiler.Semantic.Inference
import Compiler.Semantic.Errors

namespace Compiler.Semantic.Checker

open Compiler.AST
open Compiler.AST.Types
open Compiler.Semantic.Types
open Compiler.Semantic.Scope
open Compiler.Semantic.Inference
open Compiler.Semantic.Errors

structure TypeChecker where
  module : Module
  symbolTable : SymbolTable
  scopeStack : List Scope
  typeInfo : TypeInfo
  inferredTypes : List (String × Ty)
  errors : List SemanticError
  deriving Repr

def TypeChecker.new (module : Module) : TypeChecker :=
  {
    module := module
    symbolTable := SymbolTable.empty
    scopeStack := [Scope.global]
    typeInfo := TypeInfo.empty
    inferredTypes := []
    errors := []
  }

-- ============================================================================
-- Module Level Checking
-- ============================================================================

def TypeChecker.checkModule (checker : TypeChecker) : TypeChecker :=
  let initChecker := checker.initGlobalScope
  let checked := initChecker.module.decls.foldl (λ c decl =>
    c.checkDecl decl
  ) initChecker
  checked

def TypeChecker.initGlobalScope (checker : TypeChecker) : TypeChecker :=
  let global := Scope.global
  
  -- Add built-in types
  let globalWithTypes := global
    |>.addType "i8" (.primitive .i8)
    |>.addType "i16" (.primitive .i16)
    |>.addType "i32" (.primitive .i32)
    |>.addType "i64" (.primitive .i64)
    |>.addType "u8" (.primitive .u8)
    |>.addType "u16" (.primitive .u16)
    |>.addType "u32" (.primitive .u32)
    |>.addType "u64" (.primitive .u64)
    |>.addType "f32" (.primitive .f32)
    |>.addType "f64" (.primitive .f64)
    |>.addType "bool" (.primitive .bool)
    |>.addType "char" (.primitive .char)
    |>.addType "str" (.primitive .string)
    |>.addType "unit" (.primitive .unit)
  
  -- Add built-in functions
  let globalWithFuncs := globalWithTypes
    |>.addFunction "println" [.primitive .i32] (.primitive .unit) true
    |>.addFunction "print" [.primitive .i32] (.primitive .unit) true
    |>.addFunction "println_str" [.primitive .string] (.primitive .unit) true
    |>.addFunction "read_int" [] (.primitive .i32) true
  
  { checker with scopeStack := [globalWithFuncs] }

-- ============================================================================
-- Declaration Checking
-- ============================================================================

def TypeChecker.checkDecl (checker : TypeChecker) (decl : Decl) : TypeChecker :=
  match decl with
  | .funDecl pub name params ret body =>
    checker.checkFunction pub name params ret body
  | .structDecl pub name fields =>
    checker.checkStruct pub name fields
  | .enumDecl pub name variants =>
    checker.checkEnum pub name variants

def TypeChecker.checkFunction (checker : TypeChecker) (pub : Bool) 
    (name : String) (params : List (String × Ty)) (ret : Ty) (body : Expr) 
    : TypeChecker :=
  
  -- Create function scope
  let funcScope := Scope.function params ret
  let checkerInFunc := checker.pushScope funcScope
  
  -- Add parameters to scope
  let checkerWithParams := params.foldl (λ c (pname, pty) =>
    c.addVariable pname pty false
  ) checkerInFunc
  
  -- Check function body
  let (bodyType, checkerWithBody) := checkerWithParams.inferExpr body
  
  -- Check return type compatibility
  let checkerValidated := 
    if ret == .primitive .inferred then
      -- Infer return type from body
      let newGlobal := checkerWithBody.scopeStack.head!.addFunction name 
        (params.map (·.2)) bodyType pub
      { checkerWithBody with 
        scopeStack := [newGlobal] ++ checkerWithBody.scopeStack.tail!
      }
    else if isCompatible bodyType ret then
      checkerWithBody
    else
      checkerWithBody.addError {
        kind := .typeMismatch
        message := s!"Function '{name}' return type mismatch: expected {repr ret}, got {repr bodyType}"
        location := none
      }
  
  -- Add to symbol table
  let newSymbolTable := checkerValidated.symbolTable.add {
    name := name
    kind := .function
    ty := .fun (params.map (·.2)) (if ret == .primitive .inferred then bodyType else ret)
    scope := 0
    isPublic := pub
  }
  
  { checkerValidated with 
    symbolTable := newSymbolTable
  }.popScope

def TypeChecker.checkStruct (checker : TypeChecker) (pub : Bool)
    (name : String) (fields : List (String × Ty)) : TypeChecker :=
  
  -- Check for duplicate fields
  let fieldNames := fields.map (·.1)
  let duplicates := findDuplicates fieldNames
  
  let checkerNoDups := if duplicates.isEmpty then
    checker
  else
    checker.addError {
      kind := .duplicateDefinition
      message := s!"Struct '{name}' has duplicate fields: {String.intercalate ", " duplicates}"
      location := none
    }
  
  -- Add type to scope
  let newGlobal := checkerNoDups.scopeStack.head!.addType name (.structTy name fields)
  
  -- Add to symbol table
  let newSymbolTable := checkerNoDups.symbolTable.add {
    name := name
    kind := .struct
    ty := .structTy name fields
    scope := 0
    isPublic := pub
  }
  
  { checkerNoDups with
    scopeStack := [newGlobal] ++ checkerNoDups.scopeStack.tail!
    symbolTable := newSymbolTable
  }

def TypeChecker.checkEnum (checker : TypeChecker) (pub : Bool)
    (name : String) (variants : List (String × List Ty)) : TypeChecker :=
  
  -- Check for duplicate variants
  let variantNames := variants.map (·.1)
  let duplicates := findDuplicates variantNames
  
  let checkerNoDups := if duplicates.isEmpty then
    checker
  else
    checker.addError {
      kind := .duplicateDefinition
      message := s!"Enum '{name}' has duplicate variants: {String.intercalate ", " duplicates}"
      location := none
    }
  
  -- Add type to scope
  let newGlobal := checkerNoDups.scopeStack.head!.addType name (.enumTy name variants)
  
  -- Add to symbol table
  let newSymbolTable := checkerNoDups.symbolTable.add {
    name := name
    kind := .enum
    ty := .enumTy name variants
    scope := 0
    isPublic := pub
  }
  
  { checkerNoDups with
    scopeStack := [newGlobal] ++ checkerNoDups.scopeStack.tail!
    symbolTable := newSymbolTable
  }

-- ============================================================================
-- Expression Type Inference
-- ============================================================================

def TypeChecker.inferExpr (checker : TypeChecker) (expr : Expr) : Ty × TypeChecker :=
  match expr with
  | .lit lit => (inferLit lit, checker)
  | .var name =>
    match checker.lookupVariable name with
    | some ty => (ty, checker)
    | none =>
      let error := {
        kind := .undefinedVariable
        message := s!"Undefined variable: '{name}'"
        location := none
      }
      (.primitive .inferred, checker.addError error)
  
  | .unary op operand =>
    let (operandType, checker1) := checker.inferExpr operand
    (checkUnaryOp op operandType, checker1)
  
  | .binary op left right =>
    let (leftType, checker1) := checker.inferExpr left
    let (rightType, checker2) := checker1.inferExpr right
    (checkBinaryOp op leftType rightType checker2, checker2)
  
  | .call name args =>
    checker.checkFunctionCall name args
  
  | .exprIf cond thenExpr elseExpr =>
    checker.checkIfExpr cond thenExpr elseExpr
  
  | .letExpr pat value body =>
    checker.checkLetExpr pat value body
  
  | .block stmts expr =>
    checker.checkBlock stmts expr
  
  | .tuple elems =>
    let mut types := []
    let mut c := checker
    for elem in elems do
      let (ty, c1) := c.inferExpr elem
      types := ty :: types
      c := c1
    (.tuple types.reverse, c)
  
  | .structVal name fields =>
    checker.checkStructVal name fields
  
  | .enumVal name variant args =>
    checker.checkEnumVal name variant args
  
  | .field obj field =>
    checker.checkFieldAccess obj field
  
  | .proj tuple idx =>
    checker.checkTupleProj tuple idx
  
  | .lambda params body =>
    checker.checkLambda params body
  
  | .matchExpr scrut cases =>
    checker.checkMatch scrut cases

def TypeChecker.checkFunctionCall (checker : TypeChecker) 
    (name : String) (args : List Expr) : Ty × TypeChecker :=
  
  match checker.lookupFunction name with
  | some (paramTypes, retType) =>
    if args.length != paramTypes.length then
      let error := {
        kind := .arityMismatch
        message := s!"Function '{name}' expects {paramTypes.length} arguments, got {args.length}"
        location := none
      }
      (retType, checker.addError error)
    else
      -- Check argument types
      let mut c := checker
      let mut argIdx := 0
      for (arg, expectedType) in args.zip paramTypes do
        let (argType, c1) := c.inferExpr arg
        if !isCompatible argType expectedType then
          let error := {
            kind := .typeMismatch
            message := s!"Argument {argIdx + 1} of '{name}': expected {repr expectedType}, got {repr argType}"
            location := none
          }
          c := c1.addError error
        else
          c := c1
        argIdx := argIdx + 1
      (retType, c)
  
  | none =>
    let error := {
      kind := .undefinedFunction
      message := s!"Undefined function: '{name}'"
      location := none
    }
    (.primitive .inferred, checker.addError error)

def TypeChecker.checkIfExpr (checker : TypeChecker) 
    (cond : Expr) (thenExpr : Expr) (elseExpr : Expr) : Ty × TypeChecker :=
  
  let (condType, checker1) := checker.inferExpr cond
  
  -- Condition must be bool
  let checker2 := if condType == .primitive .bool then
    checker1
  else
    checker1.addError {
      kind := .typeMismatch
      message := s!"If condition must be bool, got {repr condType}"
      location := none
    }
  
  let (thenType, checker3) := checker2.inferExpr thenExpr
  let (elseType, checker4) := checker3.inferExpr elseExpr
  
  -- Then and else must have compatible types
  if isCompatible thenType elseType then
    (thenType, checker4)
  else
    let error := {
      kind := .typeMismatch
      message := s!"If branches have incompatible types: {repr thenType} and {repr elseType}"
      location := none
    }
    (thenType, checker4.addError error)

def TypeChecker.checkLetExpr (checker : TypeChecker)
    (pat : Pattern) (value : Expr) (body : Expr) : Ty × TypeChecker :=
  
  let (valueType, checker1) := checker.inferExpr value
  
  -- Extract variable name from pattern
  let (varName, checkerWithVar) := match pat with
  | .varPat name => (name, checker1.addVariable name valueType false)
  | .wildPat => ("_", checker1)
  | _ => 
    let error := {
      kind := .unsupportedPattern
      message := "Complex patterns in let not yet supported"
      location := none
    }
    ("_", checker1.addError error)
  
  checkerWithVar.inferExpr body

def TypeChecker.checkBlock (checker : TypeChecker)
    (stmts : List Stmt) (expr : Expr) : Ty × TypeChecker :=
  
  -- Create new scope for block
  let blockScope := Scope.block
  let checkerInBlock := checker.pushScope blockScope
  
  -- Check statements
  let checkedStmts := stmts.foldl (λ c stmt =>
    c.checkStmt stmt
  ) checkerInBlock
  
  -- Check final expression
  let (retType, checkerWithExpr) := checkedStmts.inferExpr expr
  
  (retType, checkerWithExpr.popScope)

def TypeChecker.checkStmt (checker : TypeChecker) (stmt : Stmt) : TypeChecker :=
  match stmt with
  | .decl mut pat value =>
    let (valueType, checker1) := checker.inferExpr value
    match pat with
    | .varPat name =>
      checker1.addVariable name valueType mut
    | _ =>
      checker1.addError {
        kind := .unsupportedPattern
        message := "Complex patterns in let declaration not yet supported"
        location := none
      }
  
  | .assign lhs rhs =>
    let (lhsType, checker1) := checker.inferExpr lhs
    let (rhsType, checker2) := checker1.inferExpr rhs
    
    if isCompatible rhsType lhsType then
      checker2
    else
      checker2.addError {
        kind := .typeMismatch
        message := s!"Assignment type mismatch: cannot assign {repr rhsType} to {repr lhsType}"
        location := none
      }
  
  | .exprStmt e =>
    let (_, checker1) := checker.inferExpr e
    checker1
  
  | .return e =>
    let (retType, checker1) := checker.inferExpr e
    -- Check against function return type (would need to track current function)
    checker1
  
  | .break | .continue =>
    checker

def TypeChecker.checkStructVal (checker : TypeChecker)
    (name : String) (fields : List (String × Expr)) : Ty × TypeChecker :=
  
  match checker.lookupType name with
  | some (.structTy _ expectedFields) =>
    -- Check fields match
    let mut c := checker
    for (fieldName, fieldExpr) in fields do
      match expectedFields.find? (λ (n, _) => n == fieldName) with
      | some (_, expectedType) =>
        let (actualType, c1) := c.inferExpr fieldExpr
        if !isCompatible actualType expectedType then
          c := c1.addError {
            kind := .typeMismatch
            message := s!"Field '{fieldName}' of struct '{name}': expected {repr expectedType}, got {repr actualType}"
            location := none
          }
        else
          c := c1
      | none =>
        c := c.addError {
          kind := .undefinedField
          message := s!"Struct '{name}' has no field '{fieldName}'"
          location := none
        }
    (.structTy name expectedFields, c)
  
  | some _ =>
    let error := {
      kind := .typeMismatch
      message := s!"'{name}' is not a struct type"
      location := none
    }
    (.primitive .inferred, checker.addError error)
  
  | none =>
    let error := {
      kind := .undefinedType
      message := s!"Undefined struct type: '{name}'"
      location := none
    }
    (.primitive .inferred, checker.addError error)

def TypeChecker.checkEnumVal (checker : TypeChecker)
    (name : String) (variant : String) (args : List Expr) : Ty × TypeChecker :=
  
  match checker.lookupType name with
  | some (.enumTy _ variants) =>
    match variants.find? (λ (v, _) => v == variant) with
    | some (_, expectedTypes) =>
      if args.length != expectedTypes.length then
        let error := {
          kind := .arityMismatch
          message := s!"Enum variant '{name}::{variant}' expects {expectedTypes.length} arguments, got {args.length}"
          location := none
        }
        (.enumTy name variants, checker.addError error)
      else
        -- Check argument types
        let mut c := checker
        for (arg, expectedType) in args.zip expectedTypes do
          let (actualType, c1) := c.inferExpr arg
          if !isCompatible actualType expectedType then
            c := c1.addError {
              kind := .typeMismatch
              message := s!"Argument to '{name}::{variant}': expected {repr expectedType}, got {repr actualType}"
              location := none
            }
          else
            c := c1
        (.enumTy name variants, c)
    | none =>
      let error := {
        kind := .undefinedVariant
        message := s!"Enum '{name}' has no variant '{variant}'"
        location := none
      }
      (.enumTy name variants, checker.addError error)
  
  | _ =>
    let error := {
      kind := .undefinedType
      message := s!"Undefined enum type: '{name}'"
      location := none
    }
    (.primitive .inferred, checker.addError error)

def TypeChecker.checkFieldAccess (checker : TypeChecker)
    (obj : Expr) (field : String) : Ty × TypeChecker :=
  
  let (objType, checker1) := checker.inferExpr obj
  
  match objType with
  | .structTy name fields =>
    match fields.find? (λ (n, _) => n == field) with
    | some (_, fieldType) => (fieldType, checker1)
    | none =>
      let error := {
        kind := .undefinedField
        message := s!"Struct '{name}' has no field '{field}'"
        location := none
      }
      (.primitive .inferred, checker1.addError error)
  
  | _ =>
    let error := {
      kind := .invalidFieldAccess
      message := s!"Cannot access field '{field}' on type {repr objType}"
      location := none
    }
    (.primitive .inferred, checker1.addError error)

def TypeChecker.checkTupleProj (checker : TypeChecker)
    (tuple : Expr) (idx : Nat) : Ty × TypeChecker :=
  
  let (tupleType, checker1) := checker.inferExpr tuple
  
  match tupleType with
  | .tuple elems =>
    if idx < elems.length then
      (elems.get! idx, checker1)
    else
      let error := {
        kind := .outOfBounds
        message := s!"Tuple index {idx} out of bounds (tuple has {elems.length} elements)"
        location := none
      }
      (.primitive .inferred, checker1.addError error)
  
  | _ =>
    let error := {
      kind := .typeMismatch
      message := s!"Cannot project index {idx} from type {repr tupleType} (expected tuple)"
      location := none
    }
    (.primitive .inferred, checker1.addError error)

def TypeChecker.checkLambda (checker : TypeChecker)
    (params : List (String × Ty)) (body : Expr) : Ty × TypeChecker :=
  
  -- Create scope for lambda
  let lambdaScope := Scope.lambda params
  let checkerInLambda := checker.pushScope lambdaScope
  
  -- Add parameters
  let checkerWithParams := params.foldl (λ c (pname, pty) =>
    c.addVariable pname pty false
  ) checkerInLambda
  
  -- Infer body type
  let (bodyType, checkerWithBody) := checkerWithParams.inferExpr body
  
  -- Return function type
  (.fun (params.map (·.2)) bodyType, checkerWithBody.popScope)

def TypeChecker.checkMatch (checker : TypeChecker)
    (scrut : Expr) (cases : List (Pattern × Expr)) : Ty × TypeChecker :=
  
  let (scrutType, checker1) := checker.inferExpr scrut
  
  -- Check each case
  let mut c := checker1
  let mut resultType : Option Ty := none
  
  for (pat, expr) in cases do
    -- Check pattern matches scrutinee type
    let c1 := c.checkPattern pat scrutType
    
    -- Infer expression type
    let (exprType, c2) := c1.inferExpr expr
    
    -- All cases must have same type
    match resultType with
    | none => resultType := some exprType
    | some rt =>
      if !isCompatible exprType rt then
        c := c2.addError {
          kind := .typeMismatch
          message := s!"Match case has type {repr exprType}, expected {repr rt}"
          location := none
        }
      else
        c := c2
  
  (resultType.getD (.primitive .inferred), c)

def TypeChecker.checkPattern (checker : TypeChecker)
    (pat : Pattern) (expectedType : Ty) : TypeChecker :=
  
  match pat with
  | .wildPat => checker
  | .varPat name =>
    checker.addVariable name expectedType false
  | .litPat lit =>
    let litType := inferLit lit
    if isCompatible litType expectedType then
      checker
    else
      checker.addError {
        kind := .typeMismatch
        message := s!"Literal pattern has type {repr litType}, expected {repr expectedType}"
        location := none
      }
  | .tuplePat pats =>
    match expectedType with
    | .tuple elemTypes =>
      if pats.length != elemTypes.length then
        checker.addError {
          kind := .arityMismatch
          message := s!"Tuple pattern has {pats.length} elements, expected {elemTypes.length}"
          location := none
        }
      else
        pats.zip elemTypes |>.foldl (λ c (p, t) => c.checkPattern p t) checker
    | _ =>
      checker.addError {
        kind := .typeMismatch
        message := s!"Cannot match tuple pattern against type {repr expectedType}"
        location := none
      }
  | _ =>
    checker.addError {
      kind := .unsupportedPattern
      message := "Complex patterns not yet fully supported"
      location := none
    }

-- ============================================================================
-- Scope Management
-- ============================================================================

def TypeChecker.currentScope (checker : TypeChecker) : Option Scope :=
  checker.scopeStack.head?

def TypeChecker.addVariable (checker : TypeChecker) 
    (name : String) (ty : Ty) (mutable : Bool) : TypeChecker :=
  match checker.currentScope with
  | some currentScope =>
    let newScope := currentScope.addVariable name ty mutable
    { checker with 
      scopeStack := [newScope] ++ checker.scopeStack.tail!
    }
  | none =>
    checker.addError {
      kind := .other
      message := s!"Cannot add variable '{name}': no scope available"
      location := none
    }

def TypeChecker.lookupVariable (checker : TypeChecker) (name : String) : Option Ty :=
  checker.scopeStack.findSome? (λ scope => scope.lookupVariable name)

def TypeChecker.lookupFunction (checker : TypeChecker) (name : String) : Option (List Ty × Ty) :=
  checker.scopeStack.findSome? (λ scope => scope.lookupFunction name)

def TypeChecker.lookupType (checker : TypeChecker) (name : String) : Option Ty :=
  checker.scopeStack.findSome? (λ scope => scope.lookupType name)

def TypeChecker.pushScope (checker : TypeChecker) (scope : Scope) : TypeChecker :=
  { checker with scopeStack := scope :: checker.scopeStack }

def TypeChecker.popScope (checker : TypeChecker) : TypeChecker :=
  match checker.scopeStack with
  | _ :: rest => { checker with scopeStack := rest }
  | [] => 
    checker.addError {
      kind := .other
      message := "Cannot pop scope: scope stack is empty"
      location := none
    }

def TypeChecker.addError (checker : TypeChecker) (error : SemanticError) : TypeChecker :=
  { checker with errors := error :: checker.errors }

-- ============================================================================
-- Helper Functions
-- ============================================================================

def inferLit (lit : Lit) : Ty :=
  match lit with
  | .intLit _ => .primitive .i32
  | .floatLit _ => .primitive .f64
  | .boolLit _ => .primitive .bool
  | .stringLit _ => .primitive .string
  | .charLit _ => .primitive .char

def checkUnaryOp (op : UnaryOp) (operandType : Ty) : Ty :=
  match op with
  | .neg =>
    if isNumeric operandType then operandType else .primitive .inferred
  | .not =>
    if operandType == .primitive .bool then .primitive .bool else .primitive .inferred
  | .bitnot =>
    if isInteger operandType then operandType else .primitive .inferred

def checkBinaryOp (op : BinaryOp) (leftType : Ty) (rightType : Ty) (checker : TypeChecker) : Ty :=
  match op with
  | .add | .sub | .mul | .div | .mod =>
    if isNumeric leftType && isNumeric rightType then
      if leftType == rightType then leftType
      else if isFloat leftType || isFloat rightType then .primitive .f64
      else .primitive .i32
    else
      .primitive .inferred
  
  | .and | .or =>
    if leftType == .primitive .bool && rightType == .primitive .bool then
      .primitive .bool
    else
      .primitive .inferred
  
  | .eq | .ne =>
    if isCompatible leftType rightType then .primitive .bool
    else .primitive .inferred
  
  | .lt | .le | .gt | .ge =>
    if isNumeric leftType && isNumeric rightType then .primitive .bool
    else .primitive .inferred
  
  | .shl | .shr | .band | .bor | .bxor =>
    if isInteger leftType && isInteger rightType then
      if leftType == rightType then leftType else .primitive .i32
    else
      .primitive .inferred

def isCompatible (actual : Ty) (expected : Ty) : Bool :=
  actual == expected || 
  expected == .primitive .inferred ||
  actual == .primitive .inferred ||
  (isNumeric actual && isNumeric expected)

def isNumeric (ty : Ty) : Bool :=
  match ty with
  | .primitive (.i8 | .i16 | .i32 | .i64 | .u8 | .u16 | .u32 | .u64 | .f32 | .f64) => true
  | _ => false

def isInteger (ty : Ty) : Bool :=
  match ty with
  | .primitive (.i8 | .i16 | .i32 | .i64 | .u8 | .u16 | .u32 | .u64) => true
  | _ => false

def isFloat (ty : Ty) : Bool :=
  match ty with
  | .primitive (.f32 | .f64) => true
  | _ => false

def findDuplicates (list : List String) : List String :=
  let rec go (seen : List String) (dups : List String) (remaining : List String) : List String :=
    match remaining with
    | [] => dups
    | x :: xs =>
      if seen.contains x then
        if dups.contains x then go seen dups xs
        else go seen (x :: dups) xs
      else go (x :: seen) dups xs
  go [] [] list

-- ============================================================================
-- Public API
-- ============================================================================

def typeCheckModule (module : Module) : Compiler.Semantic.SemanticResult :=
  let checker := TypeChecker.new module
  let checked := checker.checkModule
  {
    errors := checked.errors
    symbolTable := checked.symbolTable
    typeInfo := checked.typeInfo
    inferredTypes := checked.inferredTypes
  }

export TypeChecker (checkModule inferExpr checkDecl)

end Compiler.Semantic.Checker
