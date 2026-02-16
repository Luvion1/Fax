/-
FGC Main Controller
Orchestrates the phases of FGC: Mark, Relocate, and Cleanup
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Barrier
import Compiler.Runtime.GC.Heap
import Compiler.Runtime.GC.Mark
import Compiler.Runtime.GC.Relocate

namespace Compiler.Runtime.GC.Controller

open ZPointer Barrier Heap Mark Relocate

-- FGC Phase
inductive GCPhase
  | idle           -- No GC running
  | mark           -- Concurrent marking
  | markIdle       -- Pause for marking completion
  | relocate       -- Concurrent relocation
  | relocateIdle   -- Pause for relocation completion
  | cleanup        -- Cleanup phase
  deriving Repr, BEq

-- GC Configuration
structure GCConfig where
  maxPauseMs : Nat := 10              -- Target max pause time (10ms for sub-millisecond)
  concurrencyLevel : Nat := 4         -- Number of concurrent GC threads
  triggerHeapUsage : Float := 0.75    -- Trigger GC at 75% heap usage
  useGenerational : Bool := true      -- Use generational collection
  targetThroughput : Float := 0.95    -- Target application throughput (95%)
  deriving Repr

def GCConfig.default : GCConfig :=
  { maxPauseMs := 10
    concurrencyLevel := 4
    triggerHeapUsage := 0.75
    useGenerational := true
    targetThroughput := 0.95
  }

-- GC State Machine
structure GCState where
  phase : GCPhase
  heap : ZHeap
  config : GCConfig
  
  -- Marking state
  markContext : Option MarkContext
  markColor : Color
  
  -- Relocation state
  relocateContext : Option RelocateContext
  
  -- Statistics
  lastGCTimeMs : Nat
  totalGCTimeMs : Nat
  gcCount : Nat
  
  -- Performance metrics
  pauseTimesMs : Array Nat
  allocationRate : Float  -- bytes/ms
  deriving Repr

def GCState.init (heap : ZHeap) (config : GCConfig := GCConfig.default) : IO GCState :=
  let now ← IO.monoMsNow
  return {
    phase := .idle
    heap := heap
    config := config
    markContext := none
    markColor := .marked0
    relocateContext := none
    lastGCTimeMs := now
    totalGCTimeMs := 0
    gcCount := 0
    pauseTimesMs := #[]
    allocationRate := 0.0
  }

-- Check if GC should be triggered
def shouldStartGC (state : GCState) : Bool :=
  if state.phase != .idle then
    false
  else
    let stats := state.heap.stats
    let usageRatio := stats.usedBytes.toFloat / stats.totalBytes.toFloat
    usageRatio > state.config.triggerHeapUsage

-- Start marking phase
def startMarking (state : GCState) (roots : RootSet) 
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO GCState := do
  
  let startTime ← IO.monoMsNow
  
  -- Toggle mark color between cycles
  let markColor := if state.markColor == .marked0 then .marked1 else .marked0
  
  -- Start concurrent marking
  let (heap', ctx) ← concurrentMark state.heap roots markColor
    readHeader writeHeader getReferences
  
  let endTime ← IO.monoMsNow
  let pauseTime := endTime - startTime
  
  return { state with
    phase := .mark
    heap := heap'
    markContext := some ctx
    markColor := markColor
    pauseTimesMs := state.pauseTimesMs.push pauseTime
  }

-- Continue marking incrementally
def continueMarking (state : GCState) (deadlineMs : Nat)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO (GCState × Bool) := do
  
  match state.markContext with
  | none => return (state, true)
  | some ctx =>
    let incState : IncrementalMarkState := {
      context := ctx
      workUnitsCompleted := 0
      targetWorkUnits := 1000
      deadlineMs := deadlineMs
    }
    
    let (newIncState, isComplete) ← incrementalMark incState
      readHeader writeHeader getReferences
    
    let newState := { state with
      markContext := some newIncState.context
    }
    
    return (newState, isComplete)

-- Finish marking phase
def finishMarking (state : GCState) : IO GCState := do
  -- Update region live bytes
  let heap' := match state.markContext with
    | some ctx => updateLiveBytes state.heap ctx
    | none => state.heap
  
  return { state with
    phase := .markIdle
    heap := heap'
  }

-- Start relocation phase
def startRelocation (state : GCState)
    (getObjectsInRegion : ZRegion → Array ZPointer)
    (readHeader : UInt64 → ObjectHeader)
    (readBytes : UInt64 → Nat → ByteArray)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (writeBytes : UInt64 → ByteArray → IO Unit)
    : IO GCState := do
  
  let startTime ← IO.monoMsNow
  
  -- Start concurrent relocation
  let (heap', ctx) ← concurrentRelocation state.heap
    getObjectsInRegion readHeader readBytes writeHeader writeBytes
  
  let endTime ← IO.monoMsNow
  let pauseTime := endTime - startTime
  
  return { state with
    phase := .relocate
    heap := heap'
    relocateContext := some ctx
    pauseTimesMs := state.pauseTimesMs.push pauseTime
  }

-- Continue relocation
def continueRelocation (state : GCState) : IO GCState := do
  -- Relocation happens in the background via load barriers
  -- This function just checks if we're done
  match state.relocateContext with
  | none => return { state with phase := .relocateIdle }
  | some _ => return state

-- Finish relocation phase
def finishRelocation (state : GCState) : IO GCState := do
  -- Free relocated regions
  let heap' := match state.relocateContext with
    | some ctx => freeRelocatedRegions state.heap ctx
    | none => state.heap
  
  -- End relocation phase
  let heap'' := endRelocationPhase heap'
  
  return { state with
    phase := .idle
    heap := heap''
    relocateContext := none
    gcCount := state.gcCount + 1
  }

-- Run one complete GC cycle
def runGCFull (state : GCState) (roots : RootSet)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    (getObjectsInRegion : ZRegion → Array ZPointer)
    (readBytes : UInt64 → Nat → ByteArray)
    (writeBytes : UInt64 → ByteArray → IO Unit)
    : IO GCState := do
  
  IO.println "Starting full GC cycle..."
  let startTime ← IO.monoMsNow
  
  -- Phase 1: Marking
  IO.println "  Phase 1: Marking..."
  let mut state' ← startMarking state roots readHeader writeHeader getReferences
  
  -- Wait for marking to complete (in real implementation, do incrementally)
  let (state'', isComplete) ← continueMarking state' 1000
    readHeader writeHeader getReferences
  
  state' ← finishMarking state''
  IO.println s!"    Marked {state'.markContext.map (λ c => c.markedCount) |>.getD 0} objects"
  
  -- Phase 2: Relocation
  IO.println "  Phase 2: Relocation..."
  state' ← startRelocation state'
    getObjectsInRegion readHeader readBytes writeHeader writeBytes
  
  state' ← continueRelocation state'
  state' ← finishRelocation state'
  
  let endTime ← IO.monoMsNow
  let totalTime := endTime - startTime
  
  IO.println s!"GC cycle completed in {totalTime}ms"
  
  return { state' with
    lastGCTimeMs := endTime
    totalGCTimeMs := state'.totalGCTimeMs + totalTime
  }

-- Background GC thread
def gcBackgroundThread (stateRef : IO.Ref GCState)
    (rootsProvider : IO RootSet)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    (getObjectsInRegion : ZRegion → Array ZPointer)
    (readBytes : UInt64 → Nat → ByteArray)
    (writeBytes : UInt64 → ByteArray → IO Unit)
    : IO Unit := do
  
  while true do
    let state ← stateRef.get
    
    if shouldStartGC state then
      let roots ← rootsProvider
      let newState ← runGCFull state roots
        readHeader writeHeader getReferences
        getObjectsInRegion readBytes writeBytes
      stateRef.set newState
    
    -- Sleep before next check
    IO.sleep 100

-- Allocation with GC trigger
def allocateWithGC (stateRef : IO.Ref GCState) (size : Nat) (typeId : UInt32 := 0)
    : IO (Option ZPointer) := do
  
  let mut state ← stateRef.get
  
  -- Try to allocate
  match state.heap.allocate size typeId with
  | some (ptr, heap') =>
    state := { state with heap := heap' }
    stateRef.set state
    return some ptr
    
  | none =>
    -- Allocation failed, trigger GC
    IO.println "Allocation failed, triggering GC..."
    -- Would run GC here
    -- For now, return none
    return none

-- GC Statistics
structure GCStats where
  gcCount : Nat
  totalGCTimeMs : Nat
  avgPauseTimeMs : Float
  maxPauseTimeMs : Nat
  heapUsage : Float
  allocationRate : Float
  deriving Repr

def GCState.getStats (state : GCState) : GCStats :=
  let pauseTimes := state.pauseTimesMs
  let avgPause := if pauseTimes.isEmpty then 0.0
    else pauseTimes.foldl (λ acc t => acc + t.toFloat) 0.0 / pauseTimes.size.toFloat
  let maxPause := if pauseTimes.isEmpty then 0
    else pauseTimes.foldl (λ acc t => max acc t) 0
  
  let heapStats := state.heap.stats
  let usage := heapStats.usedBytes.toFloat / heapStats.totalBytes.toFloat
  
  {
    gcCount := state.gcCount
    totalGCTimeMs := state.totalGCTimeMs
    avgPauseTimeMs := avgPause
    maxPauseTimeMs := maxPause
    heapUsage := usage
    allocationRate := state.allocationRate
  }

end Compiler.Runtime.GC.Controller
