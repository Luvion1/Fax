/-
Converter between AST.Decl/Module and Proto.Decl/Module
-/

import Compiler.AST.Decls
import Compiler.Proto.Converters.Types
import Compiler.Proto.Converters.Expr

namespace Compiler.Proto.Converters

open AST
open Messages

-- Convert AST.Decl to Proto.Decl
def AST.Decl.toProto : AST.Decl → Messages.Decl
  | .funDecl pub name params ret body =>
    .decl
      (.funDecl
        name
        (params.map (λ (n, t) => { name := n, paramType := t.toProto }))
        ret.toProto
        body.toProto)
      pub
      SourceRange.default
  | .structDecl pub name fields =>
    .decl
      (.structDecl
        name
        (fields.map (λ (n, t) => { name := n, fieldType := t.toProto })))
      pub
      SourceRange.default
  | .enumDecl pub name variants =>
    .decl
      (.enumDecl
        name
        (variants.map (λ (n, ts) =>
          { name := n, payloadTypes := ts.map AST.Ty.toProto })))
      pub
      SourceRange.default

-- Convert Proto.Decl back to AST.Decl
def Decl.toAST : Messages.Decl → AST.Decl
  | { declKind := .funDecl f, isPublic := pub, .. } =>
    .funDecl
      pub
      f.name
      (f.params.map (λ p => (p.name, p.paramType.toAST)))
      f.returnType.toAST
      f.body.toAST
  | { declKind := .structDecl s, isPublic := pub, .. } =>
    .structDecl
      pub
      s.name
      (s.fields.map (λ f => (f.name, f.fieldType.toAST)))
  | { declKind := .enumDecl e, isPublic := pub, .. } =>
    .enumDecl
      pub
      e.name
      (e.variants.map (λ v => (v.name, v.payloadTypes.map Ty.toAST)))

-- Convert AST.Module to Proto.Module
def AST.Module.toProto : AST.Module → Messages.Module
  | .mk decls =>
    { name := "main"
      decls := decls.map AST.Decl.toProto
    }

-- Convert Proto.Module back to AST.Module
def Module.toAST : Messages.Module → AST.Module
  | { decls := ds, .. } =>
    .mk (ds.map Decl.toAST)

end Compiler.Proto.Converters
