/-
Limits and Constraints Validation
-}

import Compiler.Validation.Core

namespace Compiler.Validation.Limits

open Compiler.Validation.Core

-- Array size validation
def validateArraySize (size : Nat) : ValidationResult Nat :=
  if size == 0 then
    .errors ["Array size cannot be zero"]
  else if size > 1000000 then
    .errors [s!"Array size {size} exceeds maximum (1,000,000)"]
  else
    .ok size

-- Integer literal validation
def validateIntLiteral (n : Int) : ValidationResult Int :=
  let minInt := -2147483648
  let maxInt := 2147483647
  
  if n < minInt then
    .errors [s!"Integer literal {n} is below minimum i32 value ({minInt})"]
  else if n > maxInt then
    .errors [s!"Integer literal {n} exceeds maximum i32 value ({maxInt})"]
  else
    .ok n

-- String literal validation
def validateStringLiteral (s : String) : ValidationResult String :=
  let mut errors := []
  let maxLength := 10000
  
  if s.length > maxLength then
    errors := s!"String literal exceeds maximum length ({maxLength} characters)" :: errors
  
  -- Check for null bytes (can cause issues in C interop)
  if s.toList.contains '\x00' then
    errors := "String literal contains null byte" :: errors
  
  if errors.isEmpty then
    .ok s
  else
    .errors errors.reverse

-- Configuration validation
def validateHeapConfig (minSize maxSize : Nat) : ValidationResult (Nat × Nat) :=
  let mut errors := []
  
  if minSize == 0 then
    errors := "Minimum heap size cannot be zero" :: errors
  
  if maxSize == 0 then
    errors := "Maximum heap size cannot be zero" :: errors
  
  if minSize > maxSize then
    errors := "Minimum heap size cannot be greater than maximum" :: errors
  
  let maxAllowed := 16 * 1024 * 1024 * 1024  -- 16GB
  if maxSize > maxAllowed then
    errors := s!"Maximum heap size exceeds limit ({maxAllowed} bytes)" :: errors
  
  if errors.isEmpty then
    .ok (minSize, maxSize)
  else
    .errors errors.reverse

-- Recursion depth validation (for recursive functions)
def validateRecursionDepth (depth : Nat) : ValidationResult Nat :=
  let maxDepth := 1000
  if depth > maxDepth then
    .errors [s!"Recursion depth {depth} exceeds maximum ({maxDepth})"]
  else
    .ok depth

end Compiler.Validation.Limits
