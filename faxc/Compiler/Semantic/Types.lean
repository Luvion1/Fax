/-
Semantic Analysis - Type Definitions and Operations
-/

import Compiler.AST
import Compiler.AST.Types

namespace Compiler.Semantic.Types

open Compiler.AST
open Compiler.AST.Types

-- Semantic error types
inductive SemanticErrorKind
  | typeMismatch
  | undefinedVariable
  | undefinedFunction
  | undefinedType
  | undefinedField
  | undefinedVariant
  | duplicateDefinition
  | arityMismatch
  | invalidFieldAccess
  | unsupportedPattern
  | outOfBounds
  | other
  deriving Repr, BEq

structure SemanticError where
  kind : SemanticErrorKind
  message : String
  location : Option SourceLocation
  deriving Repr

structure SourceLocation where
  filename : String
  line : Nat
  column : Nat
  deriving Repr

-- Symbol information for symbol table
structure SymbolInfo where
  name : String
  kind : SymbolKind
  ty : Ty
  scope : Nat
  isPublic : Bool := false
  isMutable : Bool := false
  deriving Repr

inductive SymbolKind
  | variable
  | function
  | struct
  | enum
  | type
  | module
  deriving Repr, BEq

-- Symbol table
def SymbolTable := List SymbolInfo

def SymbolTable.empty : SymbolTable := []

def SymbolTable.add (table : SymbolTable) (sym : SymbolInfo) : SymbolTable :=
  sym :: table

def SymbolTable.find (table : SymbolTable) (name : String) : Option SymbolInfo :=
  table.find? (λ s => s.name == name)

def SymbolTable.findByKind (table : SymbolTable) (name : String) (kind : SymbolKind) : Option SymbolInfo :=
  table.find? (λ s => s.name == name && s.kind == kind)

def SymbolTable.remove (table : SymbolTable) (name : String) : SymbolTable :=
  table.filter (λ s => s.name != name)

-- Type information
def TypeInfo := List (String × Ty)

def TypeInfo.empty : TypeInfo := []

def TypeInfo.add (info : TypeInfo) (name : String) (ty : Ty) : TypeInfo :=
  (name, ty) :: info

def TypeInfo.lookup (info : TypeInfo) (name : String) : Option Ty :=
  info.find? (λ (n, _) => n == name) |>.map (·.2)

-- Scope management
structure Scope where
  kind : ScopeKind
  variables : List (String × Ty × Bool)  -- name, type, mutable
  functions : List (String × List Ty × Ty × Bool)  -- name, params, ret, public
  types : List (String × Ty)
  level : Nat
  deriving Repr

inductive ScopeKind
  | global
  | function
  | block
  | lambda
  | loop
  deriving Repr, BEq

def Scope.global : Scope :=
  { kind := .global, variables := [], functions := [], types := [], level := 0 }

def Scope.function (params : List (String × Ty)) (ret : Ty) : Scope :=
  { kind := .function, variables := [], functions := [], types := [], level := 1 }

def Scope.block : Scope :=
  { kind := .block, variables := [], functions := [], types := [], level := 0 }

def Scope.lambda (params : List (String × Ty)) : Scope :=
  { kind := .lambda, variables := [], functions := [], types := [], level := 0 }

def Scope.addVariable (scope : Scope) (name : String) (ty : Ty) (mutable : Bool) : Scope :=
  { scope with variables := (name, ty, mutable) :: scope.variables }

def Scope.addFunction (scope : Scope) (name : String) (params : List Ty) (ret : Ty) (pub : Bool) : Scope :=
  { scope with functions := (name, params, ret, pub) :: scope.functions }

def Scope.addType (scope : Scope) (name : String) (ty : Ty) : Scope :=
  { scope with types := (name, ty) :: scope.types }

def Scope.lookupVariable (scope : Scope) (name : String) : Option Ty :=
  scope.variables.find? (λ (n, _, _) => n == name) |>.map (λ (_, t, _) => t)

def Scope.lookupFunction (scope : Scope) (name : String) : Option (List Ty × Ty) :=
  scope.functions.find? (λ (n, _, _, _) => n == name) |>.map (λ (_, p, r, _) => (p, r))

def Scope.lookupType (scope : Scope) (name : String) : Option Ty :=
  scope.types.find? (λ (n, _) => n == name) |>.map (·.2)

-- Type operations
namespace Ty

-- Get string representation of type
def toString (ty : Ty) : String :=
  match ty with
  | .primitive p =>
    match p with
    | .unit => "unit"
    | .i8 => "i8"
    | .i16 => "i16"
    | .i32 => "i32"
    | .i64 => "i64"
    | .u8 => "u8"
    | .u16 => "u16"
    | .u32 => "u32"
    | .u64 => "u64"
    | .f32 => "f32"
    | .f64 => "f64"
    | .bool => "bool"
    | .char => "char"
    | .string => "str"
    | .inferred => "_"
  | .array elem size => s!"[{toString elem}; {size}]"
  | .tuple elems => s!"({String.intercalate ", " (elems.map toString)})"
  | .structTy name _ => name
  | .enumTy name _ => name
  | .fun args ret => s!"fn({String.intercalate ", " (args.map toString)}) -> {toString ret}"
  | .named n => n

-- Check if type is concrete (not inferred)
def isConcrete (ty : Ty) : Bool :=
  match ty with
  | .primitive .inferred => false
  | .tuple elems => elems.all isConcrete
  | .array elem _ => isConcrete elem
  | .fun args ret => args.all isConcrete && isConcrete ret
  | _ => true

-- Get size of type in bytes (simplified)
def sizeOf (ty : Ty) : Nat :=
  match ty with
  | .primitive p =>
    match p with
    | .unit => 0
    | .i8 | .u8 | .bool | .char => 1
    | .i16 | .u16 => 2
    | .i32 | .u32 | .f32 => 4
    | .i64 | .u64 | .f64 => 8
    | .string => 8  -- Pointer size
    | .inferred => 0
  | .array elem n => sizeOf elem * n
  | .tuple elems => elems.foldl (λ acc t => acc + sizeOf t) 0
  | _ => 8  -- Default to pointer size

-- Check if type can be implicitly converted to another
def canConvertTo (from : Ty) (to : Ty) : Bool :=
  from == to ||
  (isNumeric from && isNumeric to) ||
  to == .primitive .inferred

-- Check if type is numeric
private def isNumeric (ty : Ty) : Bool :=
  match ty with
  | .primitive (.i8 | .i16 | .i32 | .i64 | .u8 | .u16 | .u32 | .u64 | .f32 | .f64) => true
  | _ => false

end Ty

-- Re-exports
export SemanticErrorKind (typeMismatch undefinedVariable undefinedFunction)
export SymbolKind (variable function struct enum)
export ScopeKind (global function block lambda)

end Compiler.Semantic.Types
