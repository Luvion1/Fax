/-
Converter between AST.Expr and Proto.Expr
-/

import Compiler.AST.Exprs
import Compiler.AST.Stmts
import Compiler.Proto.Converters.Types
import Compiler.Proto.Converters.Pattern

namespace Compiler.Proto.Converters

open AST
open Messages

-- Convert AST.UnaryOp to Proto.UnaryOp
def AST.UnaryOp.toProto : AST.Types.UnaryOp → Messages.UnaryOp
  | .neg => .neg
  | .not => .not
  | .bitnot => .bitnot

-- Convert Proto.UnaryOp back to AST.UnaryOp
def UnaryOp.toAST : Messages.UnaryOp → AST.Types.UnaryOp
  | .neg => .neg
  | .not => .not
  | .bitnot => .bitnot

-- Convert AST.BinOp to Proto.BinOp
def AST.BinOp.toProto : AST.Types.BinOp → Messages.BinOp
  | .add => .add
  | .sub => .sub
  | .mul => .mul
  | .div => .div
  | .mod => .mod
  | .and => .and
  | .or => .or
  | .eq => .eq
  | .ne => .ne
  | .lt => .lt
  | .le => .le
  | .gt => .gt
  | .ge => .ge
  | .shl => .shl
  | .shr => .shr
  | .bitand => .bitand
  | .bitor => .bitor
  | .bitxor => .bitxor

-- Convert Proto.BinOp back to AST.BinOp
def BinOp.toAST : Messages.BinOp → AST.Types.BinOp
  | .add => .add
  | .sub => .sub
  | .mul => .mul
  | .div => .div
  | .mod => .mod
  | .and => .and
  | .or => .or
  | .eq => .eq
  | .ne => .ne
  | .lt => .lt
  | .le => .le
  | .gt => .gt
  | .ge => .ge
  | .shl => .shl
  | .shr => .shr
  | .bitand => .bitand
  | .bitor => .bitor
  | .bitxor => .bitxor

-- Forward declaration for Stmt converter
partial def AST.Stmt.toProto : AST.Stmt → Messages.Stmt
  | .decl mut pat value =>
    .decl mut pat.toProto value.toProto
  | .assign lhs rhs =>
    .assign lhs.toProto rhs.toProto
  | .exprStmt e =>
    .exprStmt e.toProto
  | .return e =>
    .return e.toProto
  | .break => .break
  | .continue => .continue

partial def Stmt.toAST : Messages.Stmt → AST.Stmt
  | .decl mut pat value =>
    .decl mut pat.toAST value.toAST
  | .assign lhs rhs =>
    .assign lhs.toAST rhs.toAST
  | .exprStmt e =>
    .exprStmt e.toAST
  | .return e =>
    .return e.toAST
  | .break => .break
  | .continue => .continue

-- Convert AST.Expr to Proto.Expr (recursive)
partial def AST.Expr.toProto : AST.Expr → Messages.Expr
  | .lit l => .lit l.toProto
  | .var name => .var name
  | .tuple elems => .tuple (elems.map AST.Expr.toProto)
  | .structVal name fields =>
    .structVal name (fields.map (λ (n, e) => (n, e.toProto)))
  | .enumVal enumName variant args =>
    .enumVal enumName variant (args.map AST.Expr.toProto)
  | .proj e idx => .proj e.toProto idx
  | .field e fieldName => .field e.toProto fieldName
  | .unary op e => .unary op.toProto e.toProto
  | .binary op e1 e2 => .binary op.toProto e1.toProto e2.toProto
  | .call fnName args => .call fnName (args.map AST.Expr.toProto)
  | .exprIf cond thenBranch elseBranch =>
    .exprIf cond.toProto thenBranch.toProto elseBranch.toProto
  | .matchExpr scrut cases =>
    .matchExpr scrut.toProto (cases.map (λ (p, e) => (p.toProto, e.toProto)))
  | .block stmts trailing =>
    .block (stmts.map AST.Stmt.toProto) trailing.toProto
  | .lambda params body =>
    .lambda (params.map (λ (n, t) => (n, t.toProto))) body.toProto
  | .letExpr pat value body =>
    .letExpr pat.toProto value.toProto body.toProto

-- Convert Proto.Expr back to AST.Expr (recursive)
partial def Expr.toAST : Messages.Expr → AST.Expr
  | .lit l => .lit l.toAST
  | .var name => .var name
  | .tuple elems => .tuple (elems.map Expr.toAST)
  | .structVal name fields =>
    .structVal name (fields.map (λ (n, e) => (n, e.toAST)))
  | .enumVal enumName variant args =>
    .enumVal enumName variant (args.map Expr.toAST)
  | .proj e idx => .proj e.toAST idx
  | .field e fieldName => .field e.toAST fieldName
  | .unary op e => .unary op.toAST e.toAST
  | .binary op e1 e2 => .binary op.toAST e1.toAST e2.toAST
  | .call fnName args => .call fnName (args.map Expr.toAST)
  | .exprIf cond thenBranch elseBranch =>
    .exprIf cond.toAST thenBranch.toAST elseBranch.toAST
  | .matchExpr scrut cases =>
    .matchExpr scrut.toAST (cases.map (λ (p, e) => (p.toAST, e.toAST)))
  | .block stmts trailing =>
    .block (stmts.map Stmt.toAST) trailing.toAST
  | .lambda params body =>
    .lambda (params.map (λ (n, t) => (n, t.toAST))) body.toAST
  | .letExpr pat value body =>
    .letExpr pat.toAST value.toAST body.toAST

end Compiler.Proto.Converters
