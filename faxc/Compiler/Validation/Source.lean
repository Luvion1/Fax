/-
Source Code Validation
-}

import Compiler.Validation.Core

namespace Compiler.Validation.Source

open Compiler.Validation.Core

-- Source code validation
def validateSourceCode (source : String) : ValidationResult String :=
  let mut errors := []
  
  -- Check for empty source
  if source.trim.isEmpty then
    errors := "Source code is empty" :: errors
  
  -- Check for non-ASCII characters (basic check)
  let nonAscii := source.toList.find? (fun c => c.toNat > 127)
  if nonAscii.isSome then
    errors := s!"Non-ASCII character found: '{nonAscii.getD '?'}'" :: errors
  
  -- Check for unmatched braces/brackets
  let openParens := source.toList.count '('
  let closeParens := source.toList.count ')'
  if openParens != closeParens then
    errors := s!"Unmatched parentheses: {openParens} opening, {closeParens} closing" :: errors
  
  let openBraces := source.toList.count '{'
  let closeBraces := source.toList.count '}'
  if openBraces != closeBraces then
    errors := s!"Unmatched braces: {openBraces} opening, {closeBraces} closing" :: errors
  
  let openBrackets := source.toList.count '['
  let closeBrackets := source.toList.count ']'
  if openBrackets != closeBrackets then
    errors := s!"Unmatched brackets: {openBrackets} opening, {closeBrackets} closing" :: errors
  
  -- Check for valid string literals
  let quotes := source.toList.count '"'
  if quotes % 2 != 0 then
    errors := "Unmatched double quotes in string literals" :: errors
  
  if errors.isEmpty then
    .ok source
  else
    .errors errors.reverse

end Compiler.Validation.Source
