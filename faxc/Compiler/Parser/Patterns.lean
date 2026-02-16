import Compiler.Parser.Types

namespace Compiler.Parser

open Compiler.AST
open Lexer.Tokens

partial def parsePattern (p : Parser) : Parser × Pattern :=
  match p.peek with
  | .ident name => (p.advance, .var name)
  | .underscore => (p.advance, .wild)
  | .lparen => parseTuplePat p
  | _ => (p, .wild)

partial def parseTuplePat (p : Parser) : Parser × Pattern :=
  let (p, _) := p.expect .lparen
  let (p, pats) := parsePatList p
  (p, .tuple pats)

partial def parsePatList (p : Parser) : Parser × List Pattern :=
  let (p, pat) := parsePattern p
  match p.peek with
  | .comma =>
    let (p, rest) := parsePatList (p.advance)
    (p, pat :: rest)
  | .rparen => (p.advance, [pat])
  | _ => (p, [pat])

end Compiler.Parser
