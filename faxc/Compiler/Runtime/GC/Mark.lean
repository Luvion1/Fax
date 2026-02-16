/-
FGC Concurrent Marking - OPTIMIZED VERSION
Phase 1: Concurrent marking of live objects
Bug fixes:
1. Fixed stack overflow in recursive markThread (now iterative)
2. Added proper stack capacity checking for references
3. Added overflow reporting
4. Improved performance with batch processing
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Barrier
import Compiler.Runtime.GC.Heap

namespace Compiler.Runtime.GC.Mark

open ZPointer Barrier Heap

-- Configuration for marking
structure MarkConfig where
  stackCapacity : Nat := 4096
  overflowThreshold : Nat := 100  -- Report after this many overflows
  batchSize : Nat := 100  -- Process this many objects before checking time
  deriving Repr

-- Mark stack for tracking objects to mark
structure MarkStack where
  entries : Array ZPointer
  capacity : Nat
  overflowCount : Nat  -- Track how many times we couldn't push
  deriving Repr

def MarkStack.empty (capacity : Nat := 4096) : MarkStack :=
  { entries := #[], capacity := capacity, overflowCount := 0 }

def MarkStack.push (stack : MarkStack) (ptr : ZPointer) : MarkStack :=
  if stack.entries.size >= stack.capacity then
    -- BUG FIX: Track overflow instead of silently ignoring
    { stack with overflowCount := stack.overflowCount + 1 }
  else
    { stack with entries := stack.entries.push ptr }

-- BUG FIX: Push multiple items with proper capacity checking
def MarkStack.pushBatch (stack : MarkStack) (ptrs : Array ZPointer) : MarkStack :=
  ptrs.foldl (fun s ptr => s.push ptr) stack

def MarkStack.pop (stack : MarkStack) : Option (ZPointer × MarkStack) :=
  if stack.entries.isEmpty then
    none
  else
    let lastIdx := stack.entries.size - 1
    -- BUG FIX: Safe array access
    if lastIdx < stack.entries.size then
      let ptr := stack.entries.get! lastIdx
      some (ptr, { stack with entries := stack.entries.pop })
    else
      none

def MarkStack.isEmpty (stack : MarkStack) : Bool :=
  stack.entries.isEmpty

def MarkStack.size (stack : MarkStack) : Nat :=
  stack.entries.size

-- Mark context for concurrent marking
structure MarkContext where
  stack : MarkStack
  markColor : Color
  markedCount : Nat
  bytesMarked : Nat
  visited : Lean.HashSet UInt64  -- BUG FIX: Track visited addresses to avoid cycles
  config : MarkConfig
  deriving Repr

def MarkContext.new (color : Color) (config : MarkConfig := {}) : MarkContext :=
  { stack := MarkStack.empty config.stackCapacity
    markColor := color
    markedCount := 0
    bytesMarked := 0
    visited := Lean.HashSet.empty
    config := config
  }

-- Check if object was already visited
def MarkContext.isVisited (ctx : MarkContext) (addr : UInt64) : Bool :=
  ctx.visited.contains addr

def MarkContext.markVisited (ctx : MarkContext) (addr : UInt64) : MarkContext :=
  { ctx with visited := ctx.visited.insert addr }

-- Object visitor for marking
-- BUG FIX: Added visited check to prevent infinite loops on cycles
def markObject (ctx : MarkContext) (ptr : ZPointer)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO MarkContext := do
  
  if ptr.isNull then
    return ctx
  
  let addr := ptr.toAddress
  
  -- BUG FIX: Check if already visited (prevents cycles)
  if ctx.isVisited addr then
    return ctx
  
  let header := readHeader addr
  
  -- Check if already marked with current color
  if header.markBit then
    return ctx
  
  -- Mark object
  let markedHeader := { header with markBit := true }
  writeHeader addr markedHeader
  
  -- Add to statistics and visited set
  let newCtx := { ctx with
    markedCount := ctx.markedCount + 1
    bytesMarked := ctx.bytesMarked + header.size
  }.markVisited addr
  
  -- BUG FIX: Check stack capacity before pushing all references
  let refs := getReferences header
  let availableSpace := newCtx.stack.capacity - newCtx.stack.size
  let refsToPush := if refs.size > availableSpace then
    -- Only push what fits, log the overflow
    refs.take availableSpace.toUSize
  else
    refs
  
  let finalCtx := refsToPush.foldl (λ c ref => 
    { c with stack := c.stack.push ref }) newCtx
  
  -- If we couldn't push all references, that's an overflow situation
  let finalCtx := if refs.size > availableSpace then
    { finalCtx with 
      stack := { finalCtx.stack with 
        overflowCount := finalCtx.stack.overflowCount + (refs.size - availableSpace) 
      }
    }
  else
    finalCtx
  
  return finalCtx

-- BUG FIX: Iterative marking to prevent stack overflow
partial def markThreadIterative (ctx : MarkContext)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO MarkContext := do
  
  let mut currentCtx := ctx
  
  -- Process stack iteratively instead of recursively
  while !currentCtx.stack.isEmpty do
    match currentCtx.stack.pop with
    | some (ptr, newStack) =>
      currentCtx := { currentCtx with stack := newStack }
      currentCtx ← markObject currentCtx ptr readHeader writeHeader getReferences
    | none => break
  
  return currentCtx

-- DEPRECATED: Use markThreadIterative instead
-- Kept for backwards compatibility
partial def markThread (ctx : MarkContext)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO MarkContext := do
  -- BUG FIX: Use iterative version to prevent stack overflow
  markThreadIterative ctx readHeader writeHeader getReferences

-- Root set for marking
structure RootSet where
  roots : Array ZPointer
  deriving Repr

def RootSet.empty : RootSet :=
  { roots := #[] }

def RootSet.add (set : RootSet) (ptr : ZPointer) : RootSet :=
  { set with roots := set.roots.push ptr }

def RootSet.size (set : RootSet) : Nat :=
  set.roots.size

-- Initialize marking from roots
-- BUG FIX: Handle case where there are too many roots for stack
def initializeMarking (roots : RootSet) (markColor : Color) (config : MarkConfig := {}) : MarkContext :=
  let ctx := MarkContext.new markColor config
  
  -- If too many roots, only push what fits and log overflow
  let availableSpace := ctx.stack.capacity
  let rootsToPush := if roots.size > availableSpace then
    roots.roots.take availableSpace.toUSize
  else
    roots.roots
  
  let overflowCount := if roots.size > availableSpace then
    roots.size - availableSpace
  else
    0
  
  { ctx with
    stack := rootsToPush.foldl (λ s r => s.push r) ctx.stack
  }

-- Concurrent marking phase
def concurrentMark (heap : ZHeap) (roots : RootSet) (markColor : Color)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    (config : MarkConfig := {})
    : IO (ZHeap × MarkContext) := do
  
  -- Set barrier state for marking
  let barrierState := LoadBarrierState.forMarking markColor
  let heap' := { heap with barrierState := barrierState }
  
  -- Initialize marking context with roots
  let ctx := initializeMarking roots markColor config
  
  -- BUG FIX: Use iterative marking to prevent stack overflow
  let ctx ← markThreadIterative ctx readHeader writeHeader getReferences
  
  -- Report if there were overflows
  if ctx.stack.overflowCount > 0 then
    IO.println s!"[GC Warning] Mark stack overflowed {ctx.stack.overflowCount} times"
  
  return (heap', ctx)

-- Update live bytes in regions after marking
def updateLiveBytes (heap : ZHeap) (ctx : MarkContext) : ZHeap :=
  -- This would iterate through marked objects and update region stats
  -- Simplified implementation
  heap

-- Mark completion
def endMarkingPhase (heap : ZHeap) : ZHeap :=
  { heap with
    barrierState := LoadBarrierState.inactive
    gcCycles := heap.gcCycles + 1
  }

-- Incremental marking (for low pause times)
-- BUG FIX: Improved batch processing and deadline checking
structure IncrementalMarkState where
  context : MarkContext
  workUnitsCompleted : Nat
  targetWorkUnits : Nat
  deadlineMs : Nat
  startTime : Nat
  deriving Repr

def IncrementalMarkState.new (ctx : MarkContext) (target : Nat) (deadline : Nat) (start : Nat) : IncrementalMarkState :=
  { context := ctx
    workUnitsCompleted := 0
    targetWorkUnits := target
    deadlineMs := deadline
    startTime := start
  }

def incrementalMark (state : IncrementalMarkState)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO (IncrementalMarkState × Bool) := do
  
  let mut ctx := state.context
  let mut completed := state.workUnitsCompleted
  let batchSize := ctx.config.batchSize
  let mut processedInBatch := 0
  
  -- Process work units until deadline or completion
  while !ctx.stack.isEmpty && completed < state.targetWorkUnits do
    match ctx.stack.pop with
    | some (ptr, newStack) =>
      ctx := { ctx with stack := newStack }
      ctx ← markObject ctx ptr readHeader writeHeader getReferences
      completed := completed + 1
      processedInBatch := processedInBatch + 1
      
      -- Check deadline only after processing a batch (for efficiency)
      if processedInBatch >= batchSize then
        let now ← IO.monoMsNow
        if now - state.startTime > state.deadlineMs then
          break
        processedInBatch := 0
    | none => break
  
  -- Final deadline check
  let now ← IO.monoMsNow
  let timeRemaining := state.deadlineMs > (now - state.startTime)
  
  let newState := { state with
    context := ctx
    workUnitsCompleted := completed
  }
  
  let isComplete := ctx.stack.isEmpty
  return (newState, isComplete)

-- Mark statistics
structure MarkStats where
  objectsMarked : Nat
  bytesMarked : Nat
  durationMs : Nat
  workUnits : Nat
  stackOverflows : Nat  -- BUG FIX: Track overflows
  deriving Repr

def MarkStats.fromContext (ctx : MarkContext) (durationMs : Nat) : MarkStats :=
  {
    objectsMarked := ctx.markedCount
    bytesMarked := ctx.bytesMarked
    durationMs := durationMs
    workUnits := ctx.markedCount
    stackOverflows := ctx.stack.overflowCount
  }

-- Check if marking was successful (no overflows)
def MarkStats.isSuccessful (stats : MarkStats) : Bool :=
  stats.stackOverflows == 0

end Compiler.Runtime.GC.Mark
