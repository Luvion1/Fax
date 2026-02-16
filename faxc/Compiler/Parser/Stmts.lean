import Compiler.Parser.Exprs

namespace Compiler.Parser

open Compiler.AST
open Lexer.Tokens

partial def parseBlock (p : Parser) : Parser × Expr :=
  let (p, _) := p.expect .lbrace
  let (p, stmts) := parseStmts p
  let (p, _) := p.expect .rbrace
  (p, Expr.block stmts (Expr.lit (Literal.int 0)))

partial def parseStmts (p : Parser) : Parser × List Stmt :=
  match p.peek with
  | .rbrace => (p, [])
  | _ =>
    match parseStmt p with
    | (p, some s) =>
      let (p, rest) := parseStmts p
      (p, s :: rest)
    | (p, none) => (p, [])

partial def parseLet (p : Parser) : Parser × Option Stmt :=
  let (p, _) := p.expect .kw_let
  let (p, pat) := parsePattern p
  let (p, _) := p.expect (.op Tokens.OperatorToken.assign)
  let (p, val) := parseExpr p
  (p, some (Stmt.decl false pat val))

partial def parseStmt (p : Parser) : Parser × Option Stmt :=
  match p.peek with
  | .kw_let => parseLet p
  | .kw_return =>
    let p := p.advance
    let (p, e) := parseExpr p
    (p, some (Stmt.return e))
  | .kw_break => (p.advance, some .break)
  | .kw_continue => (p.advance, some .continue)
  | _ =>
    let (p, e) := parseExpr p
    (p, some (Stmt.exprStmt e))

partial def parseLambda (p : Parser) : Parser × Expr :=
  let (p, _) := p.expect .kw_fn
  let (p, params) := parseFunParams p
  let (p, _) := p.expect .arrow
  let (p, body) := parseExpr p
  (p, Expr.lambda params body)

end Compiler.Parser
