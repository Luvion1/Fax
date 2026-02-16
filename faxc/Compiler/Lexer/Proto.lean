/-
Lexer with Protobuf output
Returns TokenStream in protobuf format
-/

import Compiler.Lexer
import Compiler.Proto

namespace Compiler.Lexer.Proto

open Proto
open Proto.Converters

-- Lex source code and return protobuf TokenStream
def lexToProtobuf (source : String) (filename : String := "input.fax") : Proto.Messages.TokenStream :=
  let tokens := Lexer.lex source
  tokensToProto tokens filename source

-- Lex source code and return serialized protobuf bytes
def lexToBytes (source : String) (filename : String := "input.fax") : ByteArray :=
  let tokenStream := lexToProtobuf source filename
  Proto.serializeTokenStream tokenStream

-- Parse protobuf TokenStream and return Lean tokens
def parseFromProtobuf (tokenStream : Proto.Messages.TokenStream) : List Tokens.Token :=
  tokenStreamToLexer tokenStream

-- Parse serialized protobuf bytes and return Lean tokens
def parseFromBytes (data : ByteArray) : Option (List Tokens.Token) :=
  match Proto.deserializeTokenStream data with
  | some ts => some (tokenStreamToLexer ts)
  | none => none

end Compiler.Lexer.Proto
