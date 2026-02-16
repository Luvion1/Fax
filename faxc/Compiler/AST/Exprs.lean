import Compiler.AST.Types
import Compiler.AST.Patterns
import Compiler.AST.Stmts

namespace Compiler.AST

inductive Expr where
  | lit (l : Lit)
  | var (name : String)
  | tuple (elems : List Expr)
  | structVal (name : String) (fields : List (String × Expr))
  | enumVal (name : String) (variant : String) (args : List Expr)
  | proj (e : Expr) (idx : Nat)
  | field (e : Expr) (field : String)
  | unary (op : UnOp) (e : Expr)
  | binary (op : BinOp) (e1 e2 : Expr)
  | call (fn : String) (args : List Expr)
  | exprIf (cond : Expr) (then : Expr) (else : Expr)
  | matchExpr (scrut : Expr) (cases : List (Pattern × Expr))
  | block (stmts : List Stmt) (expr : Expr)
  | lambda (params : List (String × Ty)) (body : Expr)
  | letExpr (pat : Pattern) (value : Expr) (body : Expr)

end Compiler.AST
