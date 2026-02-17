# Fax Compiler Technology Stack

## Core Language & Runtime

### Programming Language
- **Primary**: Rust (Edition 2021)
- **Minimum Version**: 1.75
- **Reason**: Memory safety, zero-cost abstractions, excellent compiler infrastructure

### Type System
- **Paradigm**: Statically typed with type inference
- **Inference Algorithm**: Hindley-Milner with extensions
- **Features**:
  - Generic types
  - Algebraic data types (ADT)
  - Pattern matching
  - Trait-based polymorphism
  - Lifetime management (for non-GC resources)

### Memory Management
- **Primary**: Garbage Collection (FGC - Fax Garbage Collector)
- **GC Type**: Generational, concurrent, tri-color mark-sweep
- **Design**: Low-latency, optimized for server workloads
- **Fallback**: Manual memory management for system resources

## Compiler Architecture

### Frontend
- **Lexer**: Hand-written scanner
  - Streaming tokenization
  - Efficient source mapping
  - Unicode support (UTF-8)

- **Parser**: Recursive descent with Pratt parsing
  - Operator precedence climbing
  - Error recovery (panic mode)
  - Concrete Syntax Tree (CST) → Abstract Syntax Tree (AST)

- **Semantic Analyzer**:
  - Name resolution (lexical scoping)
  - Type checking (bidirectional)
  - Type inference (constraint-based)
  - High-level IR (HIR) generation

### Middle-End
- **MIR (Mid-level IR)**: SSA-based
  - Static Single Assignment form
  - Basic block structure
  - Phi nodes for control flow

- **Optimizations**:
  - Constant folding (cst.rs)
  - Dead code elimination (dce.rs)
  - Function inlining (inl.rs)
  - Loop invariant code motion (licm.rs)
  - Tail call optimization
  - Escape analysis (for GC optimization)

- **Pass Manager**: Modular optimization pipeline
  - Analysis passes
  - Transformation passes
  - Dependency tracking

### Backend
- **LIR (Low-level IR)**: Target-agnostic
  - Virtual registers
  - Three-address code

- **Register Allocation**: Graph coloring algorithm
  - Chaitin-Briggs allocator
  - Spill code insertion

- **Code Generation**:
  - LLVM backend (primary)
  - Direct assembly emission (optional)
  - Object file generation
  - Linking (system linker integration)

## Key Dependencies

### Core Infrastructure
| Crate | Purpose |
|-------|---------|
| thiserror | Structured error handling |
| anyhow | Error propagation |
| indexmap | Ordered collections |
| rustc-hash | High-performance hashing |

### Concurrency & Memory
| Crate | Purpose |
|-------|---------|
| parking_lot | High-performance synchronization |
| crossbeam | Lock-free data structures |
| rayon | Data parallelism |

### Build System
- **Tool**: Cargo (Rust native)
- **Workspace**: Multi-crate organization
- **Profiles**: Optimized dev/release builds
- **Features**: Conditional compilation

## Garbage Collector (FGC)

### Design Principles
1. **Concurrent**: Mutator threads run parallel with GC
2. **Generational**: Nursery (young) + Tenured (old) generations
3. **Incremental**: Short pause times via incremental marking
4. **Precise**: Accurate stack and register scanning

### Components
- **Allocator**: Bump-pointer allocation (nursery)
- **Write Barrier**: Card marking for cross-generation refs
- **Mark Phase**: Tri-color marking (white/grey/black)
- **Sweep Phase**: Concurrent sweeping
- **Compaction**: Optional heap defragmentation

### Performance Characteristics
- **Pause Times**: < 10ms typical
- **Throughput**: > 90% application time
- **Scalability**: Multi-threaded collection

## Development Tools

### Debugger (fdb)
- Source-level debugging
- Breakpoints and watchpoints
- Variable inspection
- Call stack navigation
- GC heap inspection

### Profiler (fprof)
- CPU profiling (sampling)
- Memory profiling
- GC pause analysis
- Hot path identification

### Test Runner (ftest)
- Unit test framework
- Integration test runner
- UI test suite (error message testing)
- Benchmark harness

## Standard Library

### Core Library
- Primitive types
- Collections (Vector, Map, Set)
- String handling
- I/O operations
- Error types

### GC Library
- GC-managed smart pointers
- Root management
- Weak references
- Finalization

### System Library
- FFI bindings
- Platform abstractions
- Thread primitives
- Async runtime

## Target Platforms

### Tier 1 (Fully Supported)
- Linux x86_64
- macOS x86_64 & ARM64
- Windows x86_64

### Tier 2 (Supported)
- Linux ARM64
- FreeBSD x86_64
- WebAssembly (WASI)

### Tier 3 (Community)
- Various embedded targets
- Experimental architectures

## Performance Targets

### Compilation Speed
- **Cold Build**: < 30s for 10K LOC
- **Incremental**: < 1s typical change
- **Parallel**: Full CPU utilization

### Runtime Performance
- **Throughput**: Within 10% of C++
- **Latency**: GC pauses < 10ms
- **Memory**: Minimal overhead vs manual management

### Binary Size
- **Minimal**: < 100KB for hello world
- **Optimized**: Aggressive dead code elimination
- **Stripping**: Remove debug symbols for release

## Security Features

### Memory Safety
- GC prevents use-after-free
- Bounds checking on collections
- Null pointer safety

### Type Safety
- Compile-time type checking
- No implicit conversions
- Exhaustive pattern matching

### Sandboxing
- Capability-based security model
- Sandboxed execution environment
- Safe FFI boundaries

## Integration & Tooling

### IDE Support
- Language Server Protocol (LSP)
- Syntax highlighting
- Auto-completion
- Refactoring tools
- Type hints on hover

### Package Manager
- Dependency resolution
- Semantic versioning
- Lock file support
- Private registry support

### CI/CD
- Cross-compilation support
- Docker images
- GitHub Actions integration
- Automated testing

## Documentation

### Generated Docs
- API documentation (rustdoc style)
- Tutorial and guides
- Language specification
- RFCs for major features

### Examples
- Code samples
- Best practices
- Performance patterns
- FFI examples

## Future Roadmap

### Short Term (0.2)
- Async/await support
- Improved error messages
- Package registry

### Medium Term (0.5)
- JIT compilation
- Better cross-compilation
- Advanced optimizations

### Long Term (1.0)
- Self-hosting compiler
- Formal specification
- Certified compilation