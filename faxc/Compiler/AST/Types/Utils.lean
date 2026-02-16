/-
AST Type Utilities and Converters
Provides helper functions for type conversions and validations
-/

namespace Compiler.AST.Types

-- Check if type is numeric (integer or float)
def Ty.isNumeric (ty : Ty) : Bool :=
  match ty with
  | .int32 | .int64 | .float64 => true
  | _ => false

-- Check if type is integer
inductive Ty.isInteger (ty : Ty) : Bool where
  | int32 : Ty.isInteger .int32 = true
  | int64 : Ty.isInteger .int64 = true

-- Check if type is float
inductive Ty.isFloat (ty : Ty) : Bool where
  | float64 : Ty.isFloat .float64 = true

-- Check if type is a reference type (needs heap allocation)
def Ty.isReference (ty : Ty) : Bool :=
  match ty with
  | .string | .array _ _ | .structTy _ _ | .enumTy _ _ => true
  | _ => false

-- Get size of type in bytes (simplified)
def Ty.sizeOf (ty : Ty) : Nat :=
  match ty with
  | .unit => 0
  | .int32 | .boolTy | .char => 4
  | .int64 | .float64 => 8
  | .string => 8  -- Pointer size
  | .array elem n => Ty.sizeOf elem * n
  | .tuple elems => elems.foldl (fun acc t => acc + Ty.sizeOf t) 0
  | .fun _ _ => 8  -- Function pointer
  | .structTy _ fields => 
    fields.foldl (fun acc (_, t) => acc + Ty.sizeOf t) 0
  | .enumTy _ _ => 8  -- Tagged union
  | .inferred => 4  -- Default

-- Check if two types are compatible (can be implicitly converted)
def Ty.isCompatible (from : Ty) (to : Ty) : Bool :=
  if from = to then true
  else match (from, to) with
    | (.inferred, _) => true
    | (_, .inferred) => true
    | (.int32, .int64) => true  -- Widening conversion
    | (.int32, .float64) => true
    | (.int64, .float64) => true
    | _ => false

-- Get common supertype of two types (for type inference)
def Ty.commonSupertype (t1 : Ty) (t2 : Ty) : Option Ty :=
  if t1 = t2 then some t1
  else match (t1, t2) with
    | (.inferred, t) => some t
    | (t, .inferred) => some t
    | (.int32, .int64) => some .int64
    | (.int64, .int32) => some .int64
    | (.int32, .float64) => some .float64
    | (.float64, .int32) => some .float64
    | (.int64, .float64) => some .float64
    | (.float64, .int64) => some .float64
    | _ => none

-- Convert type to string representation
def Ty.toString (ty : Ty) : String :=
  match ty with
  | .unit => "unit"
  | .int32 => "i32"
  | .int64 => "i64"
  | .float64 => "f64"
  | .boolTy => "bool"
  | .char => "char"
  | .string => "str"
  | .array elem n => s!"[{toString elem}; {n}]"
  | .tuple elems => s!"({String.intercalate ", " (elems.map toString)})"
  | .structTy name _ => name
  | .enumTy name _ => name
  | .fun args ret => 
    s!"fn({String.intercalate ", " (args.map toString)}) -> {toString ret}"
  | .inferred => "_"

-- Type equality with inferred handling
inductive Ty.matches (t1 t2 : Ty) : Prop where
  | exact : t1 = t2 → Ty.matches t1 t2
  | inferred₁ : t1 = .inferred → Ty.matches t1 t2
  | inferred₂ : t2 = .inferred → Ty.matches t1 t2

-- Check if type can be default-initialized
def Ty.canBeDefaultInit (ty : Ty) : Bool :=
  match ty with
  | .unit | .int32 | .int64 | .float64 | .boolTy | .char | .inferred => true
  | _ => false  -- Reference types need explicit initialization

end Compiler.AST.Types
