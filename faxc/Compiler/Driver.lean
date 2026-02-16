/-
Compiler Driver - Microservices Architecture
Main entry point for the Fax compiler with full microservice support
-/

import Compiler.Driver.IO
import Compiler.Driver.Proto
import Compiler.Codegen.Proto
import Compiler.Proto.Grpc.Codegen

namespace Compiler.Driver

-- Re-export main driver modules
export IO (readFile writeFile)
export Proto (compileWithProtobuf compileWithFGC compileWithFGCMonitored
              compileRemote compileFromBytes main)

-- Re-export Codegen microservice
export Compiler.Codegen.Proto (compileWithMicroservices handleCodegenService
                               generateFromProtobuf generateWithResponse)

-- Re-export gRPC Codegen
export Compiler.Proto.Grpc.Codegen (CodegenClient CodegenServer
                                     healthCheck LoadBalancer CircuitBreaker)

-- Version
partial def version : String := "0.0.1"

-- High-level compilation API
def compile (source : String) (useMicroservices : Bool := false) : IO (Except String String) := do
  if useMicroservices then
    -- Use full microservice pipeline with FGC
    compileWithMicroservices source true
  else
    -- Use direct compilation (no microservices)
    match Proto.compileWithProtobuf source with
    | Except.ok ir => return Except.ok ir
    | Except.error e => return Except.error e

-- Compile with specific endpoint configuration
def compileWithEndpoints (source : String) 
    (lexerEP : ServiceEndpoint) 
    (parserEP : ServiceEndpoint) 
    (codegenEP : ServiceEndpoint) : IO (Except String String) := do
  try
    -- Step 1: Create gRPC clients
    let lexerClient ← Grpc.LexerClient.new lexerEP
    let parserClient ← Grpc.ParserClient.new parserEP
    let codegenClient ← Grpc.CodegenClient.new codegenEP
    
    -- Step 2: Lexing
    let lexReq : LexRequest := { source := source, filename := "input.fax" }
    let lexResp ← Grpc.LexerClient.tokenize lexerClient lexReq
    
    match lexResp.tokens with
    | none =>
      Grpc.CodegenClient.close codegenClient
      return Except.error "Lexing service failed"
    | some tokens =>
      -- Step 3: Parsing
      let parseReq : ParseRequest := { tokens := tokens }
      let parseResp ← Grpc.ParserClient.parse parserClient parseReq
      
      match parseResp.ast with
      | none =>
        Grpc.CodegenClient.close codegenClient
        return Except.error "Parsing service failed"
      | some ast =>
        -- Step 4: Codegen
        let codegenReq : CodegenRequest := {
          ast := ast,
          options := {
            targetTriple := "x86_64-unknown-linux-gnu",
            optLevel := 2,
            emitDebug := false
          }
        }
        
        let codegenResp ← Grpc.CodegenClient.generateIR codegenClient codegenReq
        
        -- Cleanup
        Grpc.CodegenClient.close codegenClient
        
        match codegenResp.llvmIR with
        | none =>
          match codegenResp.error with
          | some err => return Except.error s!"Codegen failed: {err}"
          | none => return Except.error "Codegen failed: Unknown error"
        | some ir =>
          return Except.ok ir
  catch e =>
    return Except.error s!"Compilation failed: {e}"

-- Batch compile multiple sources
def compileBatch (sources : List String) (useMicroservices : Bool := false) : IO (List (Except String String)) := do
  sources.mapM (fun source => compile source useMicroservices)

-- Main entry point with command line arguments
def main (args : List String) : IO UInt32 := do
  if args.isEmpty then
    IO.println "Fax Compiler v0.0.1"
    IO.println ""
    IO.println "Usage: faxc [options] <input.fax>"
    IO.println ""
    IO.println "Options:"
    IO.println "  --microservices    Use microservice architecture"
    IO.println "  --fgc             Use FGC (Fax Garbage Collector)"
    IO.println "  -o <file>         Output file"
    IO.println "  -v, --version     Show version"
    IO.println "  -h, --help        Show help"
    return 0
  
  let inputFile := args.head!
  
  if inputFile == "-v" || inputFile == "--version" then
    IO.println version
    return 0
  
  if inputFile == "-h" || inputFile == "--help" then
    IO.println "Fax Compiler v0.0.1"
    IO.println ""
    IO.println "Usage: faxc [options] <input.fax>"
    return 0
  
  -- Parse flags
  let useMicroservices := args.contains "--microservices"
  let useFGC := args.contains "--fgc"
  
  let outputFile := match args.indexOf? "-o" with
  | some idx => if idx + 1 < args.length then args.get! (idx + 1) else inputFile.replace ".fax" ".ll"
  | none => inputFile.replace ".fax" ".ll"
  
  IO.println s!"Compiling {inputFile}..."
  if useMicroservices then
    IO.println "Using microservice architecture..."
  if useFGC then
    IO.println "Using FGC..."
  
  -- Read source
  let source ← IO.readFile inputFile
  
  -- Compile
  match ← compile source useMicroservices with
  | Except.error err =>
    IO.println s!"Error: {err}"
    return 1
  | Except.ok ir =>
    IO.writeFile outputFile ir
    IO.println s!"Generated: {outputFile}"
    return 0

end Compiler.Driver
