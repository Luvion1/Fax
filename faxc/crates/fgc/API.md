# API Reference

Complete API reference for FGC (Fax Garbage Collector).

## Table of Contents

- [Core Types](#core-types)
  - [GarbageCollector](#garbagecollector)
  - [Runtime](#runtime)
  - [GcConfig](#gcconfig)
  - [FgcError](#fgcerror)
- [GC Control](#gc-control)
- [Allocation](#allocation)
- [Root Management](#root-management)
- [Statistics](#statistics)
- [Barrier Types](#barrier-types)
- [Heap Types](#heap-types)
- [Marker Types](#marker-types)
- [Relocate Types](#relocate-types)

---

## Core Types

### GarbageCollector

Main GC orchestrator that coordinates all garbage collection operations.

#### `new(config: GcConfig) -> Result<GarbageCollector>`

Creates a new GarbageCollector with the specified configuration.

**Parameters:**
- `config` - GC configuration parameters

**Returns:**
- `Ok(GarbageCollector)` - New GC instance
- `Err(FgcError)` - Initialization failed

**Example:**
```rust
use fgc::{GarbageCollector, FgcConfig};

let config = FgcConfig::default();
let gc = GarbageCollector::new(config)?;
```

#### `allocate(&self, size: usize) -> Result<usize>`

Allocates memory of the specified size.

**Parameters:**
- `size` - Size in bytes to allocate

**Returns:**
- `Ok(usize)` - Address of allocated memory
- `Err(FgcError::OutOfMemory)` - Heap exhausted
- `Err(FgcError::TlabError)` - Invalid alignment

**Example:**
```rust
let addr = gc.allocate(64)?;
unsafe {
    *(addr as *mut u64) = 0x12345678;
}
```

#### `allocate_aligned(&self, size: usize, alignment: usize) -> Result<usize>`

Allocates memory with custom alignment.

**Parameters:**
- `size` - Size in bytes to allocate
- `alignment` - Required alignment (must be power of 2, minimum 8)

**Returns:**
- `Ok(usize)` - Aligned address of allocated memory
- `Err(FgcError::TlabError)` - Invalid alignment
- `Err(FgcError::OutOfMemory)` - Heap exhausted

**Example:**
```rust
// 32-byte alignment for SIMD operations
let addr = gc.allocate_aligned(128, 32)?;

// 64-byte alignment for cache line
let addr = gc.allocate_aligned(256, 64)?;
```

#### `register_root(&self, address: usize) -> Result<()>`

Registers an address as a GC root. Rooted objects are preserved during GC.

**Parameters:**
- `address` - Address to register as root

**Returns:**
- `Ok(())` - Root registered successfully
- `Err(FgcError::InvalidPointer)` - Address is invalid

**Example:**
```rust
let addr = gc.allocate(64)?;
gc.register_root(addr)?;  // Prevent collection

// Use object...

gc.unregister_root(addr)?;  // Allow collection when done
```

#### `unregister_root(&self, address: usize) -> Result<()>`

Unregisters a previously registered root.

**Parameters:**
- `address` - Address to unregister

**Returns:**
- `Ok(())` - Root unregistered successfully
- `Err(FgcError::InvalidPointer)` - Address was not registered

**Example:**
```rust
gc.unregister_root(addr)?;
```

#### `collect(&self) -> Result<()>`

Executes a garbage collection cycle.

**Returns:**
- `Ok(())` - GC completed successfully
- `Err(FgcError::GcCycleFailed)` - GC cycle failed

**Example:**
```rust
// Trigger GC
gc.collect()?;

// Or request GC (runs asynchronously)
gc.request_gc(GcGeneration::Young, GcReason::Explicit);
```

#### `request_gc(&self, generation: GcGeneration, reason: GcReason)`

Requests a garbage collection cycle (asynchronous).

**Parameters:**
- `generation` - Generation to collect (Young, Old, or Full)
- `reason` - Reason for GC (for logging/statistics)

**Example:**
```rust
use fgc::{GcGeneration, GcReason};

gc.request_gc(GcGeneration::Young, GcReason::Explicit);
gc.request_gc(GcGeneration::Full, GcReason::HeapThreshold { used: 1000000, threshold: 500000 });
```

#### `is_collecting(&self) -> bool`

Returns true if GC is currently running.

**Returns:**
- `true` - GC cycle in progress
- `false` - GC idle

**Example:**
```rust
if gc.is_collecting() {
    println!("GC in progress, waiting...");
}
```

#### `state(&self) -> GcState`

Returns the current GC state.

**Returns:**
- `GcState` - Current state (Idle, Marking, Relocating, Cleanup)

**Example:**
```rust
match gc.state() {
    GcState::Idle => println!("GC idle"),
    GcState::Marking => println!("Marking in progress"),
    GcState::Relocating => println!("Relocating objects"),
    GcState::Cleanup => println!("Cleaning up"),
}
```

#### `stats(&self) -> Arc<GcStats>`

Returns GC statistics collector.

**Returns:**
- `Arc<GcStats>` - Statistics collector

**Example:**
```rust
let stats = gc.stats();
let summary = stats.summary();
println!("Total GC cycles: {}", summary.total_cycles);
```

#### `cycle_count(&self) -> u64`

Returns the total number of GC cycles executed.

**Returns:**
- `u64` - Number of completed GC cycles

**Example:**
```rust
println!("GC has run {} times", gc.cycle_count());
```

#### `heap(&self) -> &Arc<Heap>`

Returns a reference to the managed heap.

**Returns:**
- `&Arc<Heap>` - Heap reference

**Example:**
```rust
let heap_stats = gc.heap().get_stats();
println!("Heap used: {} bytes", heap_stats.used);
```

#### `shutdown(&self) -> Result<()>`

Gracefully shuts down the GC, stopping all threads.

**Returns:**
- `Ok(())` - Shutdown successful
- `Err(FgcError)` - Shutdown failed

**Example:**
```rust
gc.shutdown()?;
```

---

### Runtime

High-level runtime integration for GC lifecycle management.

#### `new(config: GcConfig) -> Result<Runtime>`

Creates a new Runtime with the specified configuration.

**Parameters:**
- `config` - GC configuration

**Returns:**
- `Ok(Runtime)` - New runtime instance
- `Err(FgcError)` - Initialization failed

**Example:**
```rust
use fgc::{Runtime, GcConfig};

let config = GcConfig::default();
let runtime = Runtime::new(config)?;
```

#### `start(&self) -> Result<()>`

Starts the runtime and all GC services.

**Returns:**
- `Ok(())` - Runtime started
- `Err(FgcError)` - Start failed

**Example:**
```rust
runtime.start()?;
```

#### `stop(&self) -> Result<()>`

Stops the runtime and cleans up resources.

**Returns:**
- `Ok(())` - Runtime stopped
- `Err(FgcError)` - Stop failed

**Example:**
```rust
runtime.stop()?;
```

#### `allocate(&self, size: usize) -> Result<usize>`

Allocates memory through the runtime.

**Parameters:**
- `size` - Size in bytes

**Returns:**
- `Ok(usize)` - Allocated address
- `Err(FgcError)` - Allocation failed

**Example:**
```rust
let addr = runtime.allocate(128)?;
```

#### `request_gc(&self, generation: GcGeneration)`

Requests a GC cycle.

**Parameters:**
- `generation` - Generation to collect

**Example:**
```rust
runtime.request_gc(GcGeneration::Young);
runtime.request_gc(GcGeneration::Full);
```

#### `check_safepoint(&self)`

Checks if thread should block at a safepoint.

Call this in long-running loops to allow GC to proceed.

**Example:**
```rust
for i in 0..1000000 {
    if i % 1000 == 0 {
        runtime.check_safepoint();
    }
    // ... work ...
}
```

#### `gc(&self) -> &Arc<GarbageCollector>`

Returns the underlying GC instance.

**Returns:**
- `&Arc<GarbageCollector>` - GC reference

**Example:**
```rust
let gc = runtime.gc();
gc.collect()?;
```

#### `state(&self) -> RuntimeState`

Returns the current runtime state.

**Returns:**
- `RuntimeState` - Current state (Initialized, Running, Stopping, Stopped)

**Example:**
```rust
match runtime.state() {
    RuntimeState::Running => println!("Runtime is running"),
    RuntimeState::Stopped => println!("Runtime is stopped"),
    _ => {}
}
```

#### `register_finalizer<F>(&self, object: usize, finalizer_fn: F)`

Registers a finalizer to be called when object is collected.

**Parameters:**
- `object` - Address of object
- `finalizer_fn` - Closure to call on collection

**Example:**
```rust
runtime.register_finalizer(addr, |obj_addr| {
    println!("Object at {} is being collected", obj_addr);
});
```

---

### GcConfig

Configuration parameters for GC behavior.

#### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_heap_size` | `usize` | 50% RAM | Maximum heap size in bytes |
| `min_heap_size` | `usize` | 25% max | Minimum heap size |
| `initial_heap_size` | `usize` | min_heap | Initial heap size |
| `soft_max_heap_size` | `usize` | max_heap | Soft heap limit |
| `target_pause_time_ms` | `u64` | 10 | Target GC pause time (ms) |
| `gc_threads` | `Option<usize>` | Auto | Number of GC threads |
| `generational` | `bool` | true | Enable generational GC |
| `young_ratio` | `f32` | 0.3 | Young generation ratio |
| `tenure_threshold` | `u8` | 9 | Promote after N GCs |
| `small_region_size` | `usize` | 2MB | Small region size |
| `medium_region_size` | `usize` | 32MB | Medium region size |
| `small_threshold` | `usize` | 256 | Small object threshold |
| `large_threshold` | `usize` | 4KB | Large object threshold |
| `tlab_enabled` | `bool` | true | Enable TLAB |
| `tlab_size` | `usize` | 256KB | Default TLAB size |
| `tlab_min_size` | `usize` | 16KB | Minimum TLAB size |
| `tlab_max_size` | `usize` | 2MB | Maximum TLAB size |
| `tlab_resize` | `bool` | true | Dynamic TLAB resizing |
| `numa_aware` | `bool` | true | NUMA-aware allocation |
| `use_large_pages` | `bool` | false | Use large pages |
| `large_page_size` | `usize` | 2MB | Large page size |
| `verbose` | `bool` | false | Enable GC logging |
| `stats_enabled` | `bool` | true | Enable statistics |
| `gc_interval_ms` | `u64` | 0 | Periodic GC interval |

#### `default() -> GcConfig`

Creates default configuration balanced for general-purpose workloads.

**Example:**
```rust
let config = GcConfig::default();
```

#### `validate(&self) -> Result<(), ConfigError>`

Validates configuration parameters.

**Returns:**
- `Ok(())` - Configuration is valid
- `Err(ConfigError)` - Invalid configuration

**Example:**
```rust
let config = GcConfig {
    max_heap_size: 0,  // Invalid!
    ..Default::default()
};

assert!(config.validate().is_err());
```

#### `from_env() -> GcConfig`

Creates configuration from environment variables.

**Environment Variables:**
- `FGC_MAX_HEAP` - Maximum heap size
- `FGC_MIN_HEAP` - Minimum heap size
- `FGC_PAUSE_TIME_MS` - Target pause time
- `FGC_GC_THREADS` - Number of GC threads
- `FGC_VERBOSE` - Enable verbose logging (1 = true)

**Example:**
```bash
export FGC_MAX_HEAP=4294967296
export FGC_PAUSE_TIME_MS=5
export FGC_VERBOSE=1
```

```rust
let config = GcConfig::from_env();
```

#### `estimated_overhead(&self) -> f32`

Estimates GC overhead percentage based on configuration.

**Returns:**
- `f32` - Estimated CPU overhead (0-100%)

**Example:**
```rust
let config = GcConfig::default();
println!("Estimated GC overhead: {:.1}%", config.estimated_overhead());
```

---

### FgcError

Error type for all FGC operations.

#### Variants

| Variant | Description |
|---------|-------------|
| `OutOfMemory { requested, available }` | Heap exhausted |
| `HeapInitialization(String)` | Heap setup failed |
| `InvalidPointer { address }` | Invalid memory address |
| `RegionAllocationFailed { reason }` | Region allocation failed |
| `ConcurrentModification { operation }` | Concurrent modification detected |
| `GcCycleFailed { reason }` | GC cycle failed |
| `MarkingFailed(String)` | Mark phase failed |
| `RelocationFailed(String)` | Relocate phase failed |
| `ForwardingTableError(String)` | Forwarding table error |
| `TlabError(String)` | TLAB operation failed |
| `NumaError(String)` | NUMA operation failed |
| `VirtualMemoryError(String)` | Virtual memory error |
| `Configuration(String)` | Invalid configuration |
| `Internal(String)` | Internal error |
| `AtomicUpdateFailed(usize)` | CAS operation failed |
| `LockPoisoned(String)` | Mutex poisoned |
| `InvalidState { expected, actual }` | Invalid state transition |
| `InvalidArgument(String)` | Invalid argument |
| `BoundsCheckFailed { index, length }` | Index out of bounds |
| `AlignmentError { address, alignment }` | Alignment error |
| `Timeout(String)` | Operation timeout |
| `ResourceExhausted { resource }` | Resource exhausted |

#### Methods

##### `is_recoverable(&self) -> bool`

Returns true if the error is potentially recoverable.

**Example:**
```rust
match gc.allocate(1024) {
    Ok(addr) => { /* use addr */ }
    Err(e) if e.is_recoverable() => {
        gc.collect()?;  // Try to recover
        let addr = gc.allocate(1024)?;  // Retry
    }
    Err(e) => return Err(e),
}
```

##### `is_bug(&self) -> bool`

Returns true if the error indicates a bug in FGC.

**Example:**
```rust
if err.is_bug() {
    panic!("FGC bug detected: {}", err);
}
```

---

## GC Control

### GcGeneration

Specifies which generation to collect.

```rust
pub enum GcGeneration {
    /// Young generation only (minor GC)
    /// Fast, frequent, scans young regions only
    Young,
    
    /// Old generation only (partial major GC)
    /// Slower, less frequent
    Old,
    
    /// Full heap collection
    /// Slowest, triggered when heap nearly full
    Full,
}
```

**Example:**
```rust
// Minor GC - fast, collects young objects
gc.request_gc(GcGeneration::Young, GcReason::Explicit);

// Major GC - collects old objects
gc.request_gc(GcGeneration::Old, GcReason::HeapThreshold { used: 1000000, threshold: 500000 });

// Full GC - collects everything
gc.request_gc(GcGeneration::Full, GcReason::Explicit);
```

### GcReason

Specifies the reason for GC trigger.

```rust
pub enum GcReason {
    /// Heap usage exceeded threshold
    HeapThreshold { used: usize, threshold: usize },
    
    /// Explicit GC request (user call)
    Explicit,
    
    /// Periodic GC (interval timer)
    Periodic,
    
    /// System memory pressure
    MemoryPressure,
    
    /// Shutdown - final cleanup
    Shutdown,
}
```

### GcState

Current state of the GC cycle.

```rust
pub enum GcState {
    /// Idle - no GC in progress
    Idle,
    
    /// Marking phase - identifying live objects
    Marking,
    
    /// Relocating phase - moving objects
    Relocating,
    
    /// Cleanup phase - freeing old regions
    Cleanup,
}
```

---

## Allocation

### Allocator

Main allocator managing all allocation strategies.

#### `new(heap: Arc<Heap>, generational: bool) -> Allocator`

Creates a new allocator.

#### `allocate(&self, size: usize, young: bool) -> Result<usize>`

Allocates memory with generation selection.

**Example:**
```rust
// Allocate in young generation
let addr = allocator.allocate(64, true)?;

// Allocate in old generation
let addr = allocator.allocate(64, false)?;
```

#### `allocate_young(&self, size: usize) -> Result<usize>`

Allocates in young generation.

#### `allocate_old(&self, size: usize) -> Result<usize>`

Allocates in old generation.

#### `promote_object(&self, old_address: usize, size: usize) -> Result<usize>`

Promotes object from young to old generation.

#### `stats(&self) -> AllocatorStats`

Returns allocator statistics.

### Tlab

Thread-Local Allocation Buffer for lock-free allocation.

#### `allocate(&self, size: usize) -> Result<usize>`

Allocates from TLAB.

#### `has_space(&self, size: usize) -> bool`

Checks if TLAB has space for allocation.

### TlabManager

Manages TLABs for all threads.

#### `get_or_create_tlab(&self, thread_id: ThreadId, heap: &Heap) -> Result<Arc<Tlab>>`

Gets or creates TLAB for thread.

#### `refill_tlab(&self, thread_id: ThreadId, heap: &Heap) -> Result<Arc<Tlab>>`

Refills TLAB for thread.

---

## Root Management

### RootScanner

Scans and manages GC roots.

#### `new() -> RootScanner`

Creates a new root scanner.

#### `scan_roots<F>(&self, callback: F)`

Scans all roots and calls callback for each.

**Example:**
```rust
root_scanner.scan_roots(|ref_value| {
    println!("Found root: {:#x}", ref_value);
});
```

#### `get_stats(&self) -> RootStats`

Returns root scanning statistics.

### RootType

Types of GC roots.

```rust
pub enum RootType {
    Stack,      // Stack local variables
    Global,     // Static/global variables
    Class,      // Loaded classes
    Internal,   // VM internal roots
}
```

---

## Statistics

### GcStats

Collects and provides GC statistics.

#### `new() -> GcStats`

Creates a new statistics collector.

#### `record_collection(&self, cycle: u64, generation: GcGeneration, duration: Duration)`

Records a GC collection event.

#### `summary(&self) -> GcSummary`

Returns summary statistics.

**Example:**
```rust
let summary = stats.summary();
println!("Total cycles: {}", summary.total_cycles);
println!("Avg pause: {:.2}ms", summary.avg_pause_ms);
println!("Max pause: {:.2}ms", summary.max_pause_ms);
```

### GcSummary

Summary statistics structure.

| Field | Type | Description |
|-------|------|-------------|
| `total_cycles` | `u64` | Total GC cycles |
| `minor_cycles` | `u64` | Minor (young) GC count |
| `major_cycles` | `u64` | Major (old/full) GC count |
| `avg_pause_ms` | `f64` | Average pause time (ms) |
| `max_pause_ms` | `f64` | Maximum pause time (ms) |
| `heap_used_mb` | `f64` | Heap used (MB) |
| `uptime_secs` | `u64` | Uptime (seconds) |

### Histogram

Pause time histogram for percentile calculations.

#### `new() -> Histogram`

Creates a new histogram.

#### `record(&self, value: u64)`

Records a value in the histogram.

#### `percentile(&self, p: u64) -> u64`

Returns the p-th percentile value.

**Example:**
```rust
let p50 = histogram.percentile(50);
let p99 = histogram.percentile(99);
println!("P50: {}ns, P99: {}ns", p50, p99);
```

#### `mean(&self) -> u64`

Returns the mean value.

#### `max(&self) -> u64`

Returns the maximum value.

---

## Barrier Types

### ColoredPointer

Pointer with GC metadata in bits 44-47.

#### `new(address: usize) -> ColoredPointer`

Creates a new colored pointer from an address.

#### `address(&self) -> usize`

Returns the pure address without color bits.

#### `is_marked0(&self) -> bool`

Checks if Marked0 bit is set.

#### `is_marked1(&self) -> bool`

Checks if Marked1 bit is set.

#### `is_marked(&self) -> bool`

Checks if either mark bit is set.

#### `is_remapped(&self) -> bool`

Checks if Remapped bit is set.

#### `is_finalizable(&self) -> bool`

Checks if Finalizable bit is set.

#### `set_marked0(&mut self)`

Sets the Marked0 bit.

#### `set_marked1(&mut self)`

Sets the Marked1 bit.

#### `set_remapped(&mut self)`

Sets the Remapped bit.

#### `set_finalizable(&mut self)`

Sets the Finalizable bit.

#### `clear_color(&mut self)`

Clears all color bits.

#### `flip_mark_bit(&mut self)`

Flips Marked0 ↔ Marked1 for new GC cycle.

#### Atomic Operations

```rust
// Set mark bit atomically
ColoredPointer::set_marked0_atomic(&atomic_ptr);

// Test and set atomically
let was_marked = ColoredPointer::test_and_set_marked0(&atomic_ptr);

// Clear color atomically
ColoredPointer::clear_color_atomic(&atomic_ptr);

// Flip mark bit atomically
ColoredPointer::flip_mark_bit_atomic(&atomic_ptr);

// Compare and swap
match ColoredPointer::cas_atomic(&atomic_ptr, expected, new) {
    Ok(old) => { /* CAS succeeded */ }
    Err(current) => { /* CAS failed, current value returned */ }
}
```

### LoadBarrier

Load barrier for concurrent marking and pointer healing.

#### `heal_pointer(addr: &mut usize)`

Heals a pointer that may have been relocated.

#### `on_object_read(addr: usize)`

Called when an object is read (for marking).

---

## Heap Types

### Heap

Region-based heap management.

#### `new(config: Arc<GcConfig>) -> Result<Heap>`

Creates a new heap.

#### `allocate_tlab_memory(&self, size: usize) -> Result<usize>`

Allocates memory using bump pointer.

#### `allocate_tlab_memory_aligned(&self, size: usize, alignment: usize) -> Result<usize>`

Allocates aligned memory using bump pointer.

#### `get_stats(&self) -> HeapStats`

Returns heap statistics.

#### `base_address(&self) -> usize`

Returns heap base address.

#### `max_size(&self) -> usize`

Returns maximum heap size.

#### `committed_size(&self) -> usize`

Returns committed memory size.

### HeapStats

Heap statistics structure.

| Field | Type | Description |
|-------|------|-------------|
| `used` | `usize` | Memory in use (bytes) |
| `committed` | `usize` | Memory committed (bytes) |
| `max` | `usize` | Maximum memory (bytes) |
| `young_size` | `usize` | Young generation size |
| `old_size` | `usize` | Old generation size |
| `region_count` | `usize` | Number of active regions |
| `free_region_count` | `usize` | Number of free regions |

### Region

Memory region for object allocation.

#### `start(&self) -> usize`

Returns region start address.

#### `end(&self) -> usize`

Returns region end address.

#### `size(&self) -> usize`

Returns region size.

#### `used(&self) -> usize`

Returns used bytes in region.

#### `generation(&self) -> Generation`

Returns region generation (Young/Old).

#### `region_type(&self) -> RegionType`

Returns region type (Small/Medium/Large).

---

## Marker Types

### Marker

Concurrent marking orchestrator.

#### `new(heap: Arc<Heap>) -> Marker`

Creates a new marker.

#### `start_concurrent_marking(&self, num_threads: usize) -> Result<()>`

Starts concurrent marking with worker threads.

#### `wait_completion(&self) -> Result<()>`

Waits for marking to complete.

#### `scan_roots(&self) -> Result<()>`

Scans all roots for initial marking work.

#### `marked_count(&self) -> u64`

Returns count of marked objects.

### MarkQueue

Work-stealing queue for marking work.

#### `push(&self, object: usize)`

Pushes an object to the queue.

#### `pop(&self) -> Option<usize>`

Pops an object from the queue.

#### `is_empty(&self) -> bool`

Checks if queue is empty.

#### `clear(&self)`

Clears the queue.

---

## Relocate Types

### Relocator

Object relocation manager.

#### `new(heap: Arc<Heap>) -> Relocator`

Creates a new relocator.

#### `prepare_relocation(&self) -> Result<()>`

Prepares relocation phase.

#### `start_relocation(&self) -> Result<()>`

Starts concurrent relocation.

#### `relocate_object(&self, old_address: usize, size: usize) -> Result<usize>`

Relocates a single object.

**Returns:** New address after relocation.

#### `relocate_batch(&self, objects: &[(usize, usize)]) -> Result<Vec<usize>>`

Relocates multiple objects efficiently.

#### `lookup_forwarding(&self, old_address: usize) -> Option<usize>`

Looks up forwarding address.

#### `progress(&self) -> RelocationProgress`

Returns relocation progress.

### RelocationProgress

Relocation progress information.

| Field | Type | Description |
|-------|------|-------------|
| `relocated` | `u64` | Objects relocated |
| `total` | `u64` | Total objects to relocate |
| `bytes_relocated` | `usize` | Bytes relocated |
| `in_progress` | `bool` | Relocation in progress |

### ForwardingTable

Forwarding table for relocated objects.

#### `add_entry(&self, old_address: usize, new_address: usize)`

Adds a forwarding entry.

#### `lookup(&self, old_address: usize) -> Option<usize>`

Looks up new address for old address.

---

## Macros

### `read_barrier!($ptr:expr)`

Read barrier macro for pointer access.

**Example:**
```rust
use fgc::read_barrier;

let ptr = /* some pointer */;
let safe_ptr = read_barrier!(ptr);
let value = unsafe { *safe_ptr };
```

### `lock_result!($lock:expr)`

Locks mutex with proper error handling.

**Example:**
```rust
use fgc::lock_result;

let guard = lock_result!(mutex.lock())?;
```

### `bail!($err:expr)`

Early return with error.

**Example:**
```rust
use fgc::bail;

if condition {
    bail!(FgcError::InvalidArgument("bad value".to_string()));
}
```

### `ensure!($cond:expr, $err:expr)`

Ensures condition is true, otherwise returns error.

**Example:**
```rust
use fgc::ensure;

ensure!(size > 0, FgcError::InvalidArgument("size must be > 0"));
```

---

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `FGC_MAX_HEAP` | Maximum heap size in bytes | `export FGC_MAX_HEAP=4294967296` |
| `FGC_MIN_HEAP` | Minimum heap size in bytes | `export FGC_MIN_HEAP=536870912` |
| `FGC_PAUSE_TIME_MS` | Target pause time (ms) | `export FGC_PAUSE_TIME_MS=5` |
| `FGC_GC_THREADS` | Number of GC threads | `export FGC_GC_THREADS=8` |
| `FGC_VERBOSE` | Enable verbose logging | `export FGC_VERBOSE=1` |

---

## Complete Example

```rust
use fgc::{GarbageCollector, FgcConfig, GcGeneration, GcReason};

fn main() -> Result<(), fgc::FgcError> {
    // Configure GC
    let config = FgcConfig {
        max_heap_size: 256 * 1024 * 1024,    // 256MB
        min_heap_size: 64 * 1024 * 1024,     // 64MB
        target_pause_time_ms: 5,              // 5ms target
        gc_threads: Some(4),                  // 4 GC threads
        generational: true,                   // Enable generational
        young_ratio: 0.3,                     // 30% young gen
        tlab_enabled: true,                   // Enable TLAB
        verbose: true,                        // Enable logging
        ..Default::default()
    };

    // Create GC
    let gc = GarbageCollector::new(config)?;

    // Allocate objects
    let mut addresses = Vec::new();
    for i in 0..100 {
        let addr = gc.allocate(64)?;
        gc.register_root(addr)?;
        
        // Initialize memory
        unsafe {
            *(addr as *mut u64) = i as u64;
        }
        
        addresses.push(addr);
    }

    // Trigger GC
    gc.request_gc(GcGeneration::Young, GcReason::Explicit);
    gc.collect()?;

    // Verify objects survived
    for (i, &addr) in addresses.iter().enumerate() {
        unsafe {
            assert_eq!(*(addr as *mut u64), i as u64);
        }
    }

    // Get statistics
    let stats = gc.stats();
    let summary = stats.summary();
    println!("GC cycles: {}", summary.total_cycles);
    println!("Avg pause: {:.2}ms", summary.avg_pause_ms);

    // Cleanup
    for addr in addresses {
        gc.unregister_root(addr)?;
    }

    gc.shutdown()?;
    Ok(())
}
```
