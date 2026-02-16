# FGC (Fax Garbage Collector) - Full Documentation

## Overview

FGC (Fax Garbage Collector) adalah garbage collector berlatensi rendah yang dirancang untuk Fax Compiler. FGC menggabungkan teknik dari ZGC (Z Garbage Collector) dan G1GC untuk memberikan pause times di bawah 1 milidetik.

## Key Features

### 1. **Colored Pointers (ZGC-style)**
- Metadata disimpan dalam bit pointer yang tidak digunakan
- Memungkinkan concurrent marking dan relocation tanpa stop-the-world
- 4 warna: `marked0`, `marked1`, `remapped`, `finalizable`

### 2. **Concurrent Operations**
- **Concurrent Marking**: Menandai objek live secara paralel dengan aplikasi
- **Concurrent Relocation**: Memindahkan objek tanpa menghentikan aplikasi
- **Load Barriers**: Memastikan konsistensi saat membaca pointer

### 3. **Thread-Local Allocation Buffers (TLAB)**
- Setiap thread memiliki buffer alokasi lokal
- Eliminasi contention pada heap global
- Bump pointer allocation untuk kecepatan maksimal

### 4. **Region-Based Heap**
- Heap dibagi menjadi region (default 2MB)
- Tiga ukuran region: Small, Medium, Large
- Evakuasi selektif region dengan sedikit data live

### 5. **Generational Collection**
- Young generation (Eden + Survivor spaces)
- Old generation (tenured objects)
- Promosi otomatis berdasarkan usia objek

### 6. **Write Barriers**
- **SATB (Snapshot At The Beginning)**: Untuk concurrent marking
- **Card Marking**: Untuk generational GC (old-to-young references)

### 7. **Reference Processing**
- Weak references
- Soft references
- Phantom references
- Finalizers

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        FGC Architecture                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Application Thread                    GC Threads                    │
│       │                                    │                         │
│       ▼                                    ▼                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────────┐                  │
│  │   TLAB   │───▶│  Fast    │    │   Concurrent │                  │
│  │          │    │  Path    │    │    Marking   │                  │
│  └──────────┘    └──────────┘    └──────────────┘                  │
│       │                 │                  │                         │
│       │                 ▼                  ▼                         │
│       │            ┌──────────┐    ┌──────────────┐                │
│       │            │   Heap   │    │  Concurrent  │                │
│       │            │  (ZHeap) │◀──▶│  Relocation  │                │
│       │            └──────────┘    └──────────────┘                │
│       │                 │                                            │
│       ▼                 ▼                                            │
│  ┌──────────┐    ┌──────────┐                                     │
│  │  Load    │    │  Write   │                                     │
│  │ Barriers │    │ Barriers │                                     │
│  └──────────┘    └──────────┘                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. ZPointer (Colored Pointers)

```lean
structure ZPointer where
  rawValue : UInt64
```

- Bit 42-45 digunakan untuk metadata warna
- Memungkinkan perubahan status objek tanpa modifikasi memory

**Colors:**
- `marked0`: Objek ditandai dalam fase marking (siklus ganjil)
- `marked1`: Objek ditandai dalam fase marking (siklus genap)
- `remapped`: Objek sudah di-relocate
- `finalizable`: Objek membutuhkan finalization

### 2. Heap Management (ZHeap)

```lean
structure ZHeap where
  regions : Array ZRegion
  freeRegions : List Nat
  usedRegions : List Nat
```

**Region States:**
- `empty`: Region kosong
- `used`: Region aktif
- `relocating`: Sedang di-evakuasi
- `relocated`: Sudah di-evakuasi, bisa dibebaskan
- `pinned`: Berisi objek yang di-pin

### 3. TLAB (Thread-Local Allocation Buffer)

```lean
structure ThreadLocalAllocBuffer where
  start : UInt64      -- Alamat awal TLAB
  top : UInt64        -- Pointer alokasi saat ini
  end : UInt64        -- Alamat akhir TLAB
  threadId : Nat
```

**Benefits:**
- No lock contention
- Bump pointer allocation
- Fast path: ~10 CPU cycles

### 4. Concurrent Marking

**Fase:**
1. **Initial Mark**: STW (Stop-The-World) singkat untuk mark roots
2. **Concurrent Mark**: Marking paralel dengan aplikasi
3. **Remark**: STW untuk finalize marking
4. **Concurrent Sweep**: Bersihkan objek dead

```lean
def concurrentMark (heap : ZHeap) (roots : RootSet) (markColor : Color) : IO (ZHeap × MarkContext)
```

### 5. Concurrent Relocation

**Proses:**
1. Pilih region dengan sedikit data live
2. Alokasikan ruang baru
3. Copy objek live
4. Update forwarding table
5. Heal pointers via load barriers

```lean
def concurrentRelocation (heap : ZHeap) (threshold : Float) : IO (ZHeap × RelocateContext)
```

### 6. Write Barriers

**SATB (Snapshot At The Beginning):**
```lean
def satbWriteBarrier (queue : SATBQueue) (oldRef : ZPointer) (gcActive : Bool) : SATBQueue × Bool
```

**Generational (Card Marking):**
```lean
def generationalWriteBarrier (rs : RememberedSet) (fieldAddr : UInt64) (newRef : ZPointer) (isOldGeneration : Bool) : RememberedSet
```

### 7. Reference Processing

**Types:**
- **Weak**: Diclear ketika objek tidak lagi strongly reachable
- **Soft**: Diclear ketika memory low
- **Phantom**: Tidak diclear, hanya enqueue untuk notification
- **Final**: Untuk finalization

```lean
def processSoftReferences (state : ReferenceProcessorState) (memoryLow : Bool) (heapUsage : Float) : ReferenceProcessorState × Array ReferenceObject
```

## Usage Examples

### Basic Allocation

```lean
import Compiler.Runtime.GC.Full

-- Initialize GC
let state ← initializeGC (1024 * 1024 * 1024) 4  -- 1GB heap, 4 threads
let stateRef ← IO.mkRef state

-- Allocate memory
match ← allocate stateRef 64 with
| some ptr =>
  IO.println s!"Allocated at 0x{ptr.toAddress}"
| none =>
  IO.println "Allocation failed"

-- Force GC
forceGC stateRef

-- Shutdown
shutdownGC (← stateRef.get)
```

### Memory Pool

```lean
-- Create memory pool
let pool ← MemoryPool.create (512 * 1024 * 1024) 4

-- Allocate from pool
let (ptr, newPool) ← pool.allocate 1 64

-- Check stats
let stats := newPool.getStats
IO.println s!"Heap usage: {stats.heapUsage * 100}%"
```

### Monitoring

```lean
-- Start monitoring
let monitorTask ← startGCMonitoring stateRef 1000  -- Every second

-- Export metrics
let metrics ← exportGCMetrics (← stateRef.get) "prometheus"
IO.println metrics
```

## Performance Characteristics

### Allocation Speed
- **TLAB fast path**: ~10 CPU cycles
- **Global heap**: ~100-200 CPU cycles (with lock)
- **Contention**: Minimal dengan TLAB

### GC Pause Times
- **Target**: <1ms
- **Typical**: 0.1-0.5ms
- **Max**: <10ms (under extreme pressure)

### Throughput
- **Target**: >95% application time
- **Typical**: 98-99% untuk aplikasi normal

### Memory Overhead
- **TLAB waste**: ~2% (configurable)
- **Metadata**: ~5-10 bytes per object
- **Region overhead**: Minimal

## Configuration

### HeapConfig
```lean
structure HeapConfig where
  minHeapSize : Nat := 8 * 1024 * 1024      -- 8MB minimum
  maxHeapSize : Nat := 1024 * 1024 * 1024   -- 1GB maximum
  regionSize : Nat := 2 * 1024 * 1024       -- 2MB regions
  concurrentGCThreads : Nat := 4            -- 4 GC threads
```

### GCConfig
```lean
structure GCConfig where
  maxPauseMs : Nat := 10                    -- Target max pause
  triggerHeapUsage : Float := 0.75          -- Trigger at 75% full
  useGenerational : Bool := true            -- Enable generational GC
```

### TLABConfig
```lean
structure TLABConfig where
  minSize : Nat := 32 * 1024                -- 32KB minimum TLAB
  maxSize : Nat := 1024 * 1024              -- 1MB maximum TLAB
  targetRefillWaste : Float := 0.02         -- 2% waste threshold
```

## Testing

### Run Tests
```bash
# All GC tests
lake exe test-gc

# Specific component tests
lake exe test-gc-zpointer
lake exe test-gc-heap
lake exe test-gc-tlab
lake exe test-gc-mark
lake exe test-gc-relocate

# Benchmarks
lake exe benchmark-gc

# Stress test
lake exe stress-test-gc -- --duration 60
```

### Test Coverage
- ✅ ZPointer operations
- ✅ Heap allocation/deallocation
- ✅ TLAB management
- ✅ Concurrent marking
- ✅ Concurrent relocation
- ✅ Write barriers
- ✅ Reference processing
- ✅ Generational collection
- ✅ GC controller

## Monitoring & Metrics

### Available Metrics
- `fgc_gc_total`: Total GC cycles
- `fgc_gc_duration_seconds`: Total GC time
- `fgc_heap_usage_ratio`: Current heap usage
- `fgc_tlab_hit_ratio`: TLAB hit rate
- `fgc_allocation_rate`: Objects allocated per second

### Prometheus Format
```
# HELP fgc_gc_total Total number of GC cycles
# TYPE fgc_gc_total counter
fgc_gc_total 42

# HELP fgc_heap_usage_ratio Current heap usage
# TYPE fgc_heap_usage_ratio gauge
fgc_heap_usage_ratio 0.65
```

### Alerts
- **Heap usage >90%**: Trigger GC more aggressively
- **Pause time >10ms**: Tune GC parameters
- **Allocation failures**: Increase heap size

## Troubleshooting

### High Pause Times
1. Check heap size - may need more memory
2. Reduce `maxPauseMs` configuration
3. Increase number of GC threads
4. Enable incremental marking

### Allocation Failures
1. Increase `maxHeapSize`
2. Trigger GC more aggressively
3. Check for memory leaks
4. Verify object liveness

### Low Throughput
1. Tune TLAB sizes
2. Adjust generational thresholds
3. Optimize write barriers
4. Check reference processing overhead

## Comparison with Other GCs

| Feature | FGC | ZGC | G1GC | ParallelGC |
|---------|-----|-----|------|------------|
| Max Pause | <1ms | <10ms | <200ms | <1s |
| Concurrent | Yes | Yes | Partial | No |
| Colored Pointers | Yes | Yes | No | No |
| Generational | Yes | No | Yes | Yes |
| Region-based | Yes | Yes | Yes | No |
| TLAB | Yes | Yes | Yes | Yes |
| Target Heap | Any | Large | Large | Small |

## Future Enhancements

1. **String Deduplication**: Automatic string interning
2. **Class Data Sharing**: Shared metaspace
3. **Compaction**: Full heap compaction option
4. **NUMA Awareness**: Non-uniform memory access optimization
5. **JIT Integration**: Compiler hints for GC

## References

- [ZGC: A Scalable Low-Latency Garbage Collector](https://openjdk.org/projects/zgc/)
- [G1GC: Garbage-First Garbage Collector](https://www.oracle.com/technetwork/tutorials/tutorials-1876574.html)
- [The Garbage Collection Handbook](http://gchandbook.org/)

## License

FGC is part of the Fax Compiler project and follows the same license.
