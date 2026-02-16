/-
Parser Module - Main Entry Point
Exports all parser components
-/

import Compiler.Lexer
import Compiler.AST
import Compiler.Parser.Types
import Compiler.Parser.Patterns
import Compiler.Parser.Exprs
import Compiler.Parser.Stmts
import Compiler.Parser.Decls
import Compiler.Parser.Proto

namespace Compiler.Parser

-- Re-export all parser components
export Types (Parser ParserResult ParseError)
export Patterns (parsePattern)
export Exprs (parseExpr)
export Stmts (parseStmt)
export Decls (parseDecl parseModule)

end Compiler.Parser
