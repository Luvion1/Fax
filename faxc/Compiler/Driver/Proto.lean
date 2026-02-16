/-
Compiler Driver with Protobuf and FGC support
Orchestrates compilation using protobuf for inter-component communication
and FGC for memory management
-/

import Compiler.Driver
import Compiler.Lexer.Proto
import Compiler.Parser.Proto
import Compiler.Codegen.Proto
import Compiler.Proto
import Compiler.Proto.Services
import Compiler.Runtime

namespace Compiler.Driver.Proto

open Proto
open Services
open Runtime

-- Compile using protobuf pipeline (in-process)
def compileWithProtobuf (source : String) (filename : String := "input.fax") : Except String String := do
  -- Step 1: Lexing - produce TokenStream protobuf
  let tokenStream := Lexer.Proto.lexToProtobuf source filename
  
  -- Step 2: Parsing - consume TokenStream protobuf, produce Module protobuf
  let module ← Parser.Proto.parseFromProtobuf tokenStream
  
  -- Step 3: Codegen - consume Module protobuf, produce LLVM IR
  let ir := Codegen.Proto.generateFromProtobuf module
  
  return ir

-- Compile with detailed protobuf steps
def compileWithProtobufDetailed (source : String) (filename : String := "input.fax") 
    : Except String (Proto.Messages.TokenStream × Proto.Messages.Module × String) := do
  -- Step 1: Lexing
  let tokenStream := Lexer.Proto.lexToProtobuf source filename
  
  -- Step 2: Parsing
  let module ← Parser.Proto.parseFromProtobuf tokenStream
  
  -- Step 3: Codegen
  let ir := Codegen.Proto.generateFromProtobuf module
  
  return (tokenStream, module, ir)

-- Compile using serialized protobuf bytes (for IPC/network)
def compileFromBytes (data : ByteArray) : Except String ByteArray := do
  -- Deserialize TokenStream
  let tokenStream ← match Proto.deserializeTokenStream data with
    | some ts => Except.ok ts
    | none => Except.error "Failed to deserialize TokenStream"
  
  -- Parse to Module
  let module ← Parser.Proto.parseFromProtobuf tokenStream
  
  -- Serialize Module
  let moduleBytes := Proto.serializeModule module
  
  return moduleBytes

-- Compile with FGC-managed memory
def compileWithFGC (source : String) (filename : String := "input.fax")
    (heapSize : Nat := 256 * 1024 * 1024) : IO (Except String String) := do
  try
    -- Create runtime with FGC
    let pool ← Runtime.createRuntime heapSize 1
    
    -- Allocate memory for source
    let sourceBytes := source.toUTF8
    let requestSize := sourceBytes.size * 3  -- Estimate output size
    
    -- Compile with GC management
    let result ← Runtime.compileWithGC pool source
    
    -- Cleanup
    Runtime.shutdownRuntime pool
    
    return result
  catch e =>
    return Except.error s!"FGC compilation failed: {e}"

-- Compile with detailed FGC monitoring
def compileWithFGCMonitored (source : String) (filename : String := "input.fax")
    : IO (Except String (String × String)) := do
  try
    -- Create runtime
    let pool ← Runtime.createRuntime (512 * 1024 * 1024) 1
    
    -- Get initial stats
    let initialStats ← Runtime.getRuntimeStats pool
    
    -- Compile
    let result ← Runtime.compileWithGC pool source
    
    -- Get final stats
    let finalStats ← Runtime.getRuntimeStats pool
    
    -- Combine stats
    let fullStats := initialStats ++ "\n" ++ "=== After Compilation ===\n" ++ finalStats
    
    -- Cleanup
    Runtime.shutdownRuntime pool
    
    match result with
    | Except.ok ir => return Except.ok (ir, fullStats)
    | Except.error e => return Except.error e
  catch e =>
    return Except.error s!"Monitored compilation failed: {e}"

-- Remote compilation using gRPC with FGC
def compileRemote (source : String) (registry : ServiceRegistry) : IO (Except String String) := do
  try
    -- Create runtime with FGC
    let pool ← Runtime.createRuntime (256 * 1024 * 1024) 4
    
    -- Create clients
    let lexerClient ← Grpc.LexerClient.new registry.lexer
    let parserClient ← Grpc.ParserClient.new registry.parser
    let codegenClient ← Grpc.CodegenClient.new registry.codegen
    
    -- Step 1: Call Lexer service with GC-managed memory
    let lexReq : LexRequest := { source := source, filename := "input.fax" }
    let lexResp ← Grpc.LexerClient.tokenize lexerClient lexReq
    
    match lexResp.tokens with
    | none => 
      Runtime.shutdownRuntime pool
      return Except.error s!"Lexing failed: {lexResp.error}"
    | some tokenStream =>
      -- Step 2: Parse
      let module ← match Parser.Proto.parseFromProtobuf tokenStream with
        | Except.ok m => pure m
        | Except.error err => do
          Runtime.shutdownRuntime pool
          return Except.error err
      
      -- Step 3: Codegen
      let ir := Codegen.Proto.generateFromProtobuf module
      
      -- Cleanup
      Runtime.shutdownRuntime pool
      
      return Except.ok ir
  catch e =>
    return Except.error s!"Remote compilation failed: {e}"

-- Print help
def printHelp : IO Unit := do
  IO.println "Fax Compiler v0.0.1 with Protobuf and FGC v0.0.2"
  IO.println ""
  IO.println "Usage: faxc-proto [options] <input.fax>"
  IO.println ""
  IO.println "Options:"
  IO.println "  -o <file>           Output file (default: input.ll)"
  IO.println "  --fgc               Use FGC for memory management"
  IO.println "  --fgc-heap <size>   FGC heap size in MB (default: 256)"
  IO.println "  --fgc-monitor       Monitor GC performance"
  IO.println "  --remote <host> <port>  Remote compilation"
  IO.println "  --services <n>      Number of services (default: 1)"
  IO.println "  --stats             Show runtime statistics"
  IO.println "  --help              Show this help message"
  IO.println ""
  IO.println "Examples:"
  IO.println "  faxc-proto input.fax"
  IO.println "  faxc-proto --fgc --fgc-heap 512 input.fax"
  IO.println "  faxc-proto --fgc --fgc-monitor input.fax"
  IO.println "  faxc-proto --remote localhost 50051 input.fax"

-- Parse command line arguments
structure ProtocArgs where
  inputFile : String := ""
  outputFile : Option String := none
  useFGC : Bool := false
  fgcHeapMB : Nat := 256
  fgcMonitor : Bool := false
  remote : Option (String × Nat) := none
  numServices : Nat := 1
  showStats : Bool := false
  showHelp : Bool := false

def parseArgs (args : List String) : ProtocArgs :=
  let rec go (args : List String) (acc : ProtocArgs) : ProtocArgs :=
    match args with
    | [] => acc
    | "-o" :: file :: rest =>
      go rest { acc with outputFile := some file }
    | "--fgc" :: rest =>
      go rest { acc with useFGC := true }
    | "--fgc-heap" :: size :: rest =>
      go rest { acc with fgcHeapMB := size.toNat! }
    | "--fgc-monitor" :: rest =>
      go rest { acc with fgcMonitor := true }
    | "--remote" :: host :: port :: rest =>
      go rest { acc with remote := some (host, port.toNat!) }
    | "--services" :: n :: rest =>
      go rest { acc with numServices := n.toNat! }
    | "--stats" :: rest =>
      go rest { acc with showStats := true }
    | "--help" :: _ =>
      { acc with showHelp := true }
    | file :: rest =>
      if acc.inputFile == "" then
        go rest { acc with inputFile := file }
      else
        go rest acc
  go args {}

-- Main entry point with protobuf and FGC
def main (args : List String) : IO Unit := do
  let parsed := parseArgs args
  
  if parsed.showHelp || parsed.inputFile == "" then
    printHelp
    if parsed.inputFile == "" && !parsed.showHelp then
      IO.exit 1
    return
  
  -- Read source file
  let source ← IO.readFile parsed.inputFile
  
  -- Determine output file
  let outFile := match parsed.outputFile with
    | some f => f
    | none => parsed.inputFile.replaceSuffix ".fax" ".ll"
  
  -- Compile based on options
  if parsed.useFGC then
    IO.println "Using FGC for memory management..."
    let heapSize := parsed.fgcHeapMB * 1024 * 1024
    
    if parsed.fgcMonitor then
      -- Compile with monitoring
      match ← compileWithFGCMonitored source parsed.inputFile with
      | Except.error err =>
        IO.println s!"Error: {err}"
        IO.exit 1
      | Except.ok (ir, stats) =>
        IO.writeFile outFile ir
        IO.println s!"Compiled to {outFile} (using FGC)"
        IO.println "\n=== FGC Statistics ==="
        IO.println stats
    else
      -- Standard FGC compilation
      match ← compileWithFGC source parsed.inputFile heapSize with
      | Except.error err =>
        IO.println s!"Error: {err}"
        IO.exit 1
      | Except.ok ir =>
        IO.writeFile outFile ir
        IO.println s!"Compiled to {outFile} (using FGC)"
    
  else if parsed.remote.isSome then
    -- Remote compilation
    match parsed.remote with
    | some (host, port) =>
      let registry : ServiceRegistry := {
        lexer := { host := host, port := port }
        parser := { host := host, port := port + 1 }
        codegen := { host := host, port := port + 2 }
      }
      match ← compileRemote source registry with
      | Except.error err =>
        IO.println s!"Error: {err}"
        IO.exit 1
      | Except.ok ir =>
        IO.writeFile outFile ir
        IO.println s!"Compiled remotely to {outFile}"
    | none => pure ()
  
  else
    -- Standard protobuf compilation
    match compileWithProtobuf source parsed.inputFile with
    | Except.error err =>
      IO.println s!"Error: {err}"
      IO.exit 1
    | Except.ok ir =>
      IO.writeFile outFile ir
      IO.println s!"Compiled to {outFile} (using protobuf)"

end Compiler.Driver.Proto
