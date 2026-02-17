# FGC - Fax Garbage Collector

FGC (Fax Garbage Collector) adalah garbage collector modern yang terinspirasi dari ZGC (Z Garbage Collector) milik OpenJDK, dirancang untuk aplikasi yang membutuhkan latency sangat rendah dengan pause time di bawah 10ms.

## Fitur Utama

- **Concurrent Mark-Compact**: Marking dan compaction berjalan concurrent dengan aplikasi
- **Colored Pointers**: Metadata GC disimpan di bit pointer (bit 44-47)
- **Load Barriers**: Intercept pointer reads untuk concurrent operations
- **Region-Based Heap**: Heap dibagi menjadi small (2MB), medium (32MB), dan large regions
- **Generational Collection**: Young/Old generation untuk efisiensi
- **NUMA-Aware**: Optimasi untuk sistem multi-socket
- **TLAB**: Thread-Local Allocation Buffers untuk lock-free allocation

## Struktur Modul

```
fgc/
├── src/
│   ├── lib.rs              # Entry point & re-exports
│   ├── gc.rs               # Core GC cycle management
│   ├── config.rs           # Configuration types
│   ├── error.rs            # Error types (centralized)
│   │
│   ├── allocator/          # Memory allocation
│   │   ├── mod.rs          # Main allocator
│   │   ├── bump.rs         # Bump pointer allocator
│   │   ├── tlab.rs         # Thread-Local Allocation Buffer
│   │   └── large.rs        # Large object allocator
│   │
│   ├── barrier/            # Colored pointers & barriers
│   │   ├── mod.rs          # Module exports
│   │   ├── colored_ptr.rs  # ColoredPointer struct
│   │   ├── load_barrier.rs # Load barrier implementation
│   │   └── address_space.rs# Multi-mapping virtual memory
│   │
│   ├── heap/               # Heap management
│   │   ├── mod.rs          # Main heap struct
│   │   ├── region.rs       # Region lifecycle
│   │   ├── page.rs         # Page management
│   │   ├── virtual_memory.rs# Virtual memory ops
│   │   └── numa.rs         # NUMA awareness
│   │
│   ├── marker/             # Concurrent marking
│   │   ├── mod.rs          # Main marker
│   │   ├── mark_queue.rs   # Work-stealing queue
│   │   ├── bitmap.rs       # Mark bitmap
│   │   ├── roots.rs        # Root scanning
│   │   └── stack_scan.rs   # Concurrent stack scanning
│   │
│   ├── relocate/           # Object relocation
│   │   ├── mod.rs          # Main relocator
│   │   ├── forwarding.rs   # Forwarding tables
│   │   ├── copy.rs         # Object copying
│   │   └── compaction.rs   # Region compaction
│   │
│   ├── stats/              # Statistics & monitoring
│   │   ├── mod.rs          # Main stats collector
│   │   ├── timer.rs        # Timing utilities
│   │   ├── histogram.rs    # Pause time histogram
│   │   └── metrics.rs      # Metrics export
│   │
│   ├── util/               # Utilities
│   │   ├── mod.rs          # Module exports
│   │   ├── alignment.rs    # Alignment helpers
│   │   ├── atomic.rs       # Atomic operations
│   │   └── debug.rs        # Debug helpers
│   │
│   └── runtime/            # Runtime integration
│       ├── mod.rs          # Main runtime
│       ├── init.rs         # Initialization
│       ├── safepoint.rs    # Safepoint management
│       └── finalizer.rs    # Object finalization
│
├── tests/
│   ├── gc_basic.rs         # Basic functionality tests
│   ├── gc_concurrent.rs    # Concurrent operation tests
│   └── gc_stress.rs        # Stress tests
│
└── benches/
    └── gc_perf.rs          # Performance benchmarks
```

## Quick Start

### Basic Usage

```rust
use fgc::{GcConfig, Runtime};

fn main() -> Result<(), fgc::FgcError> {
    // Create runtime dengan default config
    let runtime = Runtime::new(GcConfig::default())?;
    runtime.start()?;

    // Allocate objects
    let addr = runtime.allocate(64)?;

    // Request GC jika perlu
    runtime.request_gc(fgc::GcGeneration::Young);

    // Shutdown saat selesai
    runtime.stop()?;

    Ok(())
}
```

### Custom Configuration

```rust
use fgc::GcConfig;

let config = GcConfig {
    max_heap_size: 4 * 1024 * 1024 * 1024,  // 4GB
    min_heap_size: 512 * 1024 * 1024,       // 512MB
    target_pause_time_ms: 5,                 // 5ms target
    gc_threads: Some(8),                     // 8 GC threads
    generational: true,                      // Enable generational
    young_ratio: 0.3,                        // 30% young gen
    tlab_enabled: true,                      // Enable TLAB
    numa_aware: true,                        // NUMA-aware
    verbose: true,                           // Enable logging
    ..Default::default()
};

let runtime = Runtime::new(config)?;
```

## GC Cycle Phases

```
┌─────────────────────────────────────────────────────────┐
│                   GC CYCLE FLOW                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Phase 1: Pause Mark Start (STW < 1ms)                  │
│    - Flip mark bits                                      │
│    - Scan roots                                          │
│                                                          │
│  Phase 2: Concurrent Mark (NO STW)                      │
│    - App threads running                                 │
│    - Load barriers mark objects                          │
│    - GC threads process queue                            │
│                                                          │
│  Phase 3: Pause Mark End (STW < 1ms)                    │
│    - Finalize marking                                    │
│                                                          │
│  Phase 4: Concurrent Prepare Relocation (NO STW)        │
│    - Select relocation set                               │
│    - Setup forwarding tables                             │
│                                                          │
│  Phase 5: Concurrent Relocate (NO STW)                  │
│    - Copy objects                                        │
│    - Pointer healing via load barriers                   │
│                                                          │
│  Phase 6: Concurrent Cleanup (NO STW)                   │
│    - Free old regions                                    │
│    - Cleanup forwarding tables                           │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Performance Characteristics

| Metric | Target | Notes |
|--------|--------|-------|
| Pause Time | < 10ms | Regardless of heap size |
| Throughput | > 90% | Application time vs GC time |
| Heap Size | 8MB - 16TB | Scalable |
| Allocation Speed | O(1) | Bump pointer + TLAB |

## Error Handling

Error handling di FGC menggunakan tipe `FgcError`:

```rust
use fgc::{FgcError, Result};

fn allocate_safely() -> Result<usize> {
    // Returns FgcError::OutOfMemory jika heap penuh
    // Returns FgcError::InvalidPointer jika address invalid
    // etc.
}
```

Lihat [docs/error-handling.md](docs/error-handling.md) untuk dokumentasi lengkap.

## Testing

### Run Tests

```bash
# Unit tests
cargo test -p fgc

# Integration tests
cargo test -p fgc --test gc_basic
cargo test -p fgc --test gc_concurrent
cargo test -p fgc --test gc_stress

# Benchmarks (requires nightly)
cargo +nightly bench -p fgc
```

## Architecture Details

### Colored Pointers

```
64-bit Pointer Layout:
┌────────────┬─────┬─────┬─────┬─────┬──────────────────────┐
│  Unused    │ Fin │ Rem │ M1  │ M0  │     Address          │
│  63-48     │ 47  │ 46  │ 45  │ 44  │       43-0           │
└────────────┴─────┴─────┴─────┴─────┴──────────────────────┘

Color Bits:
- M0 (Marked0): Object marked in even GC cycle
- M1 (Marked1): Object marked in odd GC cycle
- Rem (Remapped): Pointer already remapped
- Fin (Finalizable): Object needs finalization
```

### Multi-Mapping Virtual Memory

```
Virtual Address Space:
0x0000_0000_0000 ─┐
                  │ Remapped View (16TB)
0x0000_1000_0000 ─┘

0x0001_0000_0000 ─┐
                  │ Marked0 View (16TB)
0x0001_1000_0000 ─┘

0x0002_0000_0000 ─┐
                  │ Marked1 View (16TB)
0x0002_1000_0000 ─┘

Physical memory yang sama di-map ke 3 virtual addresses berbeda.
```

## Documentation

- [Error Handling Guide](docs/error-handling.md)
- [API Documentation](https://docs.rs/fgc)

## License

MIT OR Apache-2.0
