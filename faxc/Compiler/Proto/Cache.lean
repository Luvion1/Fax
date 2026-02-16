/-
Caching mechanism for Fax compiler protobuf messages
Enables incremental compilation and performance optimization
-/

import Compiler.Proto.Messages
import Compiler.Proto.Codec

namespace Compiler.Proto.Cache

-- Cache key based on content hash
structure CacheKey where
  hash : UInt64
  source : String
  deriving Repr, BEq, Hashable

def CacheKey.fromString (s : String) : CacheKey :=
  { hash := s.hash, source := s }

-- Cache entry with metadata
structure CacheEntry (α : Type) where
  value : α
  timestamp : Nat  -- Unix timestamp
  accessCount : Nat
  size : Nat       -- Size in bytes
  deriving Repr

-- Cache configuration
structure CacheConfig where
  maxSize : Nat := 100 * 1024 * 1024  -- 100MB default
  maxEntries : Nat := 10000
  ttlSeconds : Nat := 3600  -- 1 hour default
  deriving Repr

-- Cache state
structure Cache (α : Type) where
  entries : HashMap CacheKey (CacheEntry α)
  totalSize : Nat
  config : CacheConfig
  deriving Repr

def Cache.empty {α : Type} (config : CacheConfig := {}) : Cache α :=
  { entries := HashMap.empty, totalSize := 0, config := config }

-- Cache statistics
structure CacheStats where
  hits : Nat
  misses : Nat
  evictions : Nat
  totalSize : Nat
  entryCount : Nat
  deriving Repr

def CacheStats.empty : CacheStats :=
  { hits := 0, misses := 0, evictions := 0, totalSize := 0, entryCount := 0 }

-- Insert into cache
def Cache.insert {α : Type} [Repr α] (cache : Cache α) (key : CacheKey) (value : α) (size : Nat) 
    : Cache α × Bool :=
  let entry : CacheEntry α := {
    value := value
    timestamp := 0  -- Would use actual timestamp
    accessCount := 1
    size := size
  }
  
  -- Check if we need to evict
  if cache.totalSize + size > cache.config.maxSize then
    -- Evict oldest entries (simplified - just don't insert)
    (cache, false)
  else if cache.entries.size >= cache.config.maxEntries then
    -- Evict oldest entry (simplified)
    (cache, false)
  else
    let newEntries := cache.entries.insert key entry
    ({ cache with
      entries := newEntries
      totalSize := cache.totalSize + size
    }, true)

-- Lookup in cache
def Cache.lookup {α : Type} (cache : Cache α) (key : CacheKey) : Option (CacheEntry α) × Cache α :=
  match cache.entries.find? key with
  | some entry =>
    -- Update access count
    let updatedEntry := { entry with accessCount := entry.accessCount + 1 }
    let updatedCache := { cache with entries := cache.entries.insert key updatedEntry }
    (some updatedEntry, updatedCache)
  | none => (none, cache)

-- Invalidate cache entry
def Cache.invalidate {α : Type} (cache : Cache α) (key : CacheKey) : Cache α :=
  match cache.entries.find? key with
  | some entry =>
    { cache with
      entries := cache.entries.erase key
      totalSize := cache.totalSize - entry.size
    }
  | none => cache

-- Clear entire cache
def Cache.clear {α : Type} (cache : Cache α) : Cache α :=
  { cache with entries := HashMap.empty, totalSize := 0 }

-- Cache for token streams
def TokenCache := Cache TokenStream

-- Cache for AST modules
def ModuleCache := Cache Module

-- Global cache instances (using IO.Ref for mutability)
abbrev CacheRef (α : Type) := IO.Ref (Cache α)

def createCache {α : Type} [Repr α] (config : CacheConfig := {}) : IO (CacheRef α) :=
  IO.mkRef (Cache.empty config)

-- Cache operations with IO
namespace CacheOps

-- Token stream caching
def cacheTokenStream (cache : CacheRef TokenStream) (source : String) (ts : TokenStream) : IO Unit := do
  let key := CacheKey.fromString source
  let bytes := Codec.Token.serializeTokenStream ts
  let size := bytes.size
  
  let c ← cache.get
  let (newCache, success) := c.insert key ts size
  cache.set newCache
  
  if success then
    IO.println s!"Cached token stream ({size} bytes)"
  else
    IO.println "Failed to cache token stream"

def getCachedTokenStream (cache : CacheRef TokenStream) (source : String) : IO (Option TokenStream) := do
  let key := CacheKey.fromString source
  let c ← cache.get
  let (entry, newCache) := c.lookup key
  cache.set newCache
  
  match entry with
  | some e =>
    IO.println "Token stream cache hit"
    return some e.value
  | none =>
    IO.println "Token stream cache miss"
    return none

-- Module caching
def cacheModule (cache : CacheRef Module) (source : String) (m : Module) : IO Unit := do
  let key := CacheKey.fromString source
  let bytes := Codec.AST.serializeModule m
  let size := bytes.size
  
  let c ← cache.get
  let (newCache, success) := c.insert key m size
  cache.set newCache
  
  if success then
    IO.println s!"Cached module ({size} bytes)"

def getCachedModule (cache : CacheRef Module) (source : String) : IO (Option Module) := do
  let key := CacheKey.fromString source
  let c ← cache.get
  let (entry, newCache) := c.lookup key
  cache.set newCache
  
  match entry with
  | some e => return some e.value
  | none => return none

-- Cache statistics
def getStats {α : Type} (cache : CacheRef α) : IO CacheStats := do
  let c ← cache.get
  return {
    hits := 0  -- Would track properly
    misses := 0
    evictions := 0
    totalSize := c.totalSize
    entryCount := c.entries.size
  }

-- Clear cache
def clear {α : Type} (cache : CacheRef α) : IO Unit := do
  let c ← cache.get
  cache.set c.clear
  IO.println "Cache cleared"

end CacheOps

-- Incremental compilation support
def incrementalCompile (source : String) (oldSource : Option String)
    (tokenCache : CacheRef TokenStream) (moduleCache : CacheRef Module)
    : IO (Option Messages.Module) := do
  
  -- Check if source changed
  match oldSource with
  | some old =>
    if old == source then
      -- Source unchanged, use cached module
      CacheOps.getCachedModule moduleCache source
    else
      -- Source changed, recompile
      return none
  | none =>
    -- No previous source, need to compile
    return none

end Compiler.Proto.Cache
