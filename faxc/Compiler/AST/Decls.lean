import Compiler.AST.Types
import Compiler.AST.Exprs

namespace Compiler.AST

inductive Decl where
  | funDecl (pub : Bool) (name : String) (params : List (String × Ty)) (ret : Ty) (body : Expr)
  | structDecl (pub : Bool) (name : String) (fields : List (String × Ty))
  | enumDecl (pub : Bool) (name : String) (variants : List (String × List Ty))

inductive Module where
  | mk (decls : List Decl)

end Compiler.AST
