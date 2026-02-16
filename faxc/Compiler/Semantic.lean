/-
Semantic Analysis Module - Main Entry Point
Exports all semantic analysis components

Note: For protobuf support, import both Compiler.Semantic and Compiler.Proto separately.
-/

import Compiler.AST
import Compiler.AST.Types
import Compiler.Semantic.Types
import Compiler.Semantic.Scope
import Compiler.Semantic.Inference
import Compiler.Semantic.Errors
import Compiler.Semantic.Checker

namespace Compiler.Semantic

-- Re-export all semantic analysis components
export Types (Symbol SymbolKind SymbolTable TypeInfo TyScheme)
export Scope (Scope)
export Inference (inferType unify)
export Errors (SemanticError SemanticErrorKind)
export Checker (TypeChecker typeCheckModule checkExpr checkDecl)

-- Main semantic analysis result
structure SemanticResult where
  errors : List SemanticError
  symbolTable : SymbolTable
  typeInfo : TypeInfo
  inferredTypes : List (String × Ty)
  deriving Repr

def SemanticResult.isValid (r : SemanticResult) : Bool :=
  r.errors.isEmpty

end Compiler.Semantic
