/-
FGC Heap Management
Manages the heap as a collection of regions with concurrent allocation
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Barrier

namespace Compiler.Runtime.GC.Heap

open ZPointer Barrier

-- FGC Heap configuration
structure HeapConfig where
  minHeapSize : Nat := 8 * 1024 * 1024           -- 8MB minimum
  maxHeapSize : Nat := 16 * 1024 * 1024 * 1024   -- 16GB maximum
  regionSize : Nat := 2 * 1024 * 1024            -- 2MB regions
  concurrentGCThreads : Nat := 4                 -- Parallel GC threads
  softMaxHeapSize : Nat := 0                     -- Soft limit (0 = none)
  preTouchMemory : Bool := false                 -- Pre-touch heap pages
  deriving Repr

def HeapConfig.default : HeapConfig :=
  { minHeapSize := 8 * 1024 * 1024
    maxHeapSize := 1024 * 1024 * 1024  -- 1GB default for testing
    regionSize := 2 * 1024 * 1024
    concurrentGCThreads := 4
    softMaxHeapSize := 0
    preTouchMemory := false
  }

-- Calculate number of regions for given heap size
-- BUG FIX: Added protection against division by zero
def HeapConfig.numRegions (config : HeapConfig) : Nat :=
  if config.regionSize == 0 then
    0  -- Prevent division by zero
  else
    config.maxHeapSize / config.regionSize

-- BUG FIX: Safe region access with bounds checking
def ZHeap.getRegion (heap : ZHeap) (idx : Nat) : Option ZRegion :=
  if idx < heap.regions.size then
    some (heap.regions.get! idx)
  else
    none

-- FGC Heap State
structure ZHeap where
  config : HeapConfig
  regions : Array ZRegion
  barrierState : LoadBarrierState
  
  -- Region management
  freeRegions : List Nat        -- Indices of free regions
  usedRegions : List Nat        -- Indices of used regions
  
  -- Allocation state
  currentAllocationRegion : Option Nat
  allocationPointer : UInt64
  
  -- Statistics
  totalAllocated : Nat
  totalFreed : Nat
  gcCycles : Nat
  
  deriving Repr

-- Initialize heap
def ZHeap.init (config : HeapConfig := HeapConfig.default) : IO ZHeap := do
  let numRegions := config.numRegions
  let mut regions : Array ZRegion := #[]
  let mut freeList : List Nat := []
  
  -- Reserve virtual memory space
  let baseAddr : UInt64 := 0x100000000  -- Start at 4GB boundary
  
  -- Initialize regions
  for i in [:numRegions] do
    let regionStart := baseAddr + (i * config.regionSize).toUInt64
    let region := ZRegion.new regionStart config.regionSize
    regions := regions.push region
    freeList := i :: freeList
  
  return {
    config := config
    regions := regions
    barrierState := LoadBarrierState.inactive
    freeRegions := freeList
    usedRegions := []
    currentAllocationRegion := none
    allocationPointer := 0
    totalAllocated := 0
    totalFreed := 0
    gcCycles := 0
  }

-- Find a region with enough space
def ZHeap.findRegionWithSpace (heap : ZHeap) (bytes : Nat) : Option Nat :=
  heap.regions.findIdx? (λ r => r.hasSpace bytes)

-- Allocate a new region
def ZHeap.allocateRegion (heap : ZHeap) (regType : RegionType) 
    : Option (Nat × ZHeap) :=
  match heap.freeRegions with
  | [] => none
  | idx :: rest =>
    -- BUG FIX: Added bounds check
    match heap.getRegion idx with
    | some region =>
      let newRegion := { region with type := regType, state := .used }
      let newRegions := heap.regions.set! idx newRegion
      some (idx, { heap with
        regions := newRegions
        freeRegions := rest
        usedRegions := idx :: heap.usedRegions
      })
    | none => 
      -- Invalid region index, skip it
      allocateRegion { heap with freeRegions := rest } regType

-- Small object allocation (< 256KB)
def ZHeap.allocateSmall (heap : ZHeap) (bytes : Nat) (typeId : UInt32)
    : Option (ZPointer × ZHeap) :=
  
  -- Try current allocation region first
  match heap.currentAllocationRegion with
  | some regionIdx =>
    -- BUG FIX: Added bounds check
    match heap.getRegion regionIdx with
    | some region =>
      match region.allocate bytes with
      | some (addr, newRegion) =>
        let newRegions := heap.regions.set! regionIdx newRegion
        let ptr := ZPointer.fromAddress addr .remapped
        some (ptr, { heap with
          regions := newRegions
          totalAllocated := heap.totalAllocated + bytes
        })
      | none =>
        -- Current region full, allocate new one
        heap.allocateNewRegion bytes typeId
    | none =>
      -- Invalid region index, clear it and allocate new
      heap.allocateNewRegion bytes typeId
  | none =>
    heap.allocateNewRegion bytes typeId

-- Allocate new region for small objects
def ZHeap.allocateNewRegion (heap : ZHeap) (bytes : Nat) (typeId : UInt32)
    : Option (ZPointer × ZHeap) :=
  match heap.allocateRegion .small with
  | some (idx, heap') =>
    let region := heap'.regions.get! idx
    match region.allocate bytes with
    | some (addr, newRegion) =>
      let newRegions := heap'.regions.set! idx newRegion
      let ptr := ZPointer.fromAddress addr .remapped
      some (ptr, { heap' with
        regions := newRegions
        currentAllocationRegion := some idx
        totalAllocated := heap'.totalAllocated + bytes
      })
    | none => none
  | none => none

-- Medium object allocation (256KB - 4MB)
def ZHeap.allocateMedium (heap : ZHeap) (bytes : Nat) (typeId : UInt32)
    : Option (ZPointer × ZHeap) :=
  match heap.allocateRegion .medium with
  | some (idx, heap') =>
    let region := heap'.regions.get! idx
    let usedRegion := { region with used := bytes }
    let newRegions := heap'.regions.set! idx usedRegion
    let ptr := ZPointer.fromAddress region.startAddress .remapped
    some (ptr, { heap' with
      regions := newRegions
      totalAllocated := heap'.totalAllocated + bytes
    })
  | none => none

-- Large object allocation (> 4MB)
def ZHeap.allocateLarge (heap : ZHeap) (bytes : Nat) (typeId : UInt32)
    : Option (ZPointer × ZHeap) :=
  -- For large objects, use contiguous regions or allocate separately
  match heap.allocateRegion .large with
  | some (idx, heap') =>
    let region := heap'.regions.get! idx
    let regionSize := max bytes heap.config.regionSize
    let sizedRegion := { region with size := regionSize, used := bytes }
    let newRegions := heap'.regions.set! idx sizedRegion
    let ptr := ZPointer.fromAddress region.startAddress .remapped
    some (ptr, { heap' with
      regions := newRegions
      totalAllocated := heap'.totalAllocated + bytes
    })
  | none => none

-- General allocation
def ZHeap.allocate (heap : ZHeap) (bytes : Nat) (typeId : UInt32 := 0)
    : Option (ZPointer × ZHeap) :=
  
  if bytes <= 256 * 1024 then
    heap.allocateSmall bytes typeId
  else if bytes <= 4 * 1024 * 1024 then
    heap.allocateMedium bytes typeId
  else
    heap.allocateLarge bytes typeId

-- Free a region
def ZHeap.freeRegion (heap : ZHeap) (regionIdx : Nat) : ZHeap :=
  let region := heap.regions.get! regionIdx
  let freedRegion := { region with 
    used := 0
    liveBytes := 0
    state := .empty
    age := 0
  }
  let newRegions := heap.regions.set! regionIdx freedRegion
  
  { heap with
    regions := newRegions
    usedRegions := heap.usedRegions.filter (λ i => i != regionIdx)
    freeRegions := regionIdx :: heap.freeRegions
    totalFreed := heap.totalFreed + region.used
  }

-- Get heap statistics
structure HeapStats where
  totalRegions : Nat
  usedRegions : Nat
  freeRegions : Nat
  totalBytes : Nat
  usedBytes : Nat
  freeBytes : Nat
  gcCycles : Nat
  deriving Repr

def ZHeap.stats (heap : ZHeap) : HeapStats :=
  let usedBytes := heap.regions.foldl (λ acc r => acc + r.used) 0
  {
    totalRegions := heap.regions.size
    usedRegions := heap.usedRegions.length
    freeRegions := heap.freeRegions.length
    totalBytes := heap.config.maxHeapSize
    usedBytes := usedBytes
    freeBytes := heap.config.maxHeapSize - usedBytes
    gcCycles := heap.gcCycles
  }

-- Check if GC is needed (based on heap usage)
def ZHeap.shouldCollect (heap : ZHeap) : Bool :=
  let usedBytes := heap.regions.foldl (λ acc r => acc + r.used) 0
  let usageRatio := usedBytes.toFloat / heap.config.maxHeapSize.toFloat
  usageRatio > 0.75  -- GC when 75% full

-- Get regions that need evacuation (for relocation phase)
def ZHeap.getRelocationCandidates (heap : ZHeap) : List Nat :=
  -- Select regions with low live data ratio for evacuation
  heap.usedRegions.filter (λ idx =>
    let region := heap.regions.get! idx
    let liveRatio := if region.used > 0 then
      region.liveBytes.toFloat / region.used.toFloat
    else
      1.0
    liveRatio < 0.5  -- Evacuate if less than 50% live
  )

end Compiler.Runtime.GC.Heap
