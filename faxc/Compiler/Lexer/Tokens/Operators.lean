namespace Compiler.Lexer.Tokens

inductive OperatorToken where
  | plus | minus | star | slash | percent
  | eqeq | neq | lt | le | gt | ge
  | and | or | not
  | assign | plusEq | minusEq | starEq | slashEq

def OperatorToken.fromString? : String → Option OperatorToken
  | "+" => some .plus
  | "-" => some .minus
  | "*" => some .star
  | "/" => some .slash
  | "%" => some .percent
  | "==" => some .eqeq
  | "!=" => some .neq
  | "<" => some .lt
  | "<=" => some .le
  | ">" => some .gt
  | ">=" => some .ge
  | "&&" => some .and
  | "||" => some .or
  | "!" => some .not
  | "=" => some .assign
  | "+=" => some .plusEq
  | "-=" => some .minusEq
  | "*=" => some .starEq
  | "/=" => some .slashEq
  | _ => none

end Compiler.Lexer.Tokens
