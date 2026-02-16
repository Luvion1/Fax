namespace Compiler.AST.Types

inductive Literal where
  | int (val : Int)
  | float (val : Float)
  | bool (val : Bool)
  | string (val : String)
  | char (val : Char)

end Compiler.AST.Types
