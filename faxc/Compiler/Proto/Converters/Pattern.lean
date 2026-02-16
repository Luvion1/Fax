/-
Converter between AST.Pattern and Proto.Pattern
-/

import Compiler.AST.Patterns
import Compiler.Proto.Converters.Types

namespace Compiler.Proto.Converters

open AST
open Messages

-- Convert AST.Pattern to Proto.Pattern
partial def AST.Pattern.toProto : AST.Pattern → Messages.Pattern
  | .wild => .wild
  | .lit l => .lit l.toProto
  | .var name => .var name
  | .tuple pats => .tuple (pats.map AST.Pattern.toProto)
  | .structPat name fields =>
    .structPat name (fields.map (λ (n, p) => (n, p.toProto)))
  | .enumPat enumName variant pats =>
    .enumPat enumName variant (pats.map AST.Pattern.toProto)

-- Convert Proto.Pattern back to AST.Pattern
partial def Pattern.toAST : Messages.Pattern → AST.Pattern
  | .wild => .wild
  | .lit l => .lit l.toAST
  | .var name => .var name
  | .tuple pats => .tuple (pats.map Pattern.toAST)
  | .structPat name fields =>
    .structPat name (fields.map (λ (n, p) => (n, p.toAST)))
  | .enumPat enumName variant pats =>
    .enumPat enumName variant (pats.map Pattern.toAST)

end Compiler.Proto.Converters
