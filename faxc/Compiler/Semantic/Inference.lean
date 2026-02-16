/-
Semantic Analysis - Type Inference
-/

import Compiler.Semantic.Types

namespace Compiler.Semantic.Inference

open Compiler.Semantic.Types

-- Type constraint for inference
inductive TypeConstraint
  | equal (t1 : Ty) (t2 : Ty)
  | subtype (t1 : Ty) (t2 : Ty)
  | numeric (t : Ty)
  | integer (t : Ty)
  deriving Repr

-- Type substitution (mapping from type variable to type)
def TypeSubst := List (String × Ty)

def TypeSubst.empty : TypeSubst := []

def TypeSubst.lookup (subst : TypeSubst) (var : String) : Option Ty :=
  subst.find? (λ (v, _) => v == var) |>.map (·.2)

def TypeSubst.add (subst : TypeSubst) (var : String) (ty : Ty) : TypeSubst :=
  (var, ty) :: subst

-- Apply substitution to type
def applySubst (subst : TypeSubst) (ty : Ty) : Ty :=
  match ty with
  | .named n =>
    match subst.lookup n with
    | some t => t
    | none => ty
  | .tuple elems => .tuple (elems.map (applySubst subst))
  | .array elem size => .array (applySubst subst elem) size
  | .fun args ret => .fun (args.map (applySubst subst)) (applySubst subst ret)
  | _ => ty

-- Unify two types and produce substitution
def unify (t1 : Ty) (t2 : Ty) : Except String TypeSubst :=
  match t1, t2 with
  | .primitive p1, .primitive p2 =>
    if p1 == p2 then Except.ok []
    else Except.error s!"Cannot unify {repr p1} with {repr p2}"
  
  | .named n, ty => Except.ok [(n, ty)]
  | ty, .named n => Except.ok [(n, ty)]
  
  | .tuple es1, .tuple es2 =>
    if es1.length != es2.length then
      Except.error "Tuple length mismatch"
    else
      unifyMany es1 es2
  
  | .array e1 s1, .array e2 s2 =>
    if s1 != s2 then
      Except.error "Array size mismatch"
    else
      unify e1 e2
  
  | .fun as1 r1, .fun as2 r2 =>
    if as1.length != as2.length then
      Except.error "Function arity mismatch"
    else
      match unifyMany as1 as2 with
      | Except.ok subst1 =>
        match unify (applySubst subst1 r1) (applySubst subst1 r2) with
        | Except.ok subst2 => Except.ok (subst1 ++ subst2)
        | Except.error e => Except.error e
      | Except.error e => Except.error e
  
  | _, _ => Except.error s!"Cannot unify {repr t1} with {repr t2}"

-- Unify multiple type pairs
def unifyMany (ts1 : List Ty) (ts2 : List Ty) : Except String TypeSubst :=
  let rec go (s1 s2 : List Ty) (acc : TypeSubst) : Except String TypeSubst :=
    match s1, s2 with
    | [], [] => Except.ok acc
    | t1 :: rest1, t2 :: rest2 =>
      match unify (applySubst acc t1) (applySubst acc t2) with
      | Except.ok subst => go rest1 rest2 (acc ++ subst)
      | Except.error e => Except.error e
    | _, _ => Except.error "List length mismatch"
  go ts1 ts2 []

-- Most general unifier (MGU)
def mostGeneralUnifier (constraints : List TypeConstraint) : Except String TypeSubst :=
  let rec solve (cs : List TypeConstraint) (subst : TypeSubst) : Except String TypeSubst :=
    match cs with
    | [] => Except.ok subst
    | .equal t1 t2 :: rest =>
      match unify (applySubst subst t1) (applySubst subst t2) with
      | Except.ok newSubst => solve rest (subst ++ newSubst)
      | Except.error e => Except.error e
    | .subtype t1 t2 :: rest =>
      -- For now, treat subtype as equality
      match unify (applySubst subst t1) (applySubst subst t2) with
      | Except.ok newSubst => solve rest (subst ++ newSubst)
      | Except.error e => Except.error e
    | .numeric t :: rest =>
      -- Ensure type is numeric
      match t with
      | .primitive (.i8 | .i16 | .i32 | .i64 | .u8 | .u16 | .u32 | .u64 | .f32 | .f64) =>
        solve rest subst
      | _ => Except.error s!"Expected numeric type, got {repr t}"
    | .integer t :: rest =>
      -- Ensure type is integer
      match t with
      | .primitive (.i8 | .i16 | .i32 | .i64 | .u8 | .u16 | .u32 | .u64) =>
        solve rest subst
      | _ => Except.error s!"Expected integer type, got {repr t}"
  solve constraints []

-- Infer type of binary operation with constraint solving
def inferBinaryOp (op : BinaryOp) (t1 : Ty) (t2 : Ty) 
    : Except String Ty :=
  match op with
  | .add | .sub | .mul | .div | .mod =>
    let constraints := [.numeric t1, .numeric t2, .equal t1 t2]
    match mostGeneralUnifier constraints with
    | Except.ok _ => Except.ok t1
    | Except.error e => Except.error e
  
  | .and | .or =>
    let constraints := [.equal t1 (.primitive .bool), .equal t2 (.primitive .bool)]
    match mostGeneralUnifier constraints with
    | Except.ok _ => Except.ok (.primitive .bool)
    | Except.error e => Except.error e
  
  | .eq | .ne =>
    let constraints := [.equal t1 t2]
    match mostGeneralUnifier constraints with
    | Except.ok _ => Except.ok (.primitive .bool)
    | Except.error e => Except.error e
  
  | .lt | .le | .gt | .ge =>
    let constraints := [.numeric t1, .numeric t2, .equal t1 t2]
    match mostGeneralUnifier constraints with
    | Except.ok _ => Except.ok (.primitive .bool)
    | Except.error e => Except.error e
  
  | .shl | .shr | .band | .bor | .bxor =>
    let constraints := [.integer t1, .integer t2, .equal t1 t2]
    match mostGeneralUnifier constraints with
    | Except.ok _ => Except.ok t1
    | Except.error e => Except.error e

-- Type variable generator for inference
def TypeVarGen := Nat

def TypeVarGen.new : TypeVarGen := 0

def TypeVarGen.fresh (gen : TypeVarGen) : (Ty × TypeVarGen) :=
  let varName := s!"_t{gen}"
  (.named varName, gen + 1)

-- Hindley-Milner style type inference (simplified)
inductive MonoType
  | var (name : String)
  | con (name : String)
  | arr (arg : MonoType) (ret : MonoType)
  | tuple (elems : List MonoType)
  deriving Repr, BEq

def MonoType.toTy (mt : MonoType) : Ty :=
  match mt with
  | .var n => .named n
  | .con "i32" => .primitive .i32
  | .con "i64" => .primitive .i64
  | .con "f32" => .primitive .f32
  | .con "f64" => .primitive .f64
  | .con "bool" => .primitive .bool
  | .con "str" => .primitive .string
  | .con "unit" => .primitive .unit
  | .con n => .named n
  | .arr a r => .fun [a.toTy] r.toTy
  | .tuple elems => .tuple (elems.map toTy)

-- Type scheme (polymorphic type)
structure TypeScheme where
  vars : List String
  body : MonoType
  deriving Repr

-- Free type variables
def freeVars (mt : MonoType) : List String :=
  match mt with
  | .var n => [n]
  | .con _ => []
  | .arr a r => freeVars a ++ freeVars r
  | .tuple elems => elems.foldr (λ e acc => freeVars e ++ acc) []

def freeVarsScheme (scheme : TypeScheme) : List String :=
  freeVars scheme.body |>.filter (λ v => !scheme.vars.contains v)

-- Generalize type to scheme
def generalize (ty : MonoType) (freeInEnv : List String) : TypeScheme :=
  let vars := freeVars ty |>.filter (λ v => !freeInEnv.contains v)
  { vars := vars, body := ty }

-- Instantiate scheme to type
def instantiate (scheme : TypeScheme) (gen : TypeVarGen) : (MonoType × TypeVarGen) :=
  let subst := scheme.vars.map (λ v => 
    let (fresh, _) := TypeVarGen.fresh gen
    (v, fresh)
  )
  (applySubstMono subst scheme.body, gen + scheme.vars.length)

private def applySubstMono (subst : List (String × Ty)) (mt : MonoType) : MonoType :=
  match mt with
  | .var n =>
    match subst.find? (λ (v, _) => v == n) with
    | some ty => 
      match ty with
      | .named n => .var n
      | .primitive p => .con (repr p)
      | _ => mt
    | none => mt
  | .con n => .con n
  | .arr a r => .arr (applySubstMono subst a) (applySubstMono subst r)
  | .tuple elems => .tuple (elems.map (applySubstMono subst))

end Compiler.Semantic.Inference
