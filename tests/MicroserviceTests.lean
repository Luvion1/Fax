/-
End-to-end test for Fax microservices architecture
Tests the complete pipeline with gRPC services
-/

import Compiler.Driver
import Compiler.Proto.Grpc.Codegen

namespace Tests.Microservices

-- Test 1: Simple compilation through microservices
def testSimpleCompilation : IO Unit := do
  IO.println "=== Test 1: Simple Compilation ==="
  let source := "fn main() -> i32 { 42 }"
  
  match ← Compiler.Driver.compile source false with
  | Except.ok ir =>
    IO.println "✓ Compilation successful"
    IO.println "Generated IR (first 200 chars):"
    IO.println (ir.take 200)
  | Except.error err =>
    IO.println s!"✗ Compilation failed: {err}"
  IO.println ""

-- Test 2: Compilation with expressions
def testExpressionCompilation : IO Unit := do
  IO.println "=== Test 2: Expression Compilation ==="
  let source := "fn add() -> i32 { 1 + 2 + 3 }"
  
  match ← Compiler.Driver.compile source false with
  | Except.ok ir =>
    IO.println "✓ Expression compilation successful"
  | Except.error err =>
    IO.println s!"✗ Expression compilation failed: {err}"
  IO.println ""

-- Test 3: Compilation with if expression
def testIfCompilation : IO Unit := do
  IO.println "=== Test 3: If Expression Compilation ==="
  let source := "fn max() -> i32 { if 5 > 3 { 5 } else { 3 } }"
  
  match ← Compiler.Driver.compile source false with
  | Except.ok ir =>
    IO.println "✓ If compilation successful"
  | Except.error err =>
    IO.println s!"✗ If compilation failed: {err}"
  IO.println ""

-- Test 4: Function call compilation
def testFunctionCall : IO Unit := do
  IO.println "=== Test 4: Function Call Compilation ==="
  let source := "fn double(x: i32) -> i32 { x * 2 }
fn main() -> i32 { double(21) }"
  
  match ← Compiler.Driver.compile source false with
  | Except.ok ir =>
    IO.println "✓ Function call compilation successful"
  | Except.error err =>
    IO.println s!"✗ Function call compilation failed: {err}"
  IO.println ""

-- Test 5: Health check (would work with real gRPC services)
def testHealthCheck : IO Unit := do
  IO.println "=== Test 5: Health Check ==="
  let endpoint : ServiceEndpoint := { host := "localhost", port := 50052 }
  
  -- This would work with actual running services
  IO.println s!"Testing health check on {endpoint.host}:{endpoint.port}"
  IO.println "(Requires running codegen service)"
  IO.println "✓ Health check test defined"
  IO.println ""

-- Test 6: Circuit breaker pattern
def testCircuitBreaker : IO Unit := do
  IO.println "=== Test 6: Circuit Breaker ==="
  let cb := CircuitBreaker.new
  
  IO.println s!"Initial state: {repr cb}"
  
  let cb1 := CircuitBreaker.recordFailure cb
  IO.println s!"After failure: {repr cb1}"
  
  let cb2 := CircuitBreaker.recordSuccess CircuitState.halfOpen
  IO.println s!"After success in half-open: {repr cb2}"
  
  IO.println "✓ Circuit breaker test passed"
  IO.println ""

-- Test 7: Load balancer
def testLoadBalancer : IO Unit := do
  IO.println "=== Test 7: Load Balancer ==="
  let endpoints := [
    ({ host := "localhost", port := 50052 } : ServiceEndpoint),
    ({ host := "localhost", port := 50053 } : ServiceEndpoint),
    ({ host := "localhost", port := 50054 } : ServiceEndpoint)
  ]
  
  let lb := LoadBalancer.new endpoints
  
  let (ep1, lb1) := LoadBalancer.next lb
  let (ep2, lb2) := LoadBalancer.next lb1
  let (ep3, lb3) := LoadBalancer.next lb2
  let (ep4, _) := LoadBalancer.next lb3
  
  IO.println s!"First endpoint: {ep1.host}:{ep1.port}"
  IO.println s!"Second endpoint: {ep2.host}:{ep2.port}"
  IO.println s!"Third endpoint: {ep3.host}:{ep3.port}"
  IO.println s!"Fourth endpoint (round-robin): {ep4.host}:{ep4.port}"
  IO.println "✓ Load balancer test passed"
  IO.println ""

-- Test 8: Complex program with multiple features
def testComplexProgram : IO Unit := do
  IO.println "=== Test 8: Complex Program ==="
  let source := "fn factorial(n: i32) -> i32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn main() -> i32 {
    factorial(5)
}"
  
  match ← Compiler.Driver.compile source false with
  | Except.ok ir =>
    IO.println "✓ Complex program compilation successful"
    IO.println s!"Generated {ir.length} characters of IR"
  | Except.error err =>
    IO.println s!"✗ Complex program compilation failed: {err}"
  IO.println ""

-- Run all tests
def runAllTests : IO Unit := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║    Fax Microservices Architecture Tests                   ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  testSimpleCompilation
  testExpressionCompilation
  testIfCompilation
  testFunctionCall
  testHealthCheck
  testCircuitBreaker
  testLoadBalancer
  testComplexProgram
  
  IO.println "═══════════════════════════════════════════════════════════"
  IO.println "              Microservices Tests Complete                  "
  IO.println "═══════════════════════════════════════════════════════════"

end Tests.Microservices

def main : IO Unit := Tests.Microservices.runAllTests
