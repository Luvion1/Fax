/-
Identifier Validation
-}

import Compiler.Validation.Core

namespace Compiler.Validation.Identifiers

open Compiler.Validation.Core

-- Identifier name validation
def validateIdentifier (name : String) : ValidationResult String :=
  let mut errors := []
  
  -- Check empty
  if name.isEmpty then
    errors := "Identifier cannot be empty" :: errors
  
  -- Check first character (must be letter or underscore)
  if !name.isEmpty then
    let first := name.get 0
    if !first.isAlpha && first != '_' then
      errors := s!"Identifier '{name}' must start with letter or underscore" :: errors
  
  -- Check valid characters (letters, digits, underscore)
  for c in name.toList do
    if !c.isAlphanum && c != '_' then
      errors := s!"Invalid character '{c}' in identifier '{name}'" :: errors
      break
  
  -- Check for reserved keywords
  let reserved := ["fn", "let", "mut", "if", "else", "while", "return", 
                   "struct", "enum", "match", "true", "false", "unit"]
  if reserved.contains name then
    errors := s!"'{name}' is a reserved keyword" :: errors
  
  if errors.isEmpty then
    .ok name
  else
    .errors errors.reverse

-- Module name validation
def validateModuleName (name : String) : ValidationResult String :=
  let mut errors := []
  
  -- Check for valid file name characters
  let invalidChars := ['/', '\\', ':', '*', '?', '"', '<', '>', '|']
  for c in name.toList do
    if invalidChars.contains c then
      errors := s!"Invalid character '{c}' in module name" :: errors
      break
  
  -- Check extension
  if !name.endsWith ".fax" then
    errors := "Module name must end with .fax extension" :: errors
  
  if errors.isEmpty then
    .ok name
  else
    .errors errors.reverse

end Compiler.Validation.Identifiers
