import Compiler.AST.Types

namespace Compiler.AST

inductive Pattern where
  | wild
  | lit (l : Lit)
  | var (name : String)
  | tuple (pats : List Pattern)
  | structPat (name : String) (fields : List (String × Pattern))
  | enumPat (name : String) (variant : String) (pats : List Pattern)

end Compiler.AST
