/-
Converter between Lexer.Token and Proto.Token
-/

import Compiler.Lexer.Tokens
import Compiler.Proto.Messages

namespace Compiler.Proto.Converters

open Lexer.Tokens
open Messages

-- Convert KeywordToken to Proto TokenType
def KeywordToken.toProto : KeywordToken → TokenType
  | .kw_fn => .kwFn
  | .kw_let => .kwLet
  | .kw_mut => .kwMut
  | .kw_if => .kwIf
  | .kw_else => .kwElse
  | .kw_match => .kwMatch
  | .kw_struct => .kwStruct
  | .kw_enum => .kwEnum
  | .kw_return => .kwReturn
  | .kw_true => .kwTrue
  | .kw_false => .kwFalse
  | .kw_while => .kwWhile
  | .kw_loop => .kwLoop
  | .kw_break => .kwBreak
  | .kw_continue => .kwContinue
  | .kw_pub => .kwPub
  | .kw_mod => .kwMod
  | .kw_use => .kwUse
  | .kw_as => .kwAs

-- Convert OperatorToken to Proto TokenType
def OperatorToken.toProto : OperatorToken → TokenType
  | .plus => .opAdd
  | .minus => .opSub
  | .star => .opMul
  | .slash => .opDiv
  | .percent => .opMod
  | .eqeq => .opEq
  | .neq => .opNe
  | .lt => .opLt
  | .le => .opLe
  | .gt => .opGt
  | .ge => .opGe
  | .and => .opAnd
  | .or => .opOr
  | .not => .opNot
  | .assign => .opAssign
  | .plusEq => .opAddAssign
  | .minusEq => .opSubAssign
  | .starEq => .opMulAssign
  | .slashEq => .opDivAssign

-- Convert Lexer Token to Proto Token
def Token.toProto : Lexer.Tokens.Token → Messages.Token
  | .kw k => { type := k.toProto, text := "", span := SourceRange.default }
  | .op o => { type := o.toProto, text := "", span := SourceRange.default }
  | .ident name => { type := .ident, text := name, span := SourceRange.default }
  | .lit_int val => { type := .litInt, text := toString val, span := SourceRange.default }
  | .lit_float val => { type := .litFloat, text := toString val, span := SourceRange.default }
  | .lit_string val => { type := .litString, text := val, span := SourceRange.default }
  | .lit_char val => { type := .litChar, text := String.singleton val, span := SourceRange.default }
  | .lparen => { type := .lparen, text := "(", span := SourceRange.default }
  | .rparen => { type := .rparen, text := ")", span := SourceRange.default }
  | .lbrace => { type := .lbrace, text := "{", span := SourceRange.default }
  | .rbrace => { type := .rbrace, text := "}", span := SourceRange.default }
  | .lbracket => { type := .lbracket, text := "[", span := SourceRange.default }
  | .rbracket => { type := .rbracket, text := "]", span := SourceRange.default }
  | .comma => { type := .comma, text := ",", span := SourceRange.default }
  | .colon => { type := .colon, text := ":", span := SourceRange.default }
  | .semicolon => { type := .semicolon, text := ";", span := SourceRange.default }
  | .dot => { type := .dot, text := ".", span := SourceRange.default }
  | .arrow => { type := .arrow, text := "->", span := SourceRange.default }
  | .pipe => { type := .pipe, text := "|", span := SourceRange.default }
  | .underscore => { type := .underscore, text := "_", span := SourceRange.default }
  | .arrowfat => { type := .arrowFat, text := "=>", span := SourceRange.default }
  | .eof => { type := .eof, text := "", span := SourceRange.default }

-- Convert list of Lexer Tokens to Proto TokenStream
def tokensToProto (tokens : List Lexer.Tokens.Token) (filename : String) (source : String) : Messages.TokenStream :=
  { tokens := tokens.map Token.toProto
    sourceFilename := filename
    sourceContent := source
  }

-- Convert Proto TokenType back to KeywordToken (partial)
def TokenType.toKeyword? : TokenType → Option Lexer.Tokens.KeywordToken
  | .kwFn => some .kw_fn
  | .kwLet => some .kw_let
  | .kwMut => some .kw_mut
  | .kwIf => some .kw_if
  | .kwElse => some .kw_else
  | .kwMatch => some .kw_match
  | .kwStruct => some .kw_struct
  | .kwEnum => some .kw_enum
  | .kwReturn => some .kw_return
  | .kwTrue => some .kw_true
  | .kwFalse => some .kw_false
  | .kwWhile => some .kw_while
  | .kwLoop => some .kw_loop
  | .kwBreak => some .kw_break
  | .kwContinue => some .kw_continue
  | .kwPub => some .kw_pub
  | .kwMod => some .kw_mod
  | .kwUse => some .kw_use
  | .kwAs => some .kw_as
  | _ => none

-- Convert Proto TokenType back to OperatorToken (partial)
def TokenType.toOperator? : TokenType → Option Lexer.Tokens.OperatorToken
  | .opAdd => some .plus
  | .opSub => some .minus
  | .opMul => some .star
  | .opDiv => some .slash
  | .opMod => some .percent
  | .opEq => some .eqeq
  | .opNe => some .neq
  | .opLt => some .lt
  | .opLe => some .le
  | .opGt => some .gt
  | .opGe => some .ge
  | .opAnd => some .and
  | .opOr => some .or
  | .opNot => some .not
  | .opAssign => some .assign
  | .opAddAssign => some .plusEq
  | .opSubAssign => some .minusEq
  | .opMulAssign => some .starEq
  | .opDivAssign => some .slashEq
  | _ => none

-- Convert Proto Token back to Lexer Token (partial)
def Token.toLexer : Messages.Token → Lexer.Tokens.Token
  | { type := t, text := text, .. } =>
    match t with
    | .litInt => match text.toInt? with
      | some n => .lit_int n
      | none => .ident text
    | .litFloat => match text.toFloat? with
      | some f => .lit_float f
      | none => .ident text
    | .litString => .lit_string text
    | .litChar => match text.toList with
      | [c] => .lit_char c
      | _ => .ident text
    | .ident => .ident text
    | .lparen => .lparen
    | .rparen => .rparen
    | .lbrace => .lbrace
    | .rbrace => .rbrace
    | .lbracket => .lbracket
    | .rbracket => .rbracket
    | .comma => .comma
    | .colon => .colon
    | .semicolon => .semicolon
    | .dot => .dot
    | .arrow => .arrow
    | .pipe => .pipe
    | .underscore => .underscore
    | .arrowFat => .arrowfat
    | .eof => .eof
    | t => match t.toKeyword? with
      | some k => .kw k
      | none => match t.toOperator? with
        | some o => .op o
        | none => .ident text

-- Convert Proto TokenStream back to list of Lexer Tokens
def tokenStreamToLexer (ts : Messages.TokenStream) : List Lexer.Tokens.Token :=
  ts.tokens.map Token.toLexer

end Compiler.Proto.Converters
