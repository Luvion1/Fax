/-
Protobuf Codec Module - Main Entry Point

Encoding and decoding functions for Protocol Buffer serialization.

## Submodules

- Token: Token stream encoding/decoding
- Types: Type and literal encoding/decoding  
- AST: AST node encoding/decoding
-}

import Compiler.Proto.Codec.Token
import Compiler.Proto.Codec.Types
import Compiler.Proto.Codec.AST

namespace Compiler.Proto.Codec

-- Re-export all codec functions
export Token (serializeTokenStream deserializeTokenStream encodeTokenStream decodeTokenStream)
export Types (encodeTy decodeTy encodeLiteral decodeLiteral)
export AST (serializeModule deserializeModule encodeModule decodeModule
            encodeExpr decodeExpr encodeStmt decodeStmt
            encodePattern decodePattern encodeDecl decodeDecl)

-- Version info
def codecVersion : String := "Fax Protobuf Codec v0.0.1"

end Compiler.Proto.Codec
