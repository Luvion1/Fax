namespace Compiler.AST.Types

inductive UnaryOp where
  | neg
  | not
  | bitnot

inductive BinaryOp where
  | add | sub | mul | div | mod
  | and | or
  | eq | ne | lt | le | gt | ge
  | shl | shr | band | bor | bxor

end Compiler.AST.Types
