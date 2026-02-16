/-
Protobuf Services Module

gRPC service definitions and types for Fax compiler microservices.
-/

import Compiler.Proto.Messages
import Compiler.Proto.Discovery

namespace Compiler.Proto.Services

-- ============================================================================
-- Service Request/Response Types
-- ============================================================================

structure LexRequest where
  source : String
  filename : String
  deriving Repr

structure LexResponse where
  tokens : Option TokenStream
  error : Option String
  deriving Repr

structure ParseRequest where
  tokens : TokenStream
  filename : String
  deriving Repr

structure ParseResponse where
  ast : Option Module
  error : Option String
  deriving Repr

structure CodegenOptions where
  targetTriple : String
  optLevel : Nat
  emitDebug : Bool
  deriving Repr

def CodegenOptions.default : CodegenOptions :=
  { targetTriple := "x86_64-unknown-linux-gnu", optLevel := 2, emitDebug := false }

structure CodegenRequest where
  ast : Module
  options : CodegenOptions
  deriving Repr

structure CodegenResponse where
  llvmIR : Option String
  error : Option String
  deriving Repr

structure CompileRequest where
  source : String
  filename : String
  options : CodegenOptions
  deriving Repr

structure CompileResponse where
  llvmIR : Option String
  errors : List String
  deriving Repr

-- ============================================================================
-- Service Definitions
-- ============================================================================

-- Service identifier
structure LexerService where
  name : String := "LexerService"
  deriving Repr

structure ParserService where
  name : String := "ParserService"
  deriving Repr

structure CodegenService where
  name : String := "CodegenService"
  deriving Repr

structure CompilerService where
  name : String := "CompilerService"
  deriving Repr

-- Re-export service endpoint from Discovery
export Discovery (ServiceEndpoint HealthStatus ServiceRegistry)

-- Service error
inductive ServiceError
  | connectionFailed (message : String)
  | timeout
  | notFound (service : String)
  | invalidRequest (message : String)
  | serverError (code : Nat) (message : String)
  deriving Repr

end Compiler.Proto.Services
