/-
Validation Module - Main Entry Point
Exports all validation components
-/

import Compiler.Validation.Core
import Compiler.Validation.Source
import Compiler.Validation.Identifiers
import Compiler.Validation.Types
import Compiler.Validation.Limits

namespace Compiler.Validation

-- Re-export all validation components
export Core (ValidationResult ok errors isOk isErr map andThen combine)
export Source (validateSourceCode)
export Identifiers (validateIdentifier validateModuleName)
export Types (validateTypeName validateFunctionSignature)
export Limits (validateArraySize validateIntLiteral validateStringLiteral 
               validateHeapConfig validateRecursionDepth)

-- Convenience function to run all validations
def validateAll (source : String) : ValidationResult String :=
  -- Validate source code structure
  match validateSourceCode source with
  | .errors msgs => .errors msgs
  | .ok src => .ok src

end Compiler.Validation
