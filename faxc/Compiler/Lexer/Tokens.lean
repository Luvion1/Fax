import Compiler.Lexer.Tokens.Keywords
import Compiler.Lexer.Tokens.Operators

namespace Compiler.Lexer.Tokens

inductive Token where
  | kw (k : KeywordToken)
  | op (op : OperatorToken)
  | ident (name : String)
  | lit_int (val : Int) | lit_float (val : Float)
  | lit_string (val : String) | lit_char (val : Char)
  | lparen | rparen | lbrace | rbrace | lbracket | rbracket
  | comma | colon | semicolon | dot | arrow | pipe
  | underscore | arrowfat
  | error (msg : String)  -- Error token for invalid input
  | eof

end Compiler.Lexer.Tokens
