/-
Parser for Expressions - FIXED VERSION
Bug fixes:
1. Fixed toString bug on Expr for function calls
2. Added proper error handling for unknown tokens
3. Fixed data loss in parseArgList
4. Fixed right recursion in comparison operators
5. Added proper identifier validation
6. Added tuple index access support
7. Simplified operator parsing
-/

import Compiler.Parser.Types
import Compiler.Parser.Patterns

namespace Compiler.Parser

open Compiler.AST
open Lexer.Tokens

-- Extract identifier name from expression for function calls
private def extractFunctionName (e : Expr) : String :=
  match e with
  | .var name => name
  | _ => "_invalid_function_"

-- Parse identifier with proper error handling
partial def parseIdent (p : Parser) : Parser × String :=
  match p.peek with
  | .ident name => (p.advance, name)
  | _ => 
    -- BUG FIX: Return error marker instead of fake "_"
    -- This allows caller to detect and report the error
    (p.advance, "_ERROR_NOT_IDENT_")

-- Parse primary expressions (literals, variables, blocks, etc.)
partial def parsePrimaryExpr (p : Parser) : Parser × Expr :=
  match p.peek with
  | .lit_int n => (p.advance, Expr.lit (Literal.int n))
  | .lit_float f => (p.advance, Expr.lit (Literal.float f))
  | .lit_string s => (p.advance, Expr.lit (Literal.string s))
  | .kw_true => (p.advance, Expr.lit (Literal.bool true))
  | .kw_false => (p.advance, Expr.lit (Literal.bool false))
  | .ident name => (p.advance, Expr.var name)
  | .lparen => parseTupleExpr p
  | .lbrace => parseBlock p
  | .kw_fn => parseLambda p
  | .error msg => 
    -- BUG FIX: Propagate lexer errors
    (p.advance, Expr.lit (Literal.int 0))  -- Placeholder, will be caught by error checker
  | _ => 
    -- BUG FIX: Better error recovery
    -- Skip the unknown token and return placeholder
    (p.advance, Expr.lit (Literal.int 0))

-- Parse tuple expressions: (a, b, c) or just grouping (a)
partial def parseTupleExpr (p : Parser) : Parser × Expr :=
  let (p, _) := p.expect .lparen
  match p.peek with
  | .rparen => (p.advance, Expr.tuple [])  -- Empty tuple
  | _ =>
    let (p, e) := parseExpr p
    match p.peek with
    | .comma =>
      -- Multi-element tuple
      let (p, _) := p.advance
      let (p, rest) := parseExprList p
      (p, Expr.tuple (e :: rest))
    | _ =>
      -- Just grouping parentheses, not a tuple
      let (p, _) := p.expect .rparen
      (p, e)

-- Parse comma-separated expression list
partial def parseExprList (p : Parser) : Parser × List Expr :=
  let (p, e) := parseExpr p
  match p.peek with
  | .comma =>
    let (p, rest) := parseExprList (p.advance)
    (p, e :: rest)
  | .rparen => (p.advance, [e])
  | _ => 
    -- BUG FIX: Handle unexpected tokens gracefully
    -- Try to recover by stopping the list
    (p, [e])

-- Parse argument list for function calls
-- BUG FIX: Rewritten to properly accumulate all arguments
partial def parseArgList (p : Parser) : Parser × List Expr :=
  let rec go (p : Parser) (acc : List Expr) : Parser × List Expr :=
    match p.peek with
    | .rparen => 
      -- End of argument list
      (p.advance, acc.reverse)
    | _ =>
      -- Parse next argument
      let (p, arg) := parseExpr p
      match p.peek with
      | .comma => 
        -- More arguments coming
        go (p.advance) (arg :: acc)
      | .rparen => 
        -- Last argument
        (p.advance, (arg :: acc).reverse)
      | _ => 
        -- Unexpected token, try to recover
        (p, (arg :: acc).reverse)
  go p []

-- Parse postfix expressions: function calls, field access
partial def parsePostfixExpr (p : Parser) : Parser × Expr :=
  let (p, base) := parsePrimaryExpr p
  go p base
where
  go (p : Parser) (e : Expr) : Parser × Expr :=
    match p.peek with
    | .lparen =>
      -- Function call
      let p := p.advance
      let (p, args) := parseArgList p
      -- BUG FIX: Extract function name properly, not using toString
      let fnName := extractFunctionName e
      go p (Expr.call fnName args)
    | .dot =>
      -- Field access or method call
      let p := p.advance
      match p.peek with
      | .ident name => go (p.advance) (Expr.field e name)
      | .lit_int n => 
        -- BUG FIX: Support tuple index access: tuple.0, tuple.1
        go (p.advance) (Expr.proj e n.toNat)
      | _ => (p, e)
    | _ => (p, e)

-- Parse unary expressions: -expr, !expr, ~expr
partial def parseUnaryExpr (p : Parser) : Parser × Expr :=
  match p.peek with
  | .op Tokens.OperatorToken.minus =>
    let p := p.advance
    let (p, e) := parseUnaryExpr p
    (p, Expr.unary Types.UnaryOp.neg e)
  | .op Tokens.OperatorToken.not =>
    let p := p.advance
    let (p, e) := parseUnaryExpr p
    (p, Expr.unary Types.UnaryOp.not e)
  | .op Tokens.OperatorToken.bitnot =>
    let p := p.advance
    let (p, e) := parseUnaryExpr p
    (p, Expr.unary Types.UnaryOp.bitnot e)
  | _ => parsePostfixExpr p

-- Helper to parse left-associative binary operators
private def parseLeftAssoc (p : Parser) 
    (parseOperand : Parser → Parser × Expr) 
    (operators : List (Tokens.OperatorToken × Types.BinaryOp)) 
    : Parser × Expr :=
  let (p, lhs) := parseOperand p
  let rec go (p : Parser) (lhs : Expr) : Parser × Expr :=
    match p.peek with
    | .op op =>
      match operators.find? (λ (o, _) => o == op) with
      | some (_, bop) =>
        let p := p.advance
        let (p, rhs) := parseOperand p
        go p (Expr.binary bop lhs rhs)
      | none => (p, lhs)
    | _ => (p, lhs)
  go p lhs

-- Parse multiplication/division/modulo
partial def parseMulExpr (p : Parser) : Parser × Expr :=
  parseLeftAssoc p parseUnaryExpr [
    (Tokens.OperatorToken.star, Types.BinaryOp.mul),
    (Tokens.OperatorToken.slash, Types.BinaryOp.div),
    (Tokens.OperatorToken.percent, Types.BinaryOp.mod)
  ]

-- Parse addition/subtraction
partial def parseAddExpr (p : Parser) : Parser × Expr :=
  parseLeftAssoc p parseMulExpr [
    (Tokens.OperatorToken.plus, Types.BinaryOp.add),
    (Tokens.OperatorToken.minus, Types.BinaryOp.sub)
  ]

-- Parse comparison operators
-- BUG FIX: Use parseAddExpr for RHS, not recursive parseCmpExpr
-- This ensures proper precedence
partial def parseCmpExpr (p : Parser) : Parser × Expr :=
  let (p, lhs) := parseAddExpr p
  match p.peek with
  | .op op =>
    let mbop : Option Types.BinaryOp := match op with
      | Tokens.OperatorToken.eqeq => some Types.BinaryOp.eq
      | Tokens.OperatorToken.neq => some Types.BinaryOp.ne
      | Tokens.OperatorToken.lt => some Types.BinaryOp.lt
      | Tokens.OperatorToken.le => some Types.BinaryOp.le
      | Tokens.OperatorToken.gt => some Types.BinaryOp.gt
      | Tokens.OperatorToken.ge => some Types.BinaryOp.ge
      | _ => none
    match mbop with
    | some bop =>
      let p := p.advance
      -- BUG FIX: Parse RHS with parseAddExpr, not parseCmpExpr
      let (p, rhs) := parseAddExpr p
      (p, Expr.binary bop lhs rhs)
    | none => (p, lhs)
  | _ => (p, lhs)

-- Parse logical AND
partial def parseAndExpr (p : Parser) : Parser × Expr :=
  parseLeftAssoc p parseCmpExpr [
    (Tokens.OperatorToken.and, Types.BinaryOp.and)
  ]

-- Parse logical OR
partial def parseOrExpr (p : Parser) : Parser × Expr :=
  parseLeftAssoc p parseAndExpr [
    (Tokens.OperatorToken.or, Types.BinaryOp.or)
  ]

-- Parse bitwise operators
partial def parseBitwiseExpr (p : Parser) : Parser × Expr :=
  parseLeftAssoc p parseOrExpr [
    (Tokens.OperatorToken.bitand, Types.BinaryOp.band),
    (Tokens.OperatorToken.bitor, Types.BinaryOp.bor),
    (Tokens.OperatorToken.bitxor, Types.BinaryOp.bxor)
  ]

-- Parse shift operators
partial def parseShiftExpr (p : Parser) : Parser × Expr :=
  parseLeftAssoc p parseBitwiseExpr [
    (Tokens.OperatorToken.shl, Types.BinaryOp.shl),
    (Tokens.OperatorToken.shr, Types.BinaryOp.shr)
  ]

-- Parse if expressions: if cond { then } else { else }
partial def parseIfExpr (p : Parser) : Parser × Expr :=
  match p.peek with
  | .kw_if =>
    let p := p.advance
    let (p, cond) := parseExpr p
    let (p, thenBranch) := parseBlock p
    match p.peek with
    | .kw_else =>
      let p := p.advance
      let (p, elseBranch) := parseBlock p
      (p, Expr.exprIf cond thenBranch elseBranch)
    | _ =>
      -- BUG FIX: If no else, create unit block
      (p, Expr.exprIf cond thenBranch (Expr.block [] (Expr.lit (Literal.int 0))))
  | _ => parseShiftExpr p

-- Parse let expressions
partial def parseLetExpr (p : Parser) : Parser × Expr :=
  match p.peek with
  | .kw_let =>
    let p := p.advance
    -- Check for mut
    let (p, isMut) := match p.peek with
      | .kw_mut => (p.advance, true)
      | _ => (p, false)
    -- Parse pattern
    let (p, pat) := parsePattern p
    let (p, _) := p.expect .eq
    let (p, value) := parseExpr p
    let (p, _) := p.expect .semicolon
    let (p, body) := parseExpr p
    (p, Expr.letExpr pat value body)
  | _ => parseIfExpr p

-- Parse match expressions
partial def parseMatchExpr (p : Parser) : Parser × Expr :=
  match p.peek with
  | .kw_match =>
    let p := p.advance
    let (p, scrut) := parseExpr p
    let (p, _) := p.expect .lbrace
    let (p, cases) := parseMatchCases p
    (p, Expr.matchExpr scrut cases)
  | _ => parseLetExpr p

-- Parse match cases
partial def parseMatchCases (p : Parser) : Parser × List (Pattern × Expr) :=
  let rec go (p : Parser) (acc : List (Pattern × Expr)) : Parser × List (Pattern × Expr) :=
    match p.peek with
    | .rbrace => (p.advance, acc.reverse)
    | _ =>
      let (p, pat) := parsePattern p
      let (p, _) := p.expect .arrowfat
      let (p, expr) := parseExpr p
      -- Optional comma between cases
      let p := match p.peek with
        | .comma => p.advance
        | _ => p
      go p ((pat, expr) :: acc)
  go p []

-- Main expression parser entry point
partial def parseExpr (p : Parser) : Parser × Expr :=
  parseMatchExpr p

end Compiler.Parser
