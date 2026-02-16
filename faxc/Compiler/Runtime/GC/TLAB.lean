/-
FGC Thread-Local Allocation Buffer (TLAB)
TLAB provides fast, contention-free allocation by giving each thread its own
small allocation area. This eliminates synchronization on the global heap.
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Heap

namespace Compiler.Runtime.GC.TLAB

open ZPointer Heap

-- TLAB configuration
structure TLABConfig where
  minSize : Nat := 32 * 1024          -- 32KB minimum TLAB size
  maxSize : Nat := 1024 * 1024        -- 1MB maximum TLAB size
  targetRefillWaste : Float := 0.02   -- 2% waste before refill
  fastRefillThreshold : Nat := 64     -- Refill if less than 64 bytes free
  deriving Repr

def TLABConfig.default : TLABConfig :=
  { minSize := 32 * 1024
    maxSize := 1024 * 1024
    targetRefillWaste := 0.02
    fastRefillThreshold := 64
  }

-- TLAB state for a single thread
structure ThreadLocalAllocBuffer where
  -- Memory area
  start : UInt64                      -- Start address of TLAB
  top : UInt64                        -- Current allocation pointer (bump pointer)
  end : UInt64                        -- End address of TLAB
  
  -- Statistics
  allocatedBytes : Nat                -- Total bytes allocated in this TLAB
  wasteBytes : Nat                    -- Unused bytes at end (too small for allocation)
  allocationCount : Nat               -- Number of allocations
  
  -- Configuration
  config : TLABConfig
  
  -- Thread identification
  threadId : Nat
  
  -- Dirty flag for GC
  dirty : Bool                        -- True if allocations since last GC
  deriving Repr

-- Create a new TLAB
def ThreadLocalAllocBuffer.new (startAddr : UInt64) (size : Nat)
    (threadId : Nat) (config : TLABConfig := TLABConfig.default)
    : ThreadLocalAllocBuffer :=
  { start := startAddr
    top := startAddr
    end := startAddr + size.toUInt64
    allocatedBytes := 0
    wasteBytes := 0
    allocationCount := 0
    config := config
    threadId := threadId
    dirty := false
  }

-- Calculate remaining space in TLAB
def ThreadLocalAllocBuffer.remaining (tlab : ThreadLocalAllocBuffer) : Nat :=
  (tlab.end - tlab.top).toNat

-- Check if TLAB has enough space for allocation
def ThreadLocalAllocBuffer.hasSpace (tlab : ThreadLocalAllocBuffer) (bytes : Nat) : Bool :=
  tlab.remaining >= bytes

-- Check if TLAB needs refill
-- Returns true if remaining space is below threshold
def ThreadLocalAllocBuffer.needsRefill (tlab : ThreadLocalAllocBuffer) : Bool :=
  let remaining := tlab.remaining
  let wasteThreshold := (tlab.end - tlab.start).toNat.toFloat * tlab.config.targetRefillWaste
  
  remaining < tlab.config.fastRefillThreshold ||
  remaining.toFloat < wasteThreshold

-- Fast path allocation using bump pointer
@[inline]
def ThreadLocalAllocBuffer.allocateFast (tlab : ThreadLocalAllocBuffer) 
    (bytes : Nat) : Option (ZPointer × ThreadLocalAllocBuffer) :=
  
  let alignedBytes := alignUp bytes 8  -- Align to 8 bytes
  
  if tlab.hasSpace alignedBytes then
    let objAddr := tlab.top
    let newTop := objAddr + alignedBytes.toUInt64
    let ptr := ZPointer.fromAddress objAddr .remapped
    
    let newTLAB := { tlab with
      top := newTop
      allocatedBytes := tlab.allocatedBytes + alignedBytes
      allocationCount := tlab.allocationCount + 1
      dirty := true
    }
    
    some (ptr, newTLAB)
  else
    none

-- Align value up to alignment boundary
def alignUp (value : Nat) (alignment : Nat) : Nat :=
  let remainder := value % alignment
  if remainder == 0 then value else value + (alignment - remainder)

-- Allocate with end check (slower, handles slow path)
def ThreadLocalAllocBuffer.allocate (tlab : ThreadLocalAllocBuffer)
    (bytes : Nat) : AllocationResult :=
  
  match tlab.allocateFast bytes with
  | some (ptr, newTLAB) => 
    AllocationResult.success ptr newTLAB
  | none =>
    if tlab.remaining > 0 then
      -- Record waste and request refill
      AllocationResult.needsRefill tlab.remaining
    else
      AllocationResult.full

-- Allocation result type
inductive AllocationResult
  | success (ptr : ZPointer) (newTLAB : ThreadLocalAllocBuffer)
  | needsRefill (wasteBytes : Nat)
  | full
  deriving Repr

-- Slow path allocation (outside TLAB)
-- This goes to the global heap with synchronization
structure SlowAllocationRequest where
  bytes : Nat
  typeId : UInt32
  threadId : Nat
  preferredSize : Nat  -- Preferred TLAB size for next allocation
  deriving Repr

-- TLAB statistics
def ThreadLocalAllocBuffer.getStats (tlab : ThreadLocalAllocBuffer) : TLABStats :=
  let totalSize := (tlab.end - tlab.start).toNat
  let utilization := if totalSize > 0 then
    tlab.allocatedBytes.toFloat / totalSize.toFloat
  else
    0.0
  
  { threadId := tlab.threadId
    totalSize := totalSize
    allocatedBytes := tlab.allocatedBytes
    wasteBytes := tlab.wasteBytes
    allocationCount := tlab.allocationCount
    utilizationRate := utilization
    refillCount := 0  -- Tracked separately
  }

-- TLAB Manager - manages TLABs for all threads
structure TLABManager where
  tlabs : HashMap Nat ThreadLocalAllocBuffer  -- threadId -> TLAB
  config : TLABConfig
  globalHeap : ZHeap
  nextTLABId : Nat
  deriving Repr

def TLABManager.new (heap : ZHeap) (config : TLABConfig := TLABConfig.default)
    : TLABManager :=
  { tlabs := HashMap.empty
    config := config
    globalHeap := heap
    nextTLABId := 0
  }

-- Get or create TLAB for a thread
def TLABManager.getTLAB (manager : TLABManager) (threadId : Nat)
    : IO (ThreadLocalAllocBuffer × TLABManager) := do
  
  match manager.tlabs.find? threadId with
  | some tlab => return (tlab, manager)
  | none =>
    -- Allocate new TLAB from global heap
    let tlabSize := manager.config.minSize
    let baseAddr : UInt64 := 0  -- Would be allocated from global heap
    let newTLAB := ThreadLocalAllocBuffer.new baseAddr tlabSize threadId manager.config
    let newManager := { manager with 
      tlabs := manager.tlabs.insert threadId newTLAB
      nextTLABId := manager.nextTLABId + 1 }
    return (newTLAB, newManager)

-- Refill TLAB for a thread
def TLABManager.refillTLAB (manager : TLABManager) (threadId : Nat)
    (requestedSize : Nat) : IO (Option ThreadLocalAllocBuffer × TLABManager) := do
  
  -- Calculate new TLAB size based on allocation history
  let newSize := min (max requestedSize manager.config.minSize) manager.config.maxSize
  
  -- Allocate from global heap
  let baseAddr : UInt64 := 0  -- Would allocate from global heap
  
  if baseAddr == 0 then
    return (none, manager)
  else
    let newTLAB := ThreadLocalAllocBuffer.new baseAddr newSize threadId manager.config
    let newManager := { manager with 
      tlabs := manager.tlabs.insert threadId newTLAB }
    return (some newTLAB, newManager)

-- Retire TLAB (when thread exits or GC occurs)
def TLABManager.retireTLAB (manager : TLABManager) (threadId : Nat)
    : TLABManager × Option (UInt64 × Nat) :=
  
  match manager.tlabs.find? threadId with
  | some tlab =>
    let remaining := tlab.remaining
    if remaining > 0 then
      -- Return remaining space to global heap
      let waste := remaining
      let newManager := { manager with 
        tlabs := manager.tlabs.erase threadId }
      (newManager, some (tlab.top, waste))
    else
      ({ manager with tlabs := manager.tlabs.erase threadId }, none)
  | none => (manager, none)

-- Clear all TLABs (for GC)
def TLABManager.clearAll (manager : TLABManager) : TLABManager :=
  { manager with tlabs := HashMap.empty }

-- Statistics
def TLABManager.getStats (manager : TLABManager) : TLABManagerStats :=
  let tlabStats := manager.tlabs.toArray.map (λ (_, tlab) => tlab.getStats)
  
  let totalAllocated := tlabStats.foldl (λ acc s => acc + s.allocatedBytes) 0
  let totalWaste := tlabStats.foldl (λ acc s => acc + s.wasteBytes) 0
  let totalAllocations := tlabStats.foldl (λ acc s => acc + s.allocationCount) 0
  
  { activeTLABs := manager.tlabs.size
    totalAllocatedBytes := totalAllocated
    totalWasteBytes := totalWaste
    totalAllocations := totalAllocations
    averageTLABSize := if manager.tlabs.isEmpty then 0
      else totalAllocated / manager.tlabs.size
    tlabStats := tlabStats
  }

-- Main allocation function (handles both fast and slow paths)
def TLABManager.allocate (manager : TLABManager) (threadId : Nat)
    (bytes : Nat) (typeId : UInt32 := 0)
    : IO (Option ZPointer × TLABManager) := do
  
  -- Get TLAB for this thread
  let (tlab, manager') ← manager.getTLAB threadId
  
  -- Try fast allocation
  match tlab.allocateFast bytes with
  | some (ptr, newTLAB) =>
    -- Success, update TLAB
    let newManager := { manager' with 
      tlabs := manager'.tlabs.insert threadId newTLAB }
    return (some ptr, newManager)
    
  | none =>
    -- TLAB full, need to refill
    let waste := tlab.remaining
    
    -- Retire current TLAB
    let (manager'', remaining) := manager'.retireTLAB threadId
    
    -- Try to refill
    let (newTLABOpt, manager''') ← manager''.refillTLAB threadId bytes
    
    match newTLABOpt with
    | some newTLAB =>
      -- Retry allocation with new TLAB
      match newTLAB.allocateFast bytes with
      | some (ptr, finalTLAB) =>
        let finalManager := { manager''' with 
          tlabs := manager'''.tlabs.insert threadId finalTLAB }
        return (some ptr, finalManager)
      | none =>
        -- Even new TLAB doesn't have space, allocation too large
        return (none, manager''')
    | none =>
      -- Refill failed, heap might be full
      return (none, manager''')

-- Concurrent allocation wrapper
def TLABManager.allocateConcurrent (managerRef : IO.Ref TLABManager)
    (threadId : Nat) (bytes : Nat) (typeId : UInt32 := 0)
    : IO (Option ZPointer) := do
  
  let manager ← managerRef.get
  let (ptr, newManager) ← manager.allocate threadId bytes typeId
  managerRef.set newManager
  return ptr

-- TLAB statistics structure
structure TLABStats where
  threadId : Nat
  totalSize : Nat
  allocatedBytes : Nat
  wasteBytes : Nat
  allocationCount : Nat
  utilizationRate : Float
  refillCount : Nat
  deriving Repr

-- TLAB Manager statistics
structure TLABManagerStats where
  activeTLABs : Nat
  totalAllocatedBytes : Nat
  totalWasteBytes : Nat
  totalAllocations : Nat
  averageTLABSize : Nat
  tlabStats : Array TLABStats
  deriving Repr

-- Adaptive TLAB sizing
-- Adjust TLAB size based on allocation rate
def calculateAdaptiveSize (history : Array TLABAllocationHistory)
    (config : TLABConfig) : Nat :=
  
  if history.isEmpty then
    config.minSize
  else
    let avgAllocationSize := history.foldl (λ acc h => acc + h.allocationSize) 0 / history.size
    let avgFrequency := history.foldl (λ acc h => acc + h.timeSinceLastAllocation) 0 / history.size
    
    -- Size TLAB to handle ~50 allocations before refill
    let targetSize := avgAllocationSize * 50
    
    -- Clamp to config bounds
    min (max targetSize config.minSize) config.maxSize

-- Allocation history for adaptive sizing
structure TLABAllocationHistory where
  allocationSize : Nat
  timeSinceLastAllocation : Nat  -- ms
  timestamp : Nat
  deriving Repr

-- Thread-local allocation cache (very small, very fast)
-- Used for the smallest allocations (< 64 bytes)
structure FastAllocCache where
  buffer : Array (ZPointer × Nat)  -- (pointer, remainingSize)
  index : Nat
  capacity : Nat := 4
  deriving Repr

def FastAllocCache.new : FastAllocCache :=
  { buffer := #[], index := 0, capacity := 4 }

def FastAllocCache.allocate (cache : FastAllocCache) (bytes : Nat)
    : Option (ZPointer × FastAllocCache) :=
  
  if cache.index < cache.buffer.size then
    let (ptr, remaining) := cache.buffer.get! cache.index
    if remaining >= bytes then
      let alignedBytes := alignUp bytes 8
      let newPtr := ptr.addOffset 0  -- Use pointer as-is
      let newRemaining := remaining - alignedBytes
      let newBuffer := cache.buffer.set! cache.index (ptr.addOffset alignedBytes.toUInt64, newRemaining)
      some (newPtr, { cache with buffer := newBuffer })
    else if cache.index + 1 < cache.buffer.size then
      -- Try next buffer
      FastAllocCache.allocate { cache with index := cache.index + 1 } bytes
    else
      none
  else
    none

-- Prefetch next cache line for allocation
@[inline]
def prefetchForAllocation (addr : UInt64) : Unit :=
  -- Hint to CPU that we'll use this memory soon
  -- In real implementation, this would use prefetch instructions
  ()

-- Zero-initialize allocated memory (for security/correctness)
def zeroInitialize (ptr : ZPointer) (bytes : Nat) : IO Unit := do
  let addr := ptr.toAddress
  -- In real implementation, this would zero the memory
  -- For now, just return
  pure ()

end Compiler.Runtime.GC.TLAB
