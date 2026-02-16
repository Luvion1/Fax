import Compiler.Lexer.State

namespace Compiler.Lexer.Helpers

def isAlpha (c : Char) : Bool :=
  ('a' ≤ c ∧ c ≤ 'z') ∨ ('A' ≤ c ∧ c ≤ 'Z') ∨ c = '_'

def isDigit (c : Char) : Bool := '0' ≤ c ∧ c ≤ '9'

def isWhitespace (c : Char) : Bool :=
  c = ' ' ∨ c = '\t' ∨ c = '\n' ∨ c = '\r'

def takeWhile {α : Type} (st : State.LexerState) (p : Char → Bool) : String × State.LexerState :=
  let rec aux (acc : List Char) (s : State.LexerState) : String × State.LexerState :=
    match s.peek with
    | some c => if p c then aux (c :: acc) (s.adv) else (acc.reverse.asString, s)
    | none => (acc.reverse.asString, s)
  aux [] st

def takeUntil (st : State.LexerState) (delim : Char) : String × State.LexerState :=
  let rec aux (acc : List Char) (s : State.LexerState) : String × State.LexerState :=
    match s.peek with
    | some c => if c = delim then (acc.reverse.asString, s) else aux (c :: acc) (s.adv)
    | none => (acc.reverse.asString, s)
  aux [] st

end Compiler.Lexer.Helpers
