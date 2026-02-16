import Compiler.AST

namespace Compiler.Parser

open Compiler.AST
open Lexer.Tokens

partial def parseType (p : Parser) : Parser × Type :=
  match p.peek with
  | .ident "i32" => (p.advance, .int32)
  | .ident "i64" => (p.advance, .int64)
  | .ident "f64" => (p.advance, .float64)
  | .ident "bool" => (p.advance, .boolTy)
  | .ident "char" => (p.advance, .char)
  | .ident "str" => (p.advance, .string)
  | .ident "Unit" => (p.advance, .unit)
  | .ident name =>
    let p := p.advance
    (p, .inferred)
  | .lparen => parseTupleType p
  | _ => (p, .inferred)

partial def parseTupleType (p : Parser) : Parser × Type :=
  let (p, _) := p.expect .lparen
  let (p, elems) := parseTypeList p
  (p, .tuple elems)

partial def parseTypeList (p : Parser) : Parser × List Type :=
  let (p, t) := parseType p
  match p.peek with
  | .comma =>
    let (p, rest) := parseTypeList (p.advance)
    (p, t :: rest)
  | .rparen => (p.advance, [t])
  | _ => (p, [t])

end Compiler.Parser
