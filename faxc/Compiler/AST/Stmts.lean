import Compiler.AST.Patterns
import Compiler.AST.Exprs

namespace Compiler.AST

inductive Stmt where
  | decl (mut : Bool) (pat : Pattern) (value : Expr)
  | assign (lhs : Expr) (rhs : Expr)
  | exprStmt (e : Expr)
  | return (e : Expr)
  | break
  | continue

end Compiler.AST
