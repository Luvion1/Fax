/-
Core Validation Types and Operations
-}

namespace Compiler.Validation.Core

-- Validation result type
inductive ValidationResult (α : Type)
  | ok (value : α)
  | errors (msgs : List String)
  deriving Repr

def ValidationResult.isOk {α} (r : ValidationResult α) : Bool :=
  match r with
  | .ok _ => true
  | .errors _ => false

def ValidationResult.isErr {α} (r : ValidationResult α) : Bool :=
  !r.isOk

def ValidationResult.map {α β} (r : ValidationResult α) (f : α → β) : ValidationResult β :=
  match r with
  | .ok v => .ok (f v)
  | .errors msgs => .errors msgs

def ValidationResult.andThen {α β} (r : ValidationResult α) (f : α → ValidationResult β) : ValidationResult β :=
  match r with
  | .ok v => f v
  | .errors msgs => .errors msgs

def ValidationResult.combine {α} (r1 : ValidationResult α) (r2 : ValidationResult α) (f : α → α → α) : ValidationResult α :=
  match r1, r2 with
  | .ok v1, .ok v2 => .ok (f v1 v2)
  | .errors msgs1, .errors msgs2 => .errors (msgs1 ++ msgs2)
  | .errors msgs, _ => .errors msgs
  | _, .errors msgs => .errors msgs

-- Export
export ValidationResult (ok errors isOk isErr map andThen combine)

end Compiler.Validation.Core
