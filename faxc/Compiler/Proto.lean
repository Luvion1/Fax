/-
Protobuf Support Module - Main Entry Point

Provides Protocol Buffer message structures, serialization, and gRPC services
for the Fax compiler microservices architecture.

## Module Organization

### Core Components
- Messages: Protocol buffer message types (TokenStream, AST, etc.)
- Wire: Wire format encoding/decoding primitives
- Binary: Binary serialization utilities

### Serialization
- Codec: Encoding/decoding functions for tokens and AST
- Converters: Conversions between Lean AST and Protobuf types

### Services
- Services: Service definitions (Lexer, Parser, Codegen)
- Grpc: gRPC client implementations
- GrpcCodegen: gRPC code generation utilities
- Server: gRPC server implementation

### Infrastructure
- Cache: Service result caching
- Discovery: Service discovery and load balancing
- Diagnostics: Error reporting and diagnostics
- Semantic: Semantic analysis via protobuf

### Testing
- Test: Test utilities and test cases
-}

import Compiler.Proto.Messages
import Compiler.Proto.Wire
import Compiler.Proto.Binary
import Compiler.Proto.Codec
import Compiler.Proto.Converters
import Compiler.Proto.Services
import Compiler.Proto.Grpc
import Compiler.Proto.GrpcCodegen
import Compiler.Proto.Server
import Compiler.Proto.Semantic
import Compiler.Proto.Cache
import Compiler.Proto.Diagnostics
import Compiler.Proto.Discovery
import Compiler.Proto.Test

-- Runtime with FGC
import Compiler.Runtime

namespace Compiler.Proto

-- ============================================================================
-- Core Message Types
-- ============================================================================

export Messages (SourcePos SourceRange Token TokenType TokenStream
                 PrimitiveType Ty Literal Pattern
                 UnaryOp BinOp Expr Stmt Decl Module)

-- ============================================================================
-- Serialization
-- ============================================================================

export Codec (serializeTokenStream deserializeTokenStream
              serializeModule deserializeModule
              encodeTokenStream decodeTokenStream
              encodeModule decodeModule
              encodeExpr decodeExpr encodeStmt decodeStmt
              encodePattern decodePattern encodeDecl decodeDecl)

export Converters (tokensToProto tokenStreamToLexer Token.toProto Token.toLexer
                   AST.Ty.toProto Ty.toAST AST.Literal.toProto Literal.toAST
                   AST.Pattern.toProto Pattern.toAST
                   AST.Expr.toProto Expr.toAST AST.Stmt.toProto Stmt.toAST
                   AST.Decl.toProto Decl.toAST AST.Module.toProto Module.toAST)

-- ============================================================================
-- Services
-- ============================================================================

export Services (LexRequest LexResponse LexerService
                 ParseRequest ParseResponse ParserService
                 CodegenOptions CodegenRequest CodegenResponse CodegenService
                 CompileRequest CompileResponse CompilerService
                 ServiceEndpoint ServiceRegistry ServiceError)

export Grpc (Channel LexerClient ParserClient CodegenClient
             GrpcResult GrpcStatus CallOptions
             unaryCall clientStreamingCall serverStreamingCall bidiStreamingCall)

export Server (ServerConfig ServerState
               handleGrpcRequest startServer stopServer
               handleTokenize handleParse handleAnalyze handleGenerateIR handleCompile)

-- ============================================================================
-- Infrastructure
-- ============================================================================

export Cache (CacheKey CacheEntry Cache CacheStats
              createCache CacheOps
              TokenCache ModuleCache incrementalCompile)

export Discovery (HealthStatus ServiceInstance ServiceRegistry
                  LoadBalancer LoadBalancerStrategy ServiceDiscovery
                  CircuitBreaker CircuitBreakerState RetryPolicy
                  SmartClient)

export Diagnostics (Severity Diagnostic Diagnostics Diagnostic.format
                    ReporterM report reportError reportWarning reportInfo
                    ErrorCodes createErrorResponse)

export Semantic (SemanticError Symbol SymbolTable SemEnv
                 analyzeModule runSemanticAnalysis AnalysisResult)

-- ============================================================================
-- Testing
-- ============================================================================

export Test (testSource1 testSource2 testSource3
             testTokenStreamRoundtrip testModuleRoundtrip
             testSemanticAnalysis testCaching testFullPipeline
             runAllTests benchmarkSerialization exampleIPC)

-- ============================================================================
-- Version and Utilities
-- ============================================================================

def version : String := "Fax Protobuf v0.0.1 with FGC v0.0.2"

def isAvailable : Bool := true

-- ============================================================================
-- High-Level API
-- ============================================================================

def serializeTokenStream (ts : TokenStream) : ByteArray :=
  Codec.serializeTokenStream ts

def deserializeTokenStream (data : ByteArray) : Option TokenStream :=
  match Codec.deserializeTokenStream data with
  | Except.ok ts => some ts
  | Except.error _ => none

def serializeModule (m : Module) : ByteArray :=
  Codec.serializeModule m

def deserializeModule (data : ByteArray) : Option Module :=
  match Codec.deserializeModule data with
  | Except.ok m => some m
  | Except.error _ => none

-- Compile with full protobuf pipeline
def compileWithProtobuf (source : String) : Except String String := do
  -- Step 1: Lexing
  let tokens := Compiler.Lexer.lex source
  let tokenStream := Converters.tokensToProto tokens "input.fax" source
  let tokenBytes := serializeTokenStream tokenStream
  
  -- Step 2: Parsing
  let tokens' := Converters.tokenStreamToLexer tokenStream
  let module ← Compiler.Parser.parseModule tokens'
  let protoModule := Converters.AST.Module.toProto module
  let moduleBytes := serializeModule protoModule
  
  -- Step 3: Semantic Analysis
  let analysisResult := Semantic.runSemanticAnalysis protoModule
  if !analysisResult.isValid then
    return Except.error "Semantic analysis failed"
  
  -- Step 4: Codegen
  let ir := Compiler.Codegen.generateIR module
  
  return ir

end Compiler.Proto
