namespace Compiler.Lexer.Tokens

inductive KeywordToken where
  | kw_fn | kw_let | kw_mut | kw_if | kw_else | kw_match
  | kw_struct | kw_enum | kw_return | kw_true | kw_false
  | kw_while | kw_loop | kw_break | kw_continue
  | kw_pub | kw_mod | kw_use | kw_as

def KeywordToken.fromString? : String → Option KeywordToken
  | "fn" => some .kw_fn
  | "let" => some .kw_let
  | "mut" => some .kw_mut
  | "if" => some .kw_if
  | "else" => some .kw_else
  | "match" => some .kw_match
  | "struct" => some .kw_struct
  | "enum" => some .kw_enum
  | "return" => some .kw_return
  | "true" => some .kw_true
  | "false" => some .kw_false
  | "while" => some .kw_while
  | "loop" => some .kw_loop
  | "break" => some .kw_break
  | "continue" => some .kw_continue
  | "pub" => some .kw_pub
  | _ => none

end Compiler.Lexer.Tokens
