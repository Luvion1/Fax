namespace Compiler.Lexer.State

structure LexerState where
  input : String
  pos : Nat

def LexerState.peek (st : LexerState) : Option Char :=
  if st.pos < st.input.length then some (st.input.get! (String.Pos.mk st.pos)) else none

def LexerState.adv (st : LexerState) : LexerState :=
  { st with pos := st.pos + 1 }

def LexerState.extract (st : LexerState) (len : Nat) : String :=
  st.input.extract (String.Pos.mk st.pos) (String.Pos.mk (st.pos + len))

end Compiler.Lexer.State
