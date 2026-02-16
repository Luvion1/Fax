namespace Compiler.AST.Types

inductive Ty where
  | unit
  | int32 | int64 | float64 | boolTy | char | string
  | array (elem : Ty) (size : Nat)
  | tuple (elems : List Ty)
  | structTy (name : String) (fields : List (String × Ty))
  | enumTy (name : String) (variants : List (String × List Ty))
  | fun (args : List Ty) (ret : Ty)
  | inferred

end Compiler.AST.Types
