/-
Parser for Declarations
Fixed: Syntax errors, data loss in lists, proper accumulation
-/

import Compiler.Parser.Stmts

namespace Compiler.Parser

open Compiler.AST
open Lexer.Tokens

-- Parse function parameters: (name: Type, name: Type, ...)
partial def parseFunParams (p : Parser) : Parser × List (String × Type) :=
  let (p, _) := p.expect .lparen
  
  let rec go (p : Parser) (acc : List (String × Type)) : Parser × List (String × Type) :=
    match p.peek with
    | .rparen => 
      -- End of parameter list
      (p.advance, acc.reverse)
    | _ =>
      -- Parse parameter: name : Type
      let (p, name) := parseIdent p
      let (p, _) := p.expect .colon
      let (p, ty) := parseType p
      
      -- Check for more parameters or end
      match p.peek with
      | .comma => 
        -- More parameters coming
        go (p.advance) ((name, ty) :: acc)
      | .rparen => 
        -- Last parameter
        (p.advance, ((name, ty) :: acc).reverse)
      | _ => 
        -- Error: expected comma or rparen
        (p, ((name, ty) :: acc).reverse)
  
  go p []

-- Parse return type: -> Type or implicit unit
partial def parseReturnType (p : Parser) : Parser × Type :=
  match p.peek with
  | .arrow =>
    let p := p.advance
    parseType p
  | _ => (p, .unit)

-- Parse function declaration: fn name(params) -> Type { body }
partial def parseFunDecl (p : Parser) : Parser × Option Decl :=
  let (p, _) := p.expect (.kw Tokens.KeywordToken.kw_fn)
  let (p, name) := parseIdent p
  let (p, params) := parseFunParams p
  let (p, ret) := parseReturnType p
  let (p, body) := parseBlock p
  (p, some (Decl.funDecl false name params ret body))

-- Parse field list: name: Type, name: Type, ...
partial def parseFieldList (p : Parser) : Parser × List (String × Type) :=
  let rec go (p : Parser) (acc : List (String × Type)) : Parser × List (String × Type) :=
    match p.peek with
    | .rbrace => 
      -- End of field list
      (p.advance, acc.reverse)
    | _ =>
      -- Parse field: name : Type
      let (p, name) := parseIdent p
      let (p, _) := p.expect .colon
      let (p, ty) := parseType p
      
      -- Check for more fields or end
      match p.peek with
      | .comma => 
        -- More fields coming
        go (p.advance) ((name, ty) :: acc)
      | .rbrace => 
        -- Last field
        (p.advance, ((name, ty) :: acc).reverse)
      | _ => 
        -- Error: expected comma or rbrace
        (p, ((name, ty) :: acc).reverse)
  
  go p []

-- Parse struct declaration: struct Name { field: Type, ... }
partial def parseStructDecl (p : Parser) : Parser × Option Decl :=
  let (p, _) := p.expect (.kw Tokens.KeywordToken.kw_struct)
  let (p, name) := parseIdent p
  let (p, _) := p.expect .lbrace
  let (p, fields) := parseFieldList p
  (p, some (Decl.structDecl false name fields))

-- Parse variant arguments: (Type, Type, ...)
partial def parseVariantArgs (p : Parser) : Parser × List Type :=
  match p.peek with
  | .lparen =>
    let (p, _) := p.advance
    let (p, args) := parseTypeList p
    (p, args)
  | _ => (p, [])

-- Parse variant list: Variant | Variant(Type), ...
partial def parseVariantList (p : Parser) : Parser × List (String × List Type) :=
  let rec go (p : Parser) (acc : List (String × List Type)) : Parser × List (String × List Type) :=
    match p.peek with
    | .rbrace => 
      -- End of variant list
      (p.advance, acc.reverse)
    | _ =>
      -- Parse variant: Name or Name(Type, ...)
      let (p, varName) := parseIdent p
      let (p, args) := parseVariantArgs p
      
      -- Check for more variants or end
      match p.peek with
      | .comma => 
        -- More variants coming
        go (p.advance) ((varName, args) :: acc)
      | .rbrace => 
        -- Last variant
        (p.advance, ((varName, args) :: acc).reverse)
      | _ => 
        -- Error: expected comma or rbrace
        (p, ((varName, args) :: acc).reverse)
  
  go p []

-- Parse enum declaration: enum Name { Variant, Variant(Type), ... }
partial def parseEnumDecl (p : Parser) : Parser × Option Decl :=
  let (p, _) := p.expect (.kw Tokens.KeywordToken.kw_enum)
  let (p, name) := parseIdent p
  let (p, _) := p.expect .lbrace
  let (p, variants) := parseVariantList p
  (p, some (Decl.enumDecl false name variants))

-- Parse a single declaration
partial def parseDecl (p : Parser) : Parser × Option Decl :=
  match p.peek with
  | .kw Tokens.KeywordToken.kw_fn => parseFunDecl p
  | .kw Tokens.KeywordToken.kw_struct => parseStructDecl p
  | .kw Tokens.KeywordToken.kw_enum => parseEnumDecl p
  | _ => (p, none)

-- Parse multiple declarations
partial def parseDecls (p : Parser) : Parser × List Decl :=
  let rec go (p : Parser) (acc : List Decl) : Parser × List Decl :=
    match p.peek with
    | .eof => (p, acc.reverse)
    | _ =>
      match parseDecl p with
      | (p, some d) => go p (d :: acc)
      | (p, none) => 
        -- Skip unknown token and continue
        match p.peek with
        | .eof => (p, acc.reverse)
        | _ => go (p.advance) acc
  go p []

-- Parse complete module
partial def parseModule (tokens : List Lexer.Tokens.Token) : Except String Module :=
  -- Filter out error tokens and report them
  let errorTokens := tokens.filter (fun t => match t with | .error _ => true | _ => false)
  if !errorTokens.isEmpty then
    let errorMsgs := errorTokens.filterMap (fun t => match t with | .error msg => some msg | _ => none)
    Except.error ("Lexer errors:\n" ++ String.intercalate "\n" errorMsgs)
  else
    let p : Parser := { tokens := tokens, pos := 0 }
    match parseDecls p with
    | (p, decls) => Except.ok (Module.mk decls)

end Compiler.Parser
