/-
Protobuf Converters Module - Main Entry Point

Converts between Lean AST types and Protocol Buffer message types.

## Submodules

- Token: Token and TokenStream conversions
- Types: Type and literal conversions
- Pattern: Pattern matching conversions
- Expr: Expression conversions
- Decl: Declaration conversions
-}

import Compiler.Proto.Converters.Token
import Compiler.Proto.Converters.Types
import Compiler.Proto.Converters.Pattern
import Compiler.Proto.Converters.Expr
import Compiler.Proto.Converters.Decl

namespace Compiler.Proto.Converters

-- Re-export all converters
export Token (tokensToProto tokenStreamToLexer Token.toProto Token.toLexer)
export Types (AST.Ty.toProto Ty.toAST AST.Literal.toProto Literal.toAST)
export Pattern (AST.Pattern.toProto Pattern.toAST)
export Expr (AST.Expr.toProto Expr.toAST AST.Stmt.toProto Stmt.toAST
             AST.UnaryOp.toProto UnaryOp.toAST AST.BinOp.toProto BinOp.toAST)
export Decl (AST.Decl.toProto Decl.toAST AST.Module.toProto Module.toAST)

end Compiler.Proto.Converters
