/-
Lexer Module - Main Entry Point
Exports all lexer components
-/

import Compiler.Lexer.Tokens
import Compiler.Lexer.State
import Compiler.Lexer.Helpers
import Compiler.Lexer.Proto

namespace Compiler.Lexer

open Tokens

-- Re-export all lexer components
export Tokens (Token TokenType KeywordToken OperatorToken)
export State (LexerState)
export Helpers (isAlpha isDigit isWhitespace takeWhile takeUntil)

def lexIdent (st : State.LexerState) : List Token :=
  let (name, st) := Helpers.takeWhile st Helpers.isAlpha
  let tok := match Tokens.KeywordToken.fromString? name with
    | some k => Token.kw k
    | none => Token.ident name
  tok :: lex st

def lexNumber (st : State.LexerState) : List Token :=
  let (num, st) := Helpers.takeWhile st (fun c => Helpers.isDigit c ∨ c = '.')
  if num.contains '.' then
    match Lean.Float.ofString? num with
    | some f => Token.lit_float f :: lex st
    | none => lex st
  else
    match num.toInt? with
    | some n => Token.lit_int n :: lex st
    | none => lex st

def lexString (st : State.LexerState) : List Token :=
  let st := st.adv
  let (str, st) := Helpers.takeUntil st '"'
  Token.lit_string str :: lex (st.adv)

def lexChar (st : State.LexerState) : List Token :=
  let st := st.adv
  match st.peek with
  | some c =>
    let st := st.adv
    match st.peek with
    | some '\'' => Token.lit_char c :: lex (st.adv)
    | _ => lex st
  | none => lex st

def lexSymbol (st : State.LexerState) : List Token :=
  let rec tryMatch (syms : List String) : List Token :=
    match syms with
    | [] => 
      -- Unknown character, report error and skip
      let errChar := st.peek.getD '?'
      Token.error s!"Unexpected character: '{errChar}'" :: lex st.adv
    | s :: ss =>
      if st.input.length >= st.pos + s.length then
        let sub := st.extract s.length
        if sub = s then
          match Tokens.OperatorToken.fromString? s with
          | some op => 
            -- Advance by actual token length, not hardcoded value
            let newSt := List.range s.length |>.foldl (fun st _ => st.adv) st
            Token.op op :: lex newSt
          | none => lex st
        else tryMatch ss
      else tryMatch ss
  -- Sort by length descending to match longest first (e.g., "==" before "=")
  let sortedSyms := ["==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "->", "=>", ".."]
    |>.qsort (fun a b => a.length > b.length)
  tryMatch sortedSyms

partial def lex (st : State.LexerState) : List Token :=
  match st.peek with
  | none => [Token.eof]
  | some c =>
    if Helpers.isWhitespace c then lex (st.adv)
    else if Helpers.isAlpha c then lexIdent st
    else if Helpers.isDigit c then lexNumber st
    else if c = '"' then lexString st
    else if c = '\'' then lexChar st
    else lexSymbol st

def lex (input : String) : List Token :=
  lex { input := input, pos := 0 }

end Compiler.Lexer
