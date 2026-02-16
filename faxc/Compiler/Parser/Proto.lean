/-
Parser with Protobuf input/output
Accepts TokenStream in protobuf format and returns Module in protobuf format
-/

import Compiler.Parser
import Compiler.Proto

namespace Compiler.Parser.Proto

open Proto
open Proto.Converters

-- Parse from protobuf TokenStream and return protobuf Module
def parseFromProtobuf (tokenStream : Proto.Messages.TokenStream) : Except String Proto.Messages.Module :=
  let tokens := tokenStreamToLexer tokenStream
  match Parser.parseModule tokens with
  | Except.ok module => Except.ok (AST.Module.toProto module)
  | Except.error err => Except.error err

-- Parse from Lean tokens and return protobuf Module
def parseToProtobuf (tokens : List Lexer.Tokens.Token) : Except String Proto.Messages.Module :=
  match Parser.parseModule tokens with
  | Except.ok module => Except.ok (AST.Module.toProto module)
  | Except.error err => Except.error err

-- Parse from serialized protobuf bytes and return serialized protobuf bytes
def parseBytes (data : ByteArray) : Except String ByteArray := do
  let tokenStream ← match Proto.deserializeTokenStream data with
    | some ts => Except.ok ts
    | none => Except.error "Failed to deserialize TokenStream"
  let module ← parseFromProtobuf tokenStream
  return Proto.serializeModule module

-- Parse protobuf Module back to Lean AST
def moduleToAST (module : Proto.Messages.Module) : AST.Module :=
  Module.toAST module

end Compiler.Parser.Proto
