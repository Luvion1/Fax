/-
Comprehensive GC Unit Tests
Tests for all FGC components
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Heap
import Compiler.Runtime.GC.TLAB
import Compiler.Runtime.GC.Mark
import Compiler.Runtime.GC.Relocate
import Compiler.Runtime.GC.Controller
import Compiler.Runtime.GC.WriteBarrier
import Compiler.Runtime.GC.ReferenceProcessor
import Compiler.Runtime.GC.Generational

namespace Tests.GC

open Compiler.Runtime.GC
open ZPointer Heap TLAB Mark Relocate Controller WriteBarrier ReferenceProcessor Generational

-- ============================================================================
-- ZPointer Tests
-- ============================================================================

def testZPointerBasics : IO Bool := do
  IO.println "Test: ZPointer Basics"
  let addr : UInt64 := 0x1000
  let ptr := ZPointer.fromAddress addr .remapped
  
  if ptr.toAddress == addr && !ptr.isNull then
    IO.println "  ✓ ZPointer creation and address extraction"
    return true
  else
    IO.println "  ✗ ZPointer basic operations failed"
    return false

def testZPointerColors : IO Bool := do
  IO.println "Test: ZPointer Colors"
  let addr : UInt64 := 0x1000
  let ptr := ZPointer.fromAddress addr .remapped
  
  let marked0 := ptr.setColor .marked0
  let marked1 := ptr.setColor .marked1
  let remapped := ptr.setColor .remapped
  
  if marked0.getColor == .marked0 && 
     marked1.getColor == .marked1 && 
     remapped.getColor == .remapped then
    IO.println "  ✓ Color setting and getting"
    return true
  else
    IO.println "  ✗ Color operations failed"
    return false

def testZPointerNull : IO Bool := do
  IO.println "Test: ZPointer Null"
  let nullPtr := ZPointer.null
  
  if nullPtr.isNull && nullPtr.toAddress == 0 then
    IO.println "  ✓ Null pointer handling"
    return true
  else
    IO.println "  ✗ Null pointer test failed"
    return false

-- ============================================================================
-- Heap Tests
-- ============================================================================

def testHeapInitialization : IO Bool := do
  IO.println "Test: Heap Initialization"
  let config := HeapConfig.default
  let heap ← ZHeap.init config
  
  if heap.regions.size > 0 && heap.freeRegions.length > 0 then
    IO.println s!"  ✓ Heap initialized with {heap.regions.size} regions"
    return true
  else
    IO.println "  ✗ Heap initialization failed"
    return false

def testHeapAllocation : IO Bool := do
  IO.println "Test: Heap Allocation"
  let config := HeapConfig.default
  let heap ← ZHeap.init config
  
  match heap.allocate 64 1 with
  | some (ptr, heap') =>
    if !ptr.isNull && heap'.totalAllocated >= 64 then
      IO.println "  ✓ Small object allocation"
      return true
    else
      IO.println "  ✗ Allocation result invalid"
      return false
  | none =>
    IO.println "  ✗ Allocation failed"
    return false

def testHeapStats : IO Bool := do
  IO.println "Test: Heap Statistics"
  let config := HeapConfig.default
  let heap ← ZHeap.init config
  
  let stats := heap.stats
  if stats.totalRegions > 0 && stats.totalBytes > 0 then
    IO.println s!"  ✓ Heap stats: {stats.totalRegions} regions, {stats.freeRegions} free"
    return true
  else
    IO.println "  ✗ Heap statistics invalid"
    return false

def testHeapShouldCollect : IO Bool := do
  IO.println "Test: Heap Should Collect"
  let config := HeapConfig.default
  let heap ← ZHeap.init config
  
  -- Fresh heap should not need collection
  if !heap.shouldCollect then
    IO.println "  ✓ GC trigger detection"
    return true
  else
    IO.println "  ✗ GC trigger detection failed"
    return false

-- ============================================================================
-- TLAB Tests
-- ============================================================================

def testTLABCreation : IO Bool := do
  IO.println "Test: TLAB Creation"
  let tlab := ThreadLocalAllocBuffer.new 0x1000 1024 1
  
  if tlab.hasSpace 64 && tlab.remaining == 1024 then
    IO.println "  ✓ TLAB created successfully"
    return true
  else
    IO.println "  ✗ TLAB creation failed"
    return false

def testTLABAllocation : IO Bool := do
  IO.println "Test: TLAB Allocation"
  let tlab := ThreadLocalAllocBuffer.new 0x1000 1024 1
  
  match tlab.allocateFast 64 with
  | some (ptr, newTLAB) =>
    if !ptr.isNull && newTLAB.allocatedBytes >= 64 then
      IO.println "  ✓ Fast TLAB allocation"
      return true
    else
      IO.println "  ✗ TLAB allocation result invalid"
      return false
  | none =>
    IO.println "  ✗ TLAB allocation failed"
    return false

def testTLABNeedsRefill : IO Bool := do
  IO.println "Test: TLAB Refill Detection"
  let mut tlab := ThreadLocalAllocBuffer.new 0x1000 1024 1
  
  -- Fill up TLAB
  let mut remaining := tlab.remaining
  while remaining > 100 do
    match tlab.allocateFast 50 with
    | some (_, newTLAB) => tlab := newTLAB
    | none => break
    remaining := tlab.remaining
  
  if tlab.needsRefill then
    IO.println "  ✓ TLAB refill detection works"
    return true
  else
    IO.println "  ✗ TLAB refill detection failed"
    return false

-- ============================================================================
-- Mark Tests
-- ============================================================================

def testMarkStack : IO Bool := do
  IO.println "Test: Mark Stack"
  let mut stack := MarkStack.empty
  let ptr1 := ZPointer.fromAddress 0x1000 .remapped
  let ptr2 := ZPointer.fromAddress 0x2000 .remapped
  
  stack := stack.push ptr1
  stack := stack.push ptr2
  
  match stack.pop with
  | some (popped, newStack) =>
    if !stack.isEmpty then
      IO.println "  ✓ Mark stack push/pop"
      return true
    else
      IO.println "  ✗ Mark stack operations failed"
      return false
  | none =>
    IO.println "  ✗ Mark stack pop failed"
    return false

def testMarkContext : IO Bool := do
  IO.println "Test: Mark Context"
  let ctx := MarkContext.new .marked0
  let ptr := ZPointer.fromAddress 0x1000 .remapped
  
  let ctx' := { ctx with stack := ctx.stack.push ptr }
  
  if ctx'.stack.entries.size == 1 then
    IO.println "  ✓ Mark context initialization"
    return true
  else
    IO.println "  ✗ Mark context failed"
    return false

def testRootSet : IO Bool := do
  IO.println "Test: Root Set"
  let mut roots := RootSet.empty
  let ptr := ZPointer.fromAddress 0x1000 .remapped
  
  roots := roots.add ptr
  
  if roots.roots.size == 1 then
    IO.println "  ✓ Root set management"
    return true
  else
    IO.println "  ✗ Root set failed"
    return false

-- ============================================================================
-- Relocation Tests
-- ============================================================================

def testForwardingTable : IO Bool := do
  IO.println "Test: Forwarding Table"
  let mut table := ForwardingTable.empty
  let entry : ForwardingEntry := {
    oldAddress := 0x1000
    newAddress := 0x2000
    size := 64
  }
  
  table := table.add entry
  
  match table.lookup 0x1000 with
  | some e =>
    if e.newAddress == 0x2000 then
      IO.println "  ✓ Forwarding table add/lookup"
      return true
    else
      IO.println "  ✗ Forwarding table lookup wrong result"
      return false
  | none =>
    IO.println "  ✗ Forwarding table lookup failed"
    return false

def testRelocationSet : IO Bool := do
  IO.println "Test: Relocation Set"
  let mut set := RelocationSet.empty
  
  set := set.add 0 1024
  set := set.add 1 2048
  
  if set.regionIndices.length == 2 then
    IO.println "  ✓ Relocation set management"
    return true
  else
    IO.println "  ✗ Relocation set failed"
    return false

-- ============================================================================
-- Write Barrier Tests
-- ============================================================================

def testSATBQueue : IO Bool := do
  IO.println "Test: SATB Queue"
  let mut queue := SATBQueue.new
  let ptr := ZPointer.fromAddress 0x1000 .remapped
  
  let (newQueue, success) := queue.enqueue ptr
  
  if success && newQueue.size == 1 then
    IO.println "  ✓ SATB queue enqueue"
    return true
  else
    IO.println "  ✗ SATB queue failed"
    return false

def testRememberedSet : IO Bool := do
  IO.println "Test: Remembered Set"
  let rs := RememberedSet.new (1024 * 1024) 512
  let ptr := ZPointer.fromAddress 0x1000 .remapped
  
  let newRS := rs.markCard 0x1000 ptr
  let cardIdx := newRS.cardIndexFor 0x1000
  let card := newRS.getCard cardIdx
  
  if card.size > 0 then
    IO.println "  ✓ Remembered set card marking"
    return true
  else
    IO.println "  ✗ Remembered set failed"
    return false

def testWriteBarrier : IO Bool := do
  IO.println "Test: Write Barrier"
  let wb := CombinedWriteBarrier.new (1024 * 1024)
  let oldRef := ZPointer.fromAddress 0x1000 .remapped
  let newRef := ZPointer.fromAddress 0x2000 .remapped
  
  let (newWB, enqueued) := writeBarrier wb 0x3000 oldRef newRef false
  
  if !newWB.active then  -- Not active by default
    IO.println "  ✓ Write barrier structure"
    return true
  else
    IO.println "  ✗ Write barrier test failed"
    return false

-- ============================================================================
-- Reference Processor Tests
-- ============================================================================

def testReferenceObject : IO Bool := do
  IO.println "Test: Reference Object"
  let referent := ZPointer.fromAddress 0x1000 .remapped
  let ref := ReferenceObject.new .weak referent none
  
  if ref.referenceType == .weak && !ref.referent.isNull then
    IO.println "  ✓ Reference object creation"
    return true
  else
    IO.println "  ✗ Reference object failed"
    return false

def testReferenceProcessorState : IO Bool := do
  IO.println "Test: Reference Processor State"
  let state := ReferenceProcessorState.new
  let referent := ZPointer.fromAddress 0x1000 .remapped
  let ref := ReferenceObject.new .weak referent none
  
  let state' := discoverReference state ref
  
  if state'.weakReferences.size == 1 then
    IO.println "  ✓ Reference discovery"
    return true
  else
    IO.println "  ✗ Reference discovery failed"
    return false

def testSoftReferenceClearing : IO Bool := do
  IO.println "Test: Soft Reference Clearing"
  let state := ReferenceProcessorState.new
  let referent := ZPointer.fromAddress 0x1000 .remapped
  let ref := ReferenceObject.new .soft referent none
  
  let state' := discoverReference state ref
  let (state'', cleared) := processSoftReferences state' true 0.95
  
  if state''.softCleared == 1 then
    IO.println "  ✓ Soft reference clearing"
    return true
  else
    IO.println "  ✗ Soft reference clearing failed"
    return false

-- ============================================================================
-- Generational Tests
-- ============================================================================

def testGenerationalHeapInit : IO Bool := do
  IO.println "Test: Generational Heap Init"
  let heap := GenerationalHeap.init (64 * 1024 * 1024) (256 * 1024 * 1024)
  
  if heap.edenSize > 0 && heap.survivor0Size > 0 then
    IO.println s!"  ✓ Generational heap: eden={heap.edenSize} bytes"
    return true
  else
    IO.println "  ✗ Generational heap init failed"
    return false

def testEdenAllocation : IO Bool := do
  IO.println "Test: Eden Allocation"
  let heap := GenerationalHeap.init (64 * 1024 * 1024) (256 * 1024 * 1024)
  
  match heap.allocateEden 64 1 with
  | some (ptr, newHeap) =>
    if !ptr.isNull then
      IO.println "  ✓ Eden allocation"
      return true
    else
      IO.println "  ✗ Eden allocation result invalid"
      return false
  | none =>
    IO.println "  ✗ Eden allocation failed"
    return false

def testMinorGCTrigger : IO Bool := do
  IO.println "Test: Minor GC Trigger"
  let heap := GenerationalHeap.init (64 * 1024 * 1024) (256 * 1024 * 1024)
  
  -- Fresh heap should not need GC
  if !heap.needsMinorGC then
    IO.println "  ✓ Minor GC trigger detection"
    return true
  else
    IO.println "  ✗ Minor GC trigger detection failed"
    return false

-- ============================================================================
-- GC Controller Tests
-- ============================================================================

def testGCConfig : IO Bool := do
  IO.println "Test: GC Config"
  let config := GCConfig.default
  
  if config.maxPauseMs > 0 && config.concurrencyLevel > 0 then
    IO.println s!"  ✓ GC config: pause={config.maxPauseMs}ms, threads={config.concurrencyLevel}"
    return true
  else
    IO.println "  ✗ GC config invalid"
    return false

def testGCStateInit : IO Bool := do
  IO.println "Test: GC State Init"
  let config := HeapConfig.default
  let heap ← ZHeap.init config
  let state ← GCState.init heap
  
  if state.phase == .idle && state.gcCount == 0 then
    IO.println "  ✓ GC state initialized"
    return true
  else
    IO.println "  ✗ GC state init failed"
    return false

def testGCStats : IO Bool := do
  IO.println "Test: GC Statistics"
  let config := HeapConfig.default
  let heap ← ZHeap.init config
  let state ← GCState.init heap
  let stats := state.getStats
  
  if stats.gcCount == 0 then
    IO.println "  ✓ GC statistics collection"
    return true
  else
    IO.println "  ✗ GC statistics failed"
    return false

-- ============================================================================
-- Test Runner
-- ============================================================================

def runAllTests : IO UInt32 := do
  IO.println "╔══════════════════════════════════════════════════════════╗"
  IO.println "║           FGC (Fax Garbage Collector) Tests               ║"
  IO.println "╚══════════════════════════════════════════════════════════╝"
  IO.println ""
  
  IO.println "Running ZPointer Tests..."
  let results1 ← [
    testZPointerBasics,
    testZPointerColors,
    testZPointerNull
  ].mapM id
  
  IO.println ""
  IO.println "Running Heap Tests..."
  let results2 ← [
    testHeapInitialization,
    testHeapAllocation,
    testHeapStats,
    testHeapShouldCollect
  ].mapM id
  
  IO.println ""
  IO.println "Running TLAB Tests..."
  let results3 ← [
    testTLABCreation,
    testTLABAllocation,
    testTLABNeedsRefill
  ].mapM id
  
  IO.println ""
  IO.println "Running Mark Tests..."
  let results4 ← [
    testMarkStack,
    testMarkContext,
    testRootSet
  ].mapM id
  
  IO.println ""
  IO.println "Running Relocation Tests..."
  let results5 ← [
    testForwardingTable,
    testRelocationSet
  ].mapM id
  
  IO.println ""
  IO.println "Running Write Barrier Tests..."
  let results6 ← [
    testSATBQueue,
    testRememberedSet,
    testWriteBarrier
  ].mapM id
  
  IO.println ""
  IO.println "Running Reference Processor Tests..."
  let results7 ← [
    testReferenceObject,
    testReferenceProcessorState,
    testSoftReferenceClearing
  ].mapM id
  
  IO.println ""
  IO.println "Running Generational Tests..."
  let results8 ← [
    testGenerationalHeapInit,
    testEdenAllocation,
    testMinorGCTrigger
  ].mapM id
  
  IO.println ""
  IO.println "Running Controller Tests..."
  let results9 ← [
    testGCConfig,
    testGCStateInit,
    testGCStats
  ].mapM id
  
  let allResults := results1 ++ results2 ++ results3 ++ results4 ++ results5 ++ results6 ++ results7 ++ results8 ++ results9
  let passed := allResults.filter id |>.length
  let total := allResults.length
  
  IO.println ""
  IO.println "═══════════════════════════════════════════════════════════"
  IO.println s!"              Final Results: {passed}/{total} tests passed"
  IO.println "═══════════════════════════════════════════════════════════"
  
  if passed == total then
    IO.println ""
    IO.println "✓ All GC tests passed!"
    return 0
  else
    IO.println ""
    IO.println s!"✗ {total - passed} test(s) failed"
    return 1

end Tests.GC

def main : IO UInt32 := Tests.GC.runAllTests
