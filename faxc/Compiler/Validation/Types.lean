/-
Type Validation
-}

import Compiler.Validation.Core
import Compiler.Validation.Identifiers

namespace Compiler.Validation.Types

open Compiler.Validation.Core
open Compiler.Validation.Identifiers

-- Type name validation
def validateTypeName (typeName : String) : ValidationResult String :=
  let validTypes := ["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", 
                     "f32", "f64", "bool", "char", "str", "unit"]
  
  if validTypes.contains typeName then
    .ok typeName
  else if typeName.length > 0 && typeName.get 0 == '_' then
    .errors [s!"Type '{typeName}' is private (starts with underscore)"]
  else
    -- Allow custom types (structs/enums)
    .ok typeName

-- Function signature validation
def validateFunctionSignature (name : String) (paramCount : Nat) : ValidationResult (String × Nat) :=
  let mut errors := []
  
  -- Validate function name
  match validateIdentifier name with
  | .ok _ => pure ()
  | .errors msgs => errors := errors ++ msgs
  
  -- Check parameter count (reasonable limit)
  if paramCount > 100 then
    errors := s!"Function '{name}' has too many parameters ({paramCount}, max 100)" :: errors
  
  if errors.isEmpty then
    .ok (name, paramCount)
  else
    .errors errors

end Compiler.Validation.Types
