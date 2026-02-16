/-
AST Module - Main Entry Point
Exports all Abstract Syntax Tree components
-/

import Compiler.AST.Types
import Compiler.AST.Types.Basic
import Compiler.AST.Types.Literal
import Compiler.AST.Types.Operators
import Compiler.AST.Types.Utils
import Compiler.AST.Patterns
import Compiler.AST.Exprs
import Compiler.AST.Stmts
import Compiler.AST.Decls

namespace Compiler.AST

-- Re-export all AST components
export Types (Ty PrimitiveTy Lit Pattern Decl Expr Stmt Module)
export Types.Basic (BinOp UnaryOp)
export Types.Literal (intLit floatLit boolLit stringLit charLit)
export Types.Operators (binOpFromString unaryOpFromString)
export Types.Utils (typeToString isPrimitiveTy)
export Patterns (Pattern)
export Exprs (Expr)
export Stmts (Stmt)
export Decls (Decl)

-- Type alias for convenience
alias Pat := Pattern

end Compiler.AST
