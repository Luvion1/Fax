# Fax Compiler - Microservices Architecture

## Overview

Fax Compiler menggunakan arsitektur microservices dengan komunikasi melalui Protocol Buffers (protobuf) dan gRPC. Setiap komponen utama (Lexer, Parser, Semantic Analyzer, Codegen) berjalan sebagai service independen.

## Arsitektur

```
┌─────────────────────────────────────────────────────────────────┐
│                     Fax Compiler System                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐  │
│  │   Client     │──────▶│   Driver     │──────▶│   Services   │  │
│  └──────────────┘      └──────────────┘      └──────────────┘  │
│                              │                                    │
│                              ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Microservices Pipeline                        │  │
│  │                                                            │  │
│  │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌────────┐ │  │
│  │   │  Lexer  │───▶│  Parser │───▶│ Semantic│───▶│ Codegen│ │  │
│  │   │ Service │    │ Service │    │ Service │    │ Service│ │  │
│  │   └─────────┘    └─────────┘    └─────────┘    └────────┘ │  │
│  │        │              │              │              │      │  │
│  │        └──────────────┴──────────────┴──────────────┘      │  │
│  │                      gRPC + Protobuf                        │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Komponen Microservices

### 1. Lexer Service
**File**: `Compiler/Lexer/Proto.lean`

- **Input**: Source code (String)
- **Output**: TokenStream (protobuf)
- **Fungsi**:
  - `lexToProtobuf`: Tokenize source ke format protobuf
  - `lexToBytes`: Tokenize dan serialize ke bytes
  - `parseFromProtobuf`: Parse protobuf TokenStream ke Lean tokens

**Converter**: `Compiler/Proto/Converters/Token.lean`

### 2. Parser Service
**File**: `Compiler/Parser/Proto.lean`

- **Input**: TokenStream (protobuf)
- **Output**: Module AST (protobuf)
- **Fungsi**:
  - `parseFromProtobuf`: Parse TokenStream ke Module protobuf
  - `parseToProtobuf`: Parse Lean tokens ke Module protobuf
  - `parseBytes`: Parse dari bytes ke bytes

**Converter**: `Compiler/Proto/Converters/Expr.lean`, `Compiler/Proto/Converters/Decl.lean`

### 3. Semantic Analyzer Service
**File**: `Compiler/Semantic/Proto.lean`

- **Input**: Module AST (protobuf)
- **Output**: Analysis Result (Symbol Table, Type Info, Errors)
- **Fungsi**:
  - `analyzeProtobuf`: Analyze Module protobuf
  - `analyzeWithResponse`: Analyze dan return AnalyzeResponse
  - `analyzeFromBytes`: Analyze dari serialized bytes
  - `buildSymbolTable`: Construct symbol table
  - `extractTypeInfo`: Extract type information

### 4. Codegen Service
**File**: `Compiler/Codegen/Proto.lean`

- **Input**: Module AST (protobuf)
- **Output**: LLVM IR (String)
- **Fungsi**:
  - `generateFromProtobuf`: Generate LLVM IR dari Module protobuf
  - `generateWithResponse`: Generate dan return CodegenResponse
  - `generateFromBytes`: Generate dari serialized bytes
  - `handleCodegenService`: Service handler untuk gRPC

**Converter**: `Compiler/Codegen/Proto/Converters/IR.lean`

## Struktur File

```
faxc/
├── Compiler/
│   ├── Lexer/
│   │   ├── Proto.lean          # Lexer microservice
│   │   └── ...
│   ├── Parser/
│   │   ├── Proto.lean          # Parser microservice
│   │   └── ...
│   ├── Semantic/
│   │   ├── Proto.lean          # Semantic analyzer microservice
│   │   └── Semantic.lean
│   ├── Codegen/
│   │   ├── Proto.lean          # Codegen microservice
│   │   └── Proto/
│   │       └── Converters/
│   │           ├── IR.lean     # IR converters
│   │           └── Types.lean
│   ├── Proto/
│   │   ├── Messages.lean       # Protobuf message structures
│   │   ├── Services.lean       # Service definitions
│   │   ├── Grpc.lean          # gRPC client
│   │   ├── GrpcCodegen.lean   # Codegen gRPC extensions
│   │   └── Converters/        # Type converters
│   │       ├── Token.lean
│   │       ├── Types.lean
│   │       ├── Expr.lean
│   │       ├── Pattern.lean
│   │       └── Decl.lean
│   └── Driver/
│       ├── Proto.lean         # Driver dengan microservices
│       └── Simple.lean        # Driver tanpa microservices
```

## Protocol Buffers Schema

### Messages

```protobuf
// Token messages
message Token {
  TokenType type = 1;
  string text = 2;
  SourceRange span = 3;
}

message TokenStream {
  repeated Token tokens = 1;
  string sourceFilename = 2;
  string sourceContent = 3;
}

// AST messages
message Module {
  repeated Decl decls = 1;
}

message Decl {
  oneof decl {
    FuncDecl func = 1;
    StructDecl struct = 2;
    EnumDecl enum = 3;
  }
}

// Service messages
message CodegenRequest {
  Module ast = 1;
  CodegenOptions options = 2;
}

message CodegenResponse {
  optional string llvmIR = 1;
  optional bytes objectFile = 2;
  optional ServiceError error = 3;
}
```

## gRPC Services

### Service Definitions

```protobuf
service LexerService {
  rpc Tokenize(LexRequest) returns (LexResponse);
}

service ParserService {
  rpc Parse(ParseRequest) returns (ParseResponse);
}

service SemanticService {
  rpc Analyze(AnalyzeRequest) returns (AnalyzeResponse);
}

service CodegenService {
  rpc GenerateIR(CodegenRequest) returns (CodegenResponse);
  rpc GenerateObject(CodegenRequest) returns (CodegenResponse);
}
```

## Cara Penggunaan

### 1. Direct Compilation (Non-Microservices)

```lean
import Compiler.Driver.Simple

-- Compile directly
match ← Compiler.Driver.Simple.compileFile "input.fax" with
| Except.ok ir => IO.println ir
| Except.error err => IO.println s!"Error: {err}"
```

### 2. Microservices Pipeline

```lean
import Compiler.Driver

-- Compile with microservices
match ← Compiler.Driver.compile source true with
| Except.ok ir => IO.println ir
| Except.error err => IO.println s!"Error: {err}"
```

### 3. Individual Services

```lean
import Compiler.Lexer.Proto
import Compiler.Parser.Proto
import Compiler.Codegen.Proto

-- Step 1: Lexing
let tokenStream := Lexer.Proto.lexToProtobuf source "input.fax"

-- Step 2: Parsing
match Parser.Proto.parseFromProtobuf tokenStream with
| Except.ok module =>
  -- Step 3: Codegen
  let ir := Codegen.Proto.generateFromProtobuf module
  IO.println ir
| Except.error err => IO.println s!"Parse error: {err}"
```

## Testing

### Unit Tests

```bash
# Run all unit tests
lake exe test-unit

# Run specific component tests
lake exe test-lexer
lake exe test-parser
lake exe test-codegen
lake exe test-semantic
```

### Integration Tests

```bash
# Run integration tests
lake exe test-integration

# Run end-to-end tests
lake exe test-e2e
```

### Test Structure

```
tests/
├── unit/
│   ├── LexerTests.lean
│   ├── ParserTests.lean
│   ├── CodegenTests.lean
│   └── SemanticTests.lean
├── integration/
│   └── PipelineTests.lean
├── e2e/
│   └── (End-to-end tests)
└── TestRunner.lean
```

## Fitur Microservices

### 1. Load Balancing
- Round-robin load balancing untuk multiple service instances
- Definisi: `Compiler/Proto/GrpcCodegen.lean`

### 2. Circuit Breaker
- Fault tolerance dengan circuit breaker pattern
- States: Closed, Open, Half-Open

### 3. Connection Pooling
- Reuse connections untuk performance
- Definisi: `PooledCodegenClient`

### 4. Health Checks
- Monitor service availability
- Definisi: `healthCheck` function

### 5. FGC Integration
- Memory management dengan Fax Garbage Collector
- Automatic heap management per service

## Performansi

### Optimasi
- **Zero-copy messaging**: Efficient data transfer
- **Batch processing**: Compile multiple files dalam satu request
- **Caching**: Cache hasil parsing dan analysis
- **Parallel execution**: Multiple services berjalan paralel

### Monitoring
- **Metrics**: Track request latency, throughput, error rates
- **Logging**: Structured logging untuk setiap service
- **Tracing**: Distributed tracing untuk request flow

## Deployment

### Local Development
```bash
# Start all services
./scripts/start-services.sh

# Compile dengan microservices
./faxc --microservices input.fax
```

### Production
```bash
# Deploy dengan Docker Compose
docker-compose up -d

# Scale services
docker-compose up -d --scale codegen=3 --scale parser=2
```

## Roadmap

### Phase 1 (Current)
- ✅ Basic microservices architecture
- ✅ Protobuf integration
- ✅ Unit & integration tests

### Phase 2 (Next)
- 🔄 Full gRPC implementation
- 🔄 Service discovery
- 🔄 Load balancer

### Phase 3 (Future)
- ⏳ Kubernetes deployment
- ⏳ Auto-scaling
- ⏳ Distributed tracing

## References

- [Protocol Buffers](https://developers.google.com/protocol-buffers)
- [gRPC](https://grpc.io/)
- [Microservices Patterns](https://microservices.io/)
