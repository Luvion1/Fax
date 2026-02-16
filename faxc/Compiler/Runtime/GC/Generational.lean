/-
FGC Generational Collection
Implements generational garbage collection with young and old generations.
Young generation uses copying collector, old generation uses concurrent marking.
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Heap
import Compiler.Runtime.GC.TLAB
import Compiler.Runtime.GC.WriteBarrier
import Compiler.Runtime.GC.Mark

namespace Compiler.Runtime.GC.Generational

open ZPointer Heap TLAB WriteBarrier Mark

-- Generation types
inductive Generation
  | young        -- Young generation (eden + survivors)
  | old          -- Old generation (tenured objects)
  deriving Repr, BEq

-- Age tracking for objects
def MAX_AGE : UInt8 := 15

-- Generational heap layout
structure GenerationalHeap where
  -- Young generation
  edenStart : UInt64
  edenSize : Nat
  edenTop : UInt64
  
  -- Survivor spaces (S0 and S1)
  survivor0Start : UInt64
  survivor0Size : Nat
  survivor0Top : UInt64
  
  survivor1Start : UInt64
  survivor1Size : Nat
  survivor1Top : UInt64
  
  -- Old generation
  oldRegions : Array ZRegion
  
  -- Current survivor space
  fromSurvivor : Bool  -- false = S0, true = S1
  
  -- Configuration
  promotionThreshold : UInt8 := 3  -- Age to promote to old gen
  
  deriving Repr

-- Young generation collector (minor GC)
structure YoungCollector where
  heap : GenerationalHeap
  markContext : Option MarkContext
  bytesPromoted : Nat
  objectsPromoted : Nat
  deriving Repr

-- Initialize generational heap
def GenerationalHeap.init (totalYoungSize : Nat) (totalOldSize : Nat)
    : GenerationalHeap :=
  
  let edenRatio : Float := 0.8  -- 80% for eden
  let survivorRatio : Float := 0.1  -- 10% each for S0 and S1
  
  let edenSize := (totalYoungSize.toFloat * edenRatio).toUInt64.toNat
  let survivorSize := (totalYoungSize.toFloat * survivorRatio).toUInt64.toNat
  
  let baseAddr : UInt64 := 0x100000000
  
  { edenStart := baseAddr
    edenSize := edenSize
    edenTop := baseAddr
    
    survivor0Start := baseAddr + edenSize.toUInt64
    survivor0Size := survivorSize
    survivor0Top := baseAddr + edenSize.toUInt64
    
    survivor1Start := baseAddr + edenSize.toUInt64 + survivorSize.toUInt64
    survivor1Size := survivorSize
    survivor1Top := baseAddr + edenSize.toUInt64 + survivorSize.toUInt64
    
    oldRegions := #[]
    fromSurvivor := false
    promotionThreshold := 3
  }

-- Allocate in eden space
def GenerationalHeap.allocateEden (heap : GenerationalHeap) (bytes : Nat)
    (typeId : UInt32) : Option (ZPointer × GenerationalHeap) :=
  
  let alignedBytes := alignUp bytes 8
  let newTop := heap.edenTop + alignedBytes.toUInt64
  
  if newTop <= heap.edenStart + heap.edenSize.toUInt64 then
    let ptr := ZPointer.fromAddress heap.edenTop .remapped
    let newHeap := { heap with edenTop := newTop }
    some (ptr, newHeap)
  else
    none

-- Allocate in survivor space
def GenerationalHeap.allocateSurvivor (heap : GenerationalHeap) (bytes : Nat)
    (toSpace : Bool) : Option (ZPointer × GenerationalHeap) :=
  
  let alignedBytes := alignUp bytes 8
  
  if toSpace then
    -- Allocate in to-space (survivor1 if fromSurvivor=false, else survivor0)
    let (start, top, size) := if heap.fromSurvivor then
      (heap.survivor0Start, heap.survivor0Top, heap.survivor0Size)
    else
      (heap.survivor1Start, heap.survivor1Top, heap.survivor1Size)
    
    let newTop := top + alignedBytes.toUInt64
    if newTop <= start + size.toUInt64 then
      let ptr := ZPointer.fromAddress top .remapped
      let newHeap := if heap.fromSurvivor then
        { heap with survivor0Top := newTop }
      else
        { heap with survivor1Top := newTop }
      some (ptr, newHeap)
    else
      none
  else
    -- Allocate in from-space (shouldn't happen during normal operation)
    none

-- Check if minor GC is needed
def GenerationalHeap.needsMinorGC (heap : GenerationalHeap) : Bool :=
  let edenUsed := (heap.edenTop - heap.edenStart).toNat
  let edenUsage := edenUsed.toFloat / heap.edenSize.toFloat
  edenUsage > 0.8  -- Trigger GC when 80% full

-- Get from-space and to-space
def GenerationalHeap.getSurvivorSpaces (heap : GenerationalHeap)
    : (UInt64 × UInt64 × Nat) × (UInt64 × UInt64 × Nat) :=
  
  if heap.fromSurvivor then
    -- S1 is from-space, S0 is to-space
    ((heap.survivor1Start, heap.survivor1Top, heap.survivor1Size),
     (heap.survivor0Start, heap.survivor0Top, heap.survivor0Size))
  else
    -- S0 is from-space, S1 is to-space
    ((heap.survivor0Start, heap.survivor0Top, heap.survivor0Size),
     (heap.survivor1Start, heap.survivor1Top, heap.survivor1Size))

-- Minor GC: Collect young generation
-- Uses copying collector between survivor spaces
def YoungCollector.minorGC (collector : YoungCollector) (roots : RootSet)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO (YoungCollector × GenerationalHeap) := do
  
  let heap := collector.heap
  
  -- Get from-space and to-space
  let ((fromStart, fromTop, _), (toStart, toTop, toSize)) := heap.getSurvivorSpaces
  
  -- Initialize collector state
  let mut promoted := 0
  let mut promotedCount := 0
  let mut newToTop := toStart
  
  -- Copy live objects from eden and from-survivor to to-survivor
  -- This is simplified; real implementation would:
  -- 1. Mark live objects from roots
  -- 2. Copy surviving objects to to-space
  -- 3. Promote aged objects to old generation
  -- 4. Clear eden and from-survivor
  
  -- For each root...
  for root in roots.roots do
    if !root.isNull then
      let header := readHeader root.toAddress
      
      -- Check age
      if header.age >= heap.promotionThreshold then
        -- Promote to old generation
        promoted := promoted + header.size
        promotedCount := promotedCount + 1
      else
        -- Copy to survivor space
        let objSize := alignUp header.size 8
        if newToTop + objSize.toUInt64 <= toStart + toSize.toUInt64 then
          -- Copy object (simplified)
          newToTop := newToTop + objSize.toUInt64
      
  -- Switch survivor spaces
  let newHeap := { heap with 
    fromSurvivor := !heap.fromSurvivor
    edenTop := heap.edenStart  -- Clear eden
    survivor0Top := if heap.fromSurvivor then heap.survivor0Start else newToTop
    survivor1Top := if heap.fromSurvivor then newToTop else heap.survivor1Start
  }
  
  let newCollector := { collector with 
    heap := newHeap
    bytesPromoted := promoted
    objectsPromoted := promotedCount
  }
  
  return (newCollector, newHeap)

-- Age increment
def ObjectHeader.incrementAge (header : ObjectHeader) : ObjectHeader :=
  { header with age := min (header.age + 1) MAX_AGE }

-- Copy object to survivor space
def copyToSurvivor (heap : GenerationalHeap) (objPtr : ZPointer)
    (header : ObjectHeader)
    (toSpaceStart : UInt64) (toSpaceTop : UInt64)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    : IO (Option (ZPointer × UInt64)) := do
  
  let objSize := alignUp header.size 8
  let newTop := toSpaceTop + objSize.toUInt64
  
  -- Check if fits in to-space
  let toSpaceLimit := if heap.fromSurvivor then
    heap.survivor0Start + heap.survivor0Size.toUInt64
  else
    heap.survivor1Start + heap.survivor1Size.toUInt64
  
  if newTop > toSpaceLimit then
    return none  -- Survivor space full, must promote
  
  -- Copy object (simplified - real impl would copy bytes)
  let newPtr := ZPointer.fromAddress toSpaceTop .remapped
  
  -- Write header with incremented age
  let newHeader := header.incrementAge
  writeHeader toSpaceTop newHeader
  
  return some (newPtr, newTop)

-- Tenuring threshold calculation
-- Dynamically adjust promotion threshold based on survivor occupancy
def calculateTenuringThreshold (heap : GenerationalHeap) : UInt8 :=
  let survivor0Used := (heap.survivor0Top - heap.survivor0Start).toNat
  let survivor1Used := (heap.survivor1Top - heap.survivor1Start).toNat
  let survivorUsed := max survivor0Used survivor1Used
  let survivorUsage := survivorUsed.toFloat / heap.survivor0Size.toFloat
  
  if survivorUsage > 0.5 then
    -- Survivor space getting full, promote sooner
    max 1 (heap.promotionThreshold - 1)
  else if survivorUsage < 0.1 then
    -- Survivor space empty, promote later
    min MAX_AGE (heap.promotionThreshold + 1)
  else
    heap.promotionThreshold

-- Eden space allocation with GC trigger
def allocateInEdenWithGC (heap : GenerationalHeap) (roots : RootSet)
    (bytes : Nat) (typeId : UInt32)
    (readHeader : UInt64 → ObjectHeader)
    (writeHeader : UInt64 → ObjectHeader → IO Unit)
    (getReferences : ObjectHeader → Array ZPointer)
    : IO (Option (ZPointer × GenerationalHeap)) := do
  
  -- Try allocation first
  match heap.allocateEden bytes typeId with
  | some result => return some result
  | none =>
    -- Eden full, trigger minor GC
    let collector : YoungCollector := {
      heap := heap
      markContext := none
      bytesPromoted := 0
      objectsPromoted := 0
    }
    
    let (newCollector, newHeap) ← collector.minorGC roots
      readHeader writeHeader getReferences
    
    -- Retry allocation
    newHeap.allocateEden bytes typeId

-- Remembered set management for inter-generational references
structure GenerationalRememberedSet where
  cards : HashMap Nat (Array ZPointer)  -- Card index -> old objects
  cardSize : Nat := 512
  numCards : Nat
  deriving Repr

def GenerationalRememberedSet.new (oldGenSize : Nat) (cardSize : Nat := 512)
    : GenerationalRememberedSet :=
  { cards := HashMap.empty
    cardSize := cardSize
    numCards := (oldGenSize + cardSize - 1) / cardSize
  }

-- Mark card as dirty (contains reference to young generation)
def GenerationalRememberedSet.markCard (rs : GenerationalRememberedSet)
    (addr : UInt64) (obj : ZPointer) : GenerationalRememberedSet :=
  let cardIdx := (addr.toNat / rs.cardSize) % rs.numCards
  let current := rs.cards.findD cardIdx #[]
  { rs with cards := rs.cards.insert cardIdx (current.push obj) }

-- Get dirty cards for scanning during minor GC
def GenerationalRememberedSet.getDirtyCards (rs : GenerationalRememberedSet)
    : List (Nat × Array ZPointer) :=
  rs.cards.toList

-- Clear card after processing
def GenerationalRememberedSet.clearCard (rs : GenerationalRememberedSet)
    (cardIdx : Nat) : GenerationalRememberedSet :=
  { rs with cards := rs.cards.erase cardIdx }

-- Major GC trigger
def shouldRunMajorGC (heap : GenerationalHeap) (oldGenUsage : Float) : Bool :=
  oldGenUsage > 0.7  -- Trigger major GC when old gen > 70% full

-- Generational GC statistics
structure GenerationalStats where
  -- Young generation
  minorGCCount : Nat
  minorGCTimeMs : Nat
  edenAllocated : Nat
  edenWasted : Nat
  survivor0Used : Nat
  survivor1Used : Nat
  
  -- Promotions
  objectsPromoted : Nat
  bytesPromoted : Nat
  
  -- Old generation
  majorGCCount : Nat
  majorGCTimeMs : Nat
  oldGenUsed : Nat
  oldGenTotal : Nat
  
  -- Remembered set
  dirtyCards : Nat
  deriving Repr

def GenerationalStats.new : GenerationalStats :=
  { minorGCCount := 0
    minorGCTimeMs := 0
    edenAllocated := 0
    edenWasted := 0
    survivor0Used := 0
    survivor1Used := 0
    objectsPromoted := 0
    bytesPromoted := 0
    majorGCCount := 0
    majorGCTimeMs := 0
    oldGenUsed := 0
    oldGenTotal := 0
    dirtyCards := 0
  }

-- Allocation alignment helper
def alignUp (value : Nat) (alignment : Nat) : Nat :=
  let remainder := value % alignment
  if remainder == 0 then value else value + (alignment - remainder)

end Compiler.Runtime.GC.Generational
