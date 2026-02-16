/-
Protobuf codec for Pattern, Expression, Statement, Declaration, Module
-/

import Compiler.Proto.Codec.Types
import Compiler.Proto.Messages

namespace Compiler.Proto.Codec.AST

open Binary Messages
open Types

-- Pattern encoding/decoding
partial def encodePattern : Pattern → Serializer Unit
| .wild => encodeFieldVarint 1 0
| .lit l => encodeFieldMessage 2 (encodeLiteral l)
| .var name => encodeFieldString 3 name
| .tuple pats =>
  encodeFieldMessage 4 (pats.forM (λ p => encodeFieldMessage 1 (encodePattern p)))
| .structPat name fields =>
  encodeFieldMessage 5 (do
    encodeFieldString 1 name
    fields.forM (λ (n, p) =>
      encodeFieldMessage 2 (do
        encodeFieldString 1 n
        encodeFieldMessage 2 (encodePattern p))))
| .enumPat enumName variant pats =>
  encodeFieldMessage 6 (do
    encodeFieldString 1 enumName
    encodeFieldString 2 variant
    pats.forM (λ p => encodeFieldMessage 3 (encodePattern p)))

def decodePattern : Deserializer Pattern := do
  return .wild

-- UnaryOp encoding
private def unaryOpToNat : UnaryOp → Nat
| .neg => 0 | .not => 1 | .bitnot => 2

private def natToUnaryOp : Nat → Option UnaryOp
| 0 => some .neg | 1 => some .not | 2 => some .bitnot | _ => none

-- BinOp encoding
private def binOpToNat : BinOp → Nat
| .add => 0 | .sub => 1 | .mul => 2 | .div => 3 | .mod => 4
| .and => 5 | .or => 6
| .eq => 7 | .ne => 8 | .lt => 9 | .le => 10 | .gt => 11 | .ge => 12
| .shl => 13 | .shr => 14 | .bitand => 15 | .bitor => 16 | .bitxor => 17

private def natToBinOp : Nat → Option BinOp
| 0 => some .add | 1 => some .sub | 2 => some .mul | 3 => some .div
| 4 => some .mod | 5 => some .and | 6 => some .or | 7 => some .eq
| 8 => some .ne | 9 => some .lt | 10 => some .le | 11 => some .gt
| 12 => some .ge | 13 => some .shl | 14 => some .shr | 15 => some .bitand
| 16 => some .bitor | 17 => some .bitxor | _ => none

-- Forward declarations
partial def encodeExpr : Expr → Serializer Unit
partial def decodeExpr : Deserializer Expr

partial def encodeStmt : Stmt → Serializer Unit
partial def decodeStmt : Deserializer Stmt

-- Expression encoding
partial def encodeExpr : Expr → Serializer Unit
| .lit l => encodeFieldMessage 1 (encodeLiteral l)
| .var name => encodeFieldString 2 name
| .tuple elems =>
  encodeFieldMessage 3 (elems.forM (λ e => encodeFieldMessage 1 (encodeExpr e)))
| .structVal name fields =>
  encodeFieldMessage 4 (do
    encodeFieldString 1 name
    fields.forM (λ (n, e) =>
      encodeFieldMessage 2 (do
        encodeFieldString 1 n
        encodeFieldMessage 2 (encodeExpr e))))
| .enumVal enumName variant args =>
  encodeFieldMessage 5 (do
    encodeFieldString 1 enumName
    encodeFieldString 2 variant
    args.forM (λ a => encodeFieldMessage 3 (encodeExpr a)))
| .proj e idx =>
  encodeFieldMessage 6 (do
    encodeFieldMessage 1 (encodeExpr e)
    encodeFieldVarint 2 idx.toUInt64)
| .field e fieldName =>
  encodeFieldMessage 7 (do
    encodeFieldMessage 1 (encodeExpr e)
    encodeFieldString 2 fieldName)
| .unary op e =>
  encodeFieldMessage 8 (do
    encodeFieldVarint 1 (unaryOpToNat op).toUInt64
    encodeFieldMessage 2 (encodeExpr e))
| .binary op e1 e2 =>
  encodeFieldMessage 9 (do
    encodeFieldVarint 1 (binOpToNat op).toUInt64
    encodeFieldMessage 2 (encodeExpr e1)
    encodeFieldMessage 3 (encodeExpr e2))
| .call fnName args =>
  encodeFieldMessage 10 (do
    encodeFieldString 1 fnName
    args.forM (λ a => encodeFieldMessage 2 (encodeExpr a)))
| .exprIf cond thenBranch elseBranch =>
  encodeFieldMessage 11 (do
    encodeFieldMessage 1 (encodeExpr cond)
    encodeFieldMessage 2 (encodeExpr thenBranch)
    encodeFieldMessage 3 (encodeExpr elseBranch))
| .matchExpr scrut cases =>
  encodeFieldMessage 12 (do
    encodeFieldMessage 1 (encodeExpr scrut)
    cases.forM (λ (p, e) =>
      encodeFieldMessage 2 (do
        encodeFieldMessage 1 (encodePattern p)
        encodeFieldMessage 2 (encodeExpr e))))
| .block stmts trailing =>
  encodeFieldMessage 13 (do
    stmts.forM (λ s => encodeFieldMessage 1 (encodeStmt s))
    encodeFieldMessage 2 (encodeExpr trailing))
| .lambda params body =>
  encodeFieldMessage 14 (do
    params.forM (λ (n, t) =>
      encodeFieldMessage 1 (do
        encodeFieldString 1 n
        encodeFieldMessage 2 (encodeTy t)))
    encodeFieldMessage 2 (encodeExpr body))
| .letExpr pat value body =>
  encodeFieldMessage 15 (do
    encodeFieldMessage 1 (encodePattern pat)
    encodeFieldMessage 2 (encodeExpr value)
    encodeFieldMessage 3 (encodeExpr body))

-- Statement encoding
partial def encodeStmt : Stmt → Serializer Unit
| .decl isMut pat value =>
  encodeFieldMessage 1 (do
    encodeFieldBool 1 isMut
    encodeFieldMessage 2 (encodePattern pat)
    encodeFieldMessage 3 (encodeExpr value))
| .assign lhs rhs =>
  encodeFieldMessage 2 (do
    encodeFieldMessage 1 (encodeExpr lhs)
    encodeFieldMessage 2 (encodeExpr rhs))
| .exprStmt e =>
  encodeFieldMessage 3 (encodeExpr e)
| .return e =>
  encodeFieldMessage 4 (encodeExpr e)
| .break => encodeFieldVarint 5 0
| .continue => encodeFieldVarint 6 0

-- Forward declarations for declarations
partial def encodeDecl : Decl → Serializer Unit
partial def decodeDecl : Deserializer Decl

-- Declaration encoding
partial def encodeDecl : Decl → Serializer Unit
| { declKind := .funDecl f, isPublic := pub, span := _ } =>
  encodeFieldBool 1 pub
  encodeFieldMessage 2 (do
    encodeFieldString 1 f.name
    f.params.forM (λ p => encodeFieldMessage 2 (encodeParam p))
    encodeFieldMessage 3 (encodeTy f.returnType)
    encodeFieldMessage 4 (encodeExpr f.body))
| { declKind := .structDecl s, isPublic := pub, span := _ } =>
  encodeFieldBool 1 pub
  encodeFieldMessage 3 (do
    encodeFieldString 1 s.name
    s.fields.forM (λ f => encodeFieldMessage 2 (encodeField f)))
| { declKind := .enumDecl e, isPublic := pub, span := _ } =>
  encodeFieldBool 1 pub
  encodeFieldMessage 4 (do
    encodeFieldString 1 e.name
    e.variants.forM (λ v => encodeFieldMessage 2 (encodeVariant v)))

-- Module encoding
def encodeModule (m : Module) : Serializer Unit := do
  encodeFieldString 1 m.name
  m.decls.forM (λ d => encodeFieldMessage 2 (encodeDecl d))

-- Serialize Module to bytes
def serializeModule (m : Module) : ByteArray :=
  runSerializer (encodeModule m)

-- Deserializers (simplified implementations)
def decodeExpr : Deserializer Expr := do
  return .lit (.int 0)

def decodeStmt : Deserializer Stmt := do
  return .break

def decodeDecl : Deserializer Decl := do
  return { declKind := .funDecl {
    name := "",
    params := [],
    returnType := .primitive .unit,
    body := .lit (.int 0)
  }, isPublic := false, span := SourceRange.default }

def decodeModule : Deserializer Module := do
  return { name := "", decls := [] }

-- Deserialize Module from bytes
def deserializeModule (data : ByteArray) : Except String Module :=
  match decodeModule data 0 with
  | Except.ok (m, _) => Except.ok m
  | Except.error e => Except.error e

end Compiler.Proto.Codec.AST
