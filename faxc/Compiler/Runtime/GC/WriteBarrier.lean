/-
FGC Write Barriers
Write barriers track mutations to maintain heap consistency during concurrent marking.
This is essential for maintaining the tri-color invariant in concurrent GC.
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Barrier
import Compiler.Runtime.GC.Heap

namespace Compiler.Runtime.GC.WriteBarrier

open ZPointer Barrier Heap

-- Write barrier type for different GC phases
inductive WriteBarrierType
  | none              -- No barrier needed
  | preMark           -- SATB (Snapshot At The Beginning) barrier
  | postMark          -- Incremental update barrier
  | generational      -- Generational write barrier (card marking)
  | concurrentCopy    -- During concurrent copying
  deriving Repr, BEq

-- Write barrier state
def WriteBarrierState := WriteBarrierType

-- SATB (Snapshot At The Beginning) queue
-- Used during marking phase to track objects that existed at GC start
structure SATBQueue where
  entries : Array ZPointer
  capacity : Nat := 1024
  overflowCount : Nat := 0
  deriving Repr

def SATBQueue.new : SATBQueue :=
  { entries := #[], capacity := 1024, overflowCount := 0 }

def SATBQueue.enqueue (queue : SATBQueue) (ptr : ZPointer) : SATBQueue × Bool :=
  if queue.entries.size >= queue.capacity then
    ({ queue with overflowCount := queue.overflowCount + 1 }, false)
  else
    ({ queue with entries := queue.entries.push ptr }, true)

def SATBQueue.dequeue (queue : SATBQueue) : Option (ZPointer × SATBQueue) :=
  if queue.entries.isEmpty then
    none
  else
    let ptr := queue.entries.get! 0
    some (ptr, { queue with entries := queue.entries.eraseIdx 0 })

def SATBQueue.isEmpty (queue : SATBQueue) : Bool :=
  queue.entries.isEmpty

def SATBQueue.size (queue : SATBQueue) : Nat :=
  queue.entries.size

-- Remembered Set for generational GC
-- Tracks old-to-young references
structure RememberedSet where
  cards : HashMap Nat (Array ZPointer)  -- Card index -> objects
  cardSize : Nat := 512                 -- Bytes per card
  numCards : Nat := 0
  deriving Repr

def RememberedSet.new (heapSize : Nat) (cardSize : Nat := 512) : RememberedSet :=
  { cards := HashMap.empty
    cardSize := cardSize
    numCards := (heapSize + cardSize - 1) / cardSize
  }

-- Calculate card index for an address
def RememberedSet.cardIndexFor (rs : RememberedSet) (addr : UInt64) : Nat :=
  (addr.toNat / rs.cardSize) % rs.numCards

-- Mark card as dirty (containing old-to-young reference)
def RememberedSet.markCard (rs : RememberedSet) (addr : UInt64) 
    (ptr : ZPointer) : RememberedSet :=
  let idx := rs.cardIndexFor addr
  let current := rs.cards.findD idx #[]
  { rs with cards := rs.cards.insert idx (current.push ptr) }

-- Get all pointers in a card
def RememberedSet.getCard (rs : RememberedSet) (idx : Nat) : Array ZPointer :=
  rs.cards.findD idx #[]

-- Clear a card
def RememberedSet.clearCard (rs : RememberedSet) (idx : Nat) : RememberedSet :=
  { rs with cards := rs.cards.erase idx }

-- SATB Write Barrier
-- Called BEFORE overwriting a reference field
-- Ensures we don't lose objects that were live at GC start
def satbWriteBarrier (queue : SATBQueue) (oldRef : ZPointer) (gcActive : Bool)
    : SATBQueue × Bool :=
  if !gcActive || oldRef.isNull then
    (queue, false)
  else
    -- Enqueue the old reference (it was live at GC start)
    queue.enqueue oldRef

-- Post-write barrier for generational GC
-- Called AFTER writing a reference
-- Marks the card if writing old-to-young reference
def generationalWriteBarrier (rs : RememberedSet) (fieldAddr : UInt64)
    (newRef : ZPointer) (isOldGeneration : Bool) : RememberedSet :=
  if !isOldGeneration || newRef.isNull then
    rs
  else
    -- Mark card as dirty
    rs.markCard fieldAddr newRef

-- Combined write barrier
structure CombinedWriteBarrier where
  satbQueue : SATBQueue
  rememberedSet : RememberedSet
  active : Bool
  gcPhase : GCPhase  -- From Controller
  deriving Repr

def CombinedWriteBarrier.new (heapSize : Nat) : CombinedWriteBarrier :=
  { satbQueue := SATBQueue.new
    rememberedSet := RememberedSet.new heapSize
    active := false
    gcPhase := .idle
  }

-- Main write barrier entry point
def writeBarrier (wb : CombinedWriteBarrier) (fieldAddr : UInt64)
    (oldRef : ZPointer) (newRef : ZPointer)
    (isOldGeneration : Bool := false)
    : CombinedWriteBarrier × Bool :=
  
  if !wb.active then
    (wb, false)
  else
    let mut newWB := wb
    let mut enqueued := false
    
    -- SATB barrier during marking
    match wb.gcPhase with
    | .mark | .markIdle =>
      let (queue, wasEnqueued) := satbWriteBarrier wb.satbQueue oldRef true
      newWB := { newWB with satbQueue := queue }
      enqueued := wasEnqueued
    | _ => pure ()
    
    -- Generational barrier for old-to-young
    if isOldGeneration then
      newWB := { newWB with 
        rememberedSet := generationalWriteBarrier wb.rememberedSet fieldAddr newRef true }
    
    (newWB, enqueued)

-- Array write barrier (optimized for array stores)
def arrayWriteBarrier (wb : CombinedWriteBarrier) (arrayAddr : UInt64)
    (index : Nat) (oldRef : ZPointer) (newRef : ZPointer)
    (isOldGeneration : Bool := false)
    : CombinedWriteBarrier :=
  let fieldAddr := arrayAddr + (index * 8).toUInt64  -- Assuming 8-byte pointers
  let (newWB, _) := writeBarrier wb fieldAddr oldRef newRef isOldGeneration
  newWB

-- Bulk write barrier for array copy
-- More efficient than individual barriers
structure BulkWriteBarrierResult where
  wb : CombinedWriteBarrier
  satbEnqueued : Nat
  cardsMarked : Nat
  deriving Repr

def bulkArrayWriteBarrier (wb : CombinedWriteBarrier) (arrayAddr : UInt64)
    (startIdx : Nat) (count : Nat) (oldRefs : Array ZPointer)
    (newRefs : Array ZPointer) (isOldGeneration : Bool := false)
    : BulkWriteBarrierResult :=
  
  let mut resultWB := wb
  let mut satbCount := 0
  
  -- Process SATB queue in batch
  if wb.active && (wb.gcPhase == .mark || wb.gcPhase == .markIdle) then
    for oldRef in oldRefs do
      if !oldRef.isNull then
        let (queue, enqueued) := satbWriteBarrier resultWB.satbQueue oldRef true
        resultWB := { resultWB with satbQueue := queue }
        if enqueued then
          satbCount := satbCount + 1
  
  -- Process card marking
  let mut cardCount := 0
  if isOldGeneration then
    let cardIdx := wb.rememberedSet.cardIndexFor arrayAddr
    resultWB := { resultWB with 
      rememberedSet := { resultWB.rememberedSet with 
        cards := resultWB.rememberedSet.cards.insert cardIdx newRefs } }
    cardCount := 1
  
  { wb := resultWB, satbEnqueued := satbCount, cardsMarked := cardCount }

-- Write barrier statistics
structure WriteBarrierStats where
  totalBarriers : Nat
  satbEnqueued : Nat
  cardsMarked : Nat
  queueOverflows : Nat
  deriving Repr

def WriteBarrierStats.empty : WriteBarrierStats :=
  { totalBarriers := 0
    satbEnqueued := 0
    cardsMarked := 0
    queueOverflows := 0
  }

def WriteBarrierStats.recordBarrier (stats : WriteBarrierStats)
    (satbEnqueued : Bool) (cardMarked : Bool) (overflow : Bool) : WriteBarrierStats :=
  { stats with
    totalBarriers := stats.totalBarriers + 1
    satbEnqueued := if satbEnqueued then stats.satbEnqueued + 1 else stats.satbEnqueued
    cardsMarked := if cardMarked then stats.cardsMarked + 1 else stats.cardsMarked
    queueOverflows := if overflow then stats.queueOverflows + 1 else stats.queueOverflows
  }

-- Process SATB queue (called by GC thread)
def processSATBQueue (queue : SATBQueue) (markFn : ZPointer → IO Unit)
    (maxItems : Nat := 100) : IO SATBQueue := do
  
  let mut remaining := queue
  let mut processed := 0
  
  while !remaining.isEmpty && processed < maxItems do
    match remaining.dequeue with
    | some (ptr, newQueue) =>
      remaining := newQueue
      processed := processed + 1
      -- Mark the object (it's live from SATB perspective)
      if !ptr.isNull then
        markFn ptr
    | none => break
  
  return remaining

-- Clear remembered set (called after young GC)
def clearRememberedSet (rs : RememberedSet) : RememberedSet :=
  { rs with cards := HashMap.empty }

-- Get dirty cards for scanning
def getDirtyCards (rs : RememberedSet) : List Nat :=
  rs.cards.toList.map (·.1)

-- Concurrent modification buffer
-- For high-contention scenarios
structure ModificationBuffer where
  buffer : Array (UInt64 × ZPointer × ZPointer)  -- (fieldAddr, oldRef, newRef)
  capacity : Nat := 256
  deriving Repr

def ModificationBuffer.new : ModificationBuffer :=
  { buffer := #[], capacity := 256 }

def ModificationBuffer.isFull (buf : ModificationBuffer) : Bool :=
  buf.buffer.size >= buf.capacity

def ModificationBuffer.add (buf : ModificationBuffer) 
    (fieldAddr : UInt64) (oldRef : ZPointer) (newRef : ZPointer)
    : ModificationBuffer × Bool :=
  if buf.isFull then
    (buf, false)
  else
    ({ buf with buffer := buf.buffer.push (fieldAddr, oldRef, newRef) }, true)

def ModificationBuffer.flush (buf : ModificationBuffer) 
    (processFn : Array (UInt64 × ZPointer × ZPointer) → IO Unit)
    : IO ModificationBuffer := do
  processFn buf.buffer
  return { buf with buffer := #[] }

-- Thread-local write barrier state
structure ThreadLocalWriteBarrier where
  satbBuffer : ModificationBuffer
  stats : WriteBarrierStats
  threadId : Nat
  deriving Repr

def ThreadLocalWriteBarrier.new (id : Nat) : ThreadLocalWriteBarrier :=
  { satbBuffer := ModificationBuffer.new
    stats := WriteBarrierStats.empty
    threadId := id
  }

-- Global write barrier coordinator
structure WriteBarrierCoordinator where
  threadBarriers : Array ThreadLocalWriteBarrier
  globalSATBQueue : SATBQueue
  globalRememberedSet : RememberedSet
  active : Bool
  deriving Repr

def WriteBarrierCoordinator.new (numThreads : Nat) (heapSize : Nat) 
    : WriteBarrierCoordinator :=
  { threadBarriers := Array.range numThreads |>.map ThreadLocalWriteBarrier.new
    globalSATBQueue := SATBQueue.new
    globalRememberedSet := RememberedSet.new heapSize
    active := false
  }

-- Flush thread-local buffers to global state
def flushThreadBuffers (coord : WriteBarrierCoordinator) (threadId : Nat)
    : WriteBarrierCoordinator :=
  match coord.threadBarriers.get? threadId with
  | some local =>
    -- Merge local SATB queue with global
    let globalQueue := local.satbBuffer.buffer.foldl 
      (λ q (_, oldRef, _) => 
        let (newQ, _) := q.enqueue oldRef
        newQ) 
      coord.globalSATBQueue
    
    { coord with 
      globalSATBQueue := globalQueue
      threadBarriers := coord.threadBarriers.set! threadId 
        { local with satbBuffer := ModificationBuffer.new } }
  | none => coord

end Compiler.Runtime.GC.WriteBarrier

-- GCPhase definition for reference
inductive GCPhase
  | idle
  | mark
  | markIdle
  | relocate
  | relocateIdle
  | cleanup
  deriving Repr, BEq

namespace Compiler.Runtime.GC.WriteBarrier

-- Activate write barriers for a phase
def activateForPhase (coord : WriteBarrierCoordinator) (phase : GCPhase)
    : WriteBarrierCoordinator :=
  { coord with active := (phase == .mark || phase == .markIdle) }

-- Deactivate write barriers
def deactivate (coord : WriteBarrierCoordinator) : WriteBarrierCoordinator :=
  { coord with active := false }

end Compiler.Runtime.GC.WriteBarrier
