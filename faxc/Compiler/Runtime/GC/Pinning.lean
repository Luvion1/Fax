/-
FGC Object Pinning
Provides support for pinning objects in memory to prevent relocation during GC.
Essential for FFI and native code interoperability.
-/

import Compiler.Runtime.GC.ZPointer
import Compiler.Runtime.GC.Heap
import Compiler.Runtime.GC.Barrier

namespace Compiler.Runtime.GC.Pinning

open ZPointer Heap Barrier

-- Pin handle - represents a pinned object reference
structure PinHandle where
  handleId : Nat
  pointer : ZPointer
  threadId : Nat
  timestamp : Nat
  deriving Repr, BEq

-- Pin record tracking
structure PinRecord where
  handle : PinHandle
  refCount : Nat              -- Number of active pins
  releaseOnGC : Bool          -- Auto-release at GC safepoint
  deriving Repr

def PinRecord.new (handle : PinHandle) : PinRecord :=
  { handle := handle
    refCount := 1
    releaseOnGC := false
  }

-- Pin table - maps handles to pinned objects
structure PinTable where
  pins : HashMap Nat PinRecord
  nextHandleId : Nat
  maxPins : Nat := 10000
  deriving Repr

def PinTable.new : PinTable :=
  { pins := HashMap.empty
    nextHandleId := 1
    maxPins := 10000
  }

-- Pin an object (prevent relocation)
def PinTable.pin (table : PinTable) (ptr : ZPointer) (threadId : Nat)
    (timestamp : Nat) : Option (PinHandle × PinTable) :=
  
  if table.pins.size >= table.maxPins then
    none  -- Too many pinned objects
  else if ptr.isNull then
    none  -- Can't pin null
  else
    let handleId := table.nextHandleId
    let handle : PinHandle := {
      handleId := handleId
      pointer := ptr
      threadId := threadId
      timestamp := timestamp
    }
    
    let record := PinRecord.new handle
    let newTable := { table with 
      pins := table.pins.insert handleId record
      nextHandleId := handleId + 1 }
    
    some (handle, newTable)

-- Unpin an object (allow relocation)
def PinTable.unpin (table : PinTable) (handleId : Nat) : PinTable :=
  match table.pins.find? handleId with
  | some record =>
    if record.refCount <= 1 then
      -- Last reference, remove from table
      { table with pins := table.pins.erase handleId }
    else
      -- Decrement ref count
      let newRecord := { record with refCount := record.refCount - 1 }
      { table with pins := table.pins.insert handleId newRecord }
  | none => table

-- Increment pin reference count
def PinTable.addRef (table : PinTable) (handleId : Nat) : PinTable :=
  match table.pins.find? handleId with
  | some record =>
    let newRecord := { record with refCount := record.refCount + 1 }
    { table with pins := table.pins.insert handleId newRecord }
  | none => table

-- Check if an address is pinned
def PinTable.isPinned (table : PinTable) (addr : UInt64) : Bool :=
  table.pins.toArray.any (λ (_, record) =>
    let pinAddr := record.handle.pointer.toAddress
    -- Check if addr falls within pinned object
    -- Simplified: exact match only
    pinAddr == addr
  )

-- Get all pinned addresses
def PinTable.getPinnedAddresses (table : PinTable) : Array UInt64 :=
  table.pins.toArray.map (λ (_, record) => record.handle.pointer.toAddress)

-- Pinned region in heap
def PinTable.getPinnedRegions (table : PinTable) (readHeader : UInt64 → ObjectHeader)
    : Array (UInt64 × Nat) :=
  table.pins.toArray.map (λ (_, record) =>
    let addr := record.handle.pointer.toAddress
    let header := readHeader addr
    (addr, header.size)
  )

-- Thread-local pin set
structure ThreadLocalPins where
  pins : Array PinHandle
  maxLocalPins : Nat := 100
  deriving Repr

def ThreadLocalPins.new : ThreadLocalPins :=
  { pins := #[], maxLocalPins := 100 }

def ThreadLocalPins.pin (local : ThreadLocalPins) (handle : PinHandle)
    : Option (ThreadLocalPins × Nat) :=
  if local.pins.size >= local.maxLocalPins then
    none
  else
    let idx := local.pins.size
    some ({ local with pins := local.pins.push handle }, idx)

def ThreadLocalPins.unpin (local : ThreadLocalPins) (idx : Nat) : ThreadLocalPins :=
  if idx < local.pins.size then
    { local with pins := local.pins.set! idx handle }
  else
    local
where
  handle : PinHandle := {
    handleId := 0
    pointer := ZPointer.null
    threadId := 0
    timestamp := 0
  }

-- Scoped pinning (automatically unpins when scope ends)
structure ScopedPin where
  handle : PinHandle
  autoRelease : Bool := true
  deriving Repr

def ScopedPin.release (scoped : ScopedPin) (table : PinTable) : PinTable :=
  if scoped.autoRelease then
    table.unpin scoped.handle.handleId
  else
    table

-- Critical section pinning
-- Ensures objects stay pinned during critical operations
structure CriticalSection where
  pins : Array PinHandle
  entered : Bool
  deriving Repr

def CriticalSection.new : CriticalSection :=
  { pins := #[], entered := false }

def CriticalSection.enter (cs : CriticalSection) (objects : Array ZPointer)
    (table : PinTable) (threadId : Nat) (timestamp : Nat)
    : Option (CriticalSection × PinTable) :=
  
  if cs.entered then
    none  -- Already in critical section
  else
    let mut newTable := table
    let mut handles : Array PinHandle := #[]
    
    for obj in objects do
      match newTable.pin obj threadId timestamp with
      | some (handle, t) =>
        newTable := t
        handles := handles.push handle
      | none =>
        -- Failed to pin all objects, rollback
        return none
    
    let newCS := { pins := handles, entered := true }
    some (newCS, newTable)

def CriticalSection.exit (cs : CriticalSection) (table : PinTable) : PinTable :=
  if !cs.entered then
    table
  else
    cs.pins.foldl (λ t h => t.unpin h.handleId) table

-- Region pinning (pin entire region)
structure RegionPin where
  regionIdx : Nat
  pinnedObjects : Array PinHandle
  deriving Repr

-- Pin all objects in a region
-- Useful for native code that needs multiple related objects
def pinRegion (heap : ZHeap) (regionIdx : Nat) (table : PinTable)
    (threadId : Nat) (timestamp : Nat)
    (readHeader : UInt64 → ObjectHeader)
    (getObjectsInRegion : ZRegion → Array ZPointer)
    : Option (RegionPin × PinTable) :=
  
  if regionIdx >= heap.regions.size then
    none
  else
    let region := heap.regions.get! regionIdx
    let objects := getObjectsInRegion region
    
    let mut newTable := table
    let mut handles : Array PinHandle := #[]
    
    for obj in objects do
      match newTable.pin obj threadId timestamp with
      | some (handle, t) =>
        newTable := t
        handles := handles.push handle
      | none =>
        return none
    
    let regionPin : RegionPin := {
      regionIdx := regionIdx
      pinnedObjects := handles
    }
    
    some (regionPin, newTable)

-- Unpin entire region
def unpinRegion (regionPin : RegionPin) (table : PinTable) : PinTable :=
  regionPin.pinnedObjects.foldl (λ t h => t.unpin h.handleId) table

-- Concurrent pinning support
-- Allows multiple threads to pin objects concurrently
structure ConcurrentPinManager where
  globalTable : IO.Ref PinTable
  threadLocalTables : HashMap Nat (IO.Ref PinTable)
  deriving Repr

def ConcurrentPinManager.new : IO ConcurrentPinManager := do
  let global ← IO.mkRef PinTable.new
  return {
    globalTable := global
    threadLocalTables := HashMap.empty
  }

-- Pin with automatic handle management
def ConcurrentPinManager.pinObject (manager : ConcurrentPinManager)
    (ptr : ZPointer) (threadId : Nat)
    : IO (Option PinHandle) := do
  
  let timestamp ← IO.monoMsNow
  let table ← manager.globalTable.get
  
  match table.pin ptr threadId timestamp with
  | some (handle, newTable) =>
    manager.globalTable.set newTable
    return some handle
  | none =>
    return none

-- Release pin
def ConcurrentPinManager.unpinObject (manager : ConcurrentPinManager)
    (handleId : Nat) : IO Unit := do
  
  let table ← manager.globalTable.get
  let newTable := table.unpin handleId
  manager.globalTable.set newTable

-- Bulk pinning for array of objects
def bulkPin (table : PinTable) (objects : Array ZPointer)
    (threadId : Nat) (timestamp : Nat)
    : Option (Array PinHandle × PinTable) :=
  
  if table.pins.size + objects.size > table.maxPins then
    none
  else
    let mut newTable := table
    let mut handles : Array PinHandle := #[]
    
    for obj in objects do
      if !obj.isNull then
        match newTable.pin obj threadId timestamp with
        | some (handle, t) =>
          newTable := t
          handles := handles.push handle
        | none =>
          return none
    
    some (handles, newTable)

-- Pin statistics
def PinTable.getStats (table : PinTable) : PinStats :=
  let totalPins := table.pins.size
  let refCountSum := table.pins.fold (λ acc _ r => acc + r.refCount) 0
  
  { totalPinnedObjects := totalPins
    totalPinReferences := refCountSum
    averageRefCount := if totalPins > 0 then refCountSum.toFloat / totalPins.toFloat else 0.0
    maxHandleId := table.nextHandleId
  }

structure PinStats where
  totalPinnedObjects : Nat
  totalPinReferences : Nat
  averageRefCount : Float
  maxHandleId : Nat
  deriving Repr

-- Pinned object walker
-- Iterates over all pinned objects without moving them
def walkPinnedObjects (table : PinTable) (visitor : PinHandle → IO Unit)
    : IO Unit := do
  for (_, record) in table.pins.toArray do
    visitor record.handle

-- Handle validity checking
def PinTable.isValidHandle (table : PinTable) (handleId : Nat) : Bool :=
  table.pins.contains handleId

-- Get pinned object by handle
def PinTable.getPinnedObject (table : PinTable) (handleId : Nat) : Option ZPointer :=
  table.pins.find? handleId |>.map (λ r => r.handle.pointer)

-- Temporary pinning (short duration)
structure TemporaryPin where
  handle : PinHandle
  expiresAt : Nat  -- Timestamp when pin expires
  deriving Repr

def TemporaryPin.isExpired (tmp : TemporaryPin) (now : Nat) : Bool :=
  now > tmp.expiresAt

-- Auto-release expired temporary pins
def releaseExpiredPins (table : PinTable) (now : Nat)
    (tempPins : Array TemporaryPin) : PinTable × Array TemporaryPin :=
  
  let (expired, active) := tempPins.partition (λ p => p.isExpired now)
  
  let newTable := expired.foldl (λ t p => t.unpin p.handle.handleId) table
  (newTable, active)

-- Pinning policy configuration
structure PinPolicy where
  maxPinnedObjects : Nat := 10000
  maxPinDurationMs : Nat := 10000  -- Auto-release after 10s
  allowNestedPins : Bool := true
  trackPinSites : Bool := false     -- Track where pins are created
  deriving Repr

def PinPolicy.default : PinPolicy :=
  { maxPinnedObjects := 10000
    maxPinDurationMs := 10000
    allowNestedPins := true
    trackPinSites := false
  }

end Compiler.Runtime.GC.Pinning
