# FGC - Fax Garbage Collector

<img src="../../faxc/docs/rubah-arktik.svg" width="80" alt="Fax Arctic Fox">

[![Crates.io](https://img.shields.io/crates/v/fgc.svg)](https://crates.io/crates/fgc)
[![Documentation](https://docs.rs/fgc/badge.svg)](https://docs.rs/fgc)
[![License](https://img.shields.io/crates/l/fgc.svg)](LICENSE)

A high-performance, low-latency concurrent garbage collector for Rust applications, inspired by ZGC (Z Garbage Collector) from OpenJDK.

## Overview

FGC is a concurrent mark-compact garbage collector designed for applications requiring sub-10ms pause times regardless of heap size. It implements cutting-edge GC techniques including colored pointers, load barriers, and region-based memory management to achieve minimal application interruption during garbage collection cycles.

Built with Rust's safety guarantees in mind, FGC provides safe abstractions over complex concurrent memory management while maintaining the performance characteristics needed for latency-sensitive applications.

## Features

- **Concurrent Mark-Compact**: Marking and compaction run concurrently with application threads, minimizing pause times
- **Colored Pointers**: GC metadata stored in unused pointer bits (44-47) eliminates the need for object header modifications
- **Load Barriers**: Intercept pointer reads for concurrent marking and pointer healing (self-healing pointers)
- **Region-Based Heap**: Heap divided into small (2MB), medium (32MB), and large regions for parallel collection
- **Generational Collection**: Young/Old generation separation for improved efficiency with short-lived objects
- **Thread-Local Allocation Buffers (TLAB)**: Lock-free allocation for hot paths
- **NUMA-Aware**: Optimized memory allocation for multi-socket systems
- **Compacting**: Eliminates fragmentation through object relocation
- **Comprehensive Statistics**: Built-in performance monitoring and metrics export

## Installation

Add FGC to your `Cargo.toml`:

```toml
[dependencies]
fgc = "0.1"
```

### Platform Requirements

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | ✅ Full | Multi-mapping with mmap |
| Linux aarch64 | ✅ Full | Multi-mapping with mmap |
| macOS x86_64 | ⚠️ Partial | shm_open based, limited features |
| macOS Apple Silicon | ⚠️ Partial | shm_open based, limited features |
| Windows x86_64 | ❌ Limited | Shared memory not fully implemented |

### Prerequisites

- Rust 1.75 or later
- Linux kernel 4.0+ (for full multi-mapping support)
- cmake (for building some dependencies)

## Quick Start

### Basic Usage

```rust
use fgc::{GarbageCollector, FgcConfig, GcGeneration};

fn main() -> Result<(), fgc::FgcError> {
    // Create GC with default configuration
    let config = FgcConfig::default();
    let gc = GarbageCollector::new(config)?;
    
    // Allocate objects
    let addr = gc.allocate(64)?;
    
    // Register as root to prevent collection
    gc.register_root(addr)?;
    
    // Use the allocated memory
    unsafe {
        *(addr as *mut u64) = 0x12345678;
    }
    
    // Trigger GC when needed
    gc.collect();
    
    // Unregister root when done
    gc.unregister_root(addr)?;
    
    Ok(())
}
```

### Using the Runtime

For applications needing full GC lifecycle management:

```rust
use fgc::{Runtime, GcConfig, GcGeneration};

fn main() -> Result<(), fgc::FgcError> {
    // Create runtime with custom configuration
    let config = GcConfig {
        max_heap_size: 256 * 1024 * 1024,  // 256MB
        target_pause_time_ms: 5,            // 5ms target pause
        generational: true,                 // Enable generational GC
        tlab_enabled: true,                 // Enable TLAB
        verbose: true,                      // Enable GC logging
        ..Default::default()
    };
    
    let runtime = Runtime::new(config)?;
    runtime.start()?;
    
    // Allocate objects through runtime
    let addr = runtime.allocate(128)?;
    
    // Request GC when needed
    runtime.request_gc(GcGeneration::Young);
    
    // Check safepoints in long-running loops
    runtime.check_safepoint();
    
    // Cleanup
    runtime.stop()?;
    
    Ok(())
}
```

### Custom Configuration

```rust
use fgc::GcConfig;

let config = GcConfig {
    // Heap size configuration
    max_heap_size: 4 * 1024 * 1024 * 1024,  // 4GB max
    min_heap_size: 512 * 1024 * 1024,       // 512MB min
    initial_heap_size: 1 * 1024 * 1024 * 1024, // 1GB initial
    
    // Performance tuning
    target_pause_time_ms: 5,                 // 5ms pause target
    gc_threads: Some(8),                     // 8 GC threads
    
    // Generational settings
    generational: true,                      // Enable generational
    young_ratio: 0.3,                        // 30% young generation
    tenure_threshold: 9,                     // Promote after 9 GCs
    
    // TLAB settings
    tlab_enabled: true,                      // Enable TLAB
    tlab_size: 256 * 1024,                   // 256KB TLAB
    
    // Platform optimizations
    numa_aware: true,                        // NUMA-aware allocation
    use_large_pages: false,                  // Use large pages
    
    // Debugging
    verbose: true,                           // GC logging
    stats_enabled: true,                     // Collect statistics
    
    ..Default::default()
};

let gc = GarbageCollector::new(config)?;
```

## Configuration Reference

### Heap Size Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_heap_size` | 50% of RAM (max 4GB) | Maximum heap size in bytes |
| `min_heap_size` | 25% of max_heap_size | Minimum heap size (heap won't shrink below) |
| `initial_heap_size` | min_heap_size | Initial heap size at startup |
| `soft_max_heap_size` | max_heap_size | Soft limit - GC tries to stay below but can exceed |

### Performance Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `target_pause_time_ms` | 10 | Target GC pause time in milliseconds |
| `gc_threads` | Auto (min(4, cores/2)) | Number of concurrent GC threads |
| `gc_interval_ms` | 0 | Periodic GC interval (0 = on-demand only) |

### Generational Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `generational` | true | Enable young/old generation separation |
| `young_ratio` | 0.3 | Ratio of heap for young generation (0.0-1.0) |
| `tenure_threshold` | 9 | Objects survive N minor GCs before promotion |

### TLAB Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `tlab_enabled` | true | Enable Thread-Local Allocation Buffers |
| `tlab_size` | 256KB | Default TLAB size per thread |
| `tlab_min_size` | 16KB | Minimum TLAB size |
| `tlab_max_size` | 2MB | Maximum TLAB size |
| `tlab_resize` | true | Enable dynamic TLAB resizing |

### Region Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `small_region_size` | 2MB | Size of small regions |
| `medium_region_size` | 32MB | Size of medium regions |
| `small_threshold` | 256 bytes | Objects ≤ this size use small regions |
| `large_threshold` | 4KB | Objects > this size get dedicated regions |

### Platform Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `numa_aware` | true | Enable NUMA-aware allocation |
| `use_large_pages` | false | Use large/huge pages (requires OS support) |
| `large_page_size` | 2MB | Large page size (2MB or 1GB) |

### Debug Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `verbose` | false | Enable GC logging |
| `stats_enabled` | true | Enable statistics collection |

## Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Mutator Threads                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │  TLAB    │  │  TLAB    │  │  TLAB    │                      │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                      │
│       │             │             │                              │
│       └─────────────┴─────────────┘                              │
│                    Write Barrier                                 │
└───────────────────────────┼──────────────────────────────────────┘
                            │
┌───────────────────────────┼──────────────────────────────────────┐
│                    GC Threads                                    │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────┐          │
│  │              Mark Phase                            │          │
│  │  - Concurrent marking from roots                   │          │
│  │  - Stack scanning at safepoint                     │          │
│  │  - Load barriers mark on access                    │          │
│  └───────────────────────────────────────────────────┘          │
│                           │                                      │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────┐          │
│  │           Relocation Phase                         │          │
│  │  - Select regions for relocation                   │          │
│  │  - Copy objects to new locations                   │          │
│  │  - Update references via pointer healing           │          │
│  └───────────────────────────────────────────────────┘          │
│                           │                                      │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────┐          │
│  │           Cleanup Phase                            │          │
│  │  - Free old regions                                │          │
│  │  - Clear forwarding tables                         │          │
│  └───────────────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

### GC Cycle Phases

```
┌─────────────────────────────────────────────────────────┐
│                   GC CYCLE FLOW                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Phase 1: Pause Mark Start (STW < 1ms)                  │
│    - Flip mark bits (Marked0 ↔ Marked1)                 │
│    - Scan roots (stacks, globals)                        │
│    - Initialize mark queues                              │
│                                                          │
│  Phase 2: Concurrent Mark (NO STW)                      │
│    - Application threads running                         │
│    - Load barriers mark objects on access                │
│    - GC threads process mark queue                       │
│                                                          │
│  Phase 3: Pause Mark End (STW < 1ms)                    │
│    - Process remaining mark queue                        │
│    - Handle weak references, finalizers                  │
│                                                          │
│  Phase 4: Concurrent Prepare Relocation (NO STW)        │
│    - Select regions for relocation set                   │
│    - Setup forwarding tables                             │
│    - Allocate destination regions                        │
│                                                          │
│  Phase 5: Concurrent Relocate (NO STW)                  │
│    - Copy objects to destination regions                 │
│    - Load barriers handle pointer healing                │
│    - Update forwarding tables                            │
│                                                          │
│  Phase 6: Concurrent Cleanup (NO STW)                   │
│    - Free source regions                                 │
│    - Clear forwarding tables                             │
│    - Update heap statistics                              │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Colored Pointers

FGC stores GC metadata directly in pointer bits 44-47:

```
64-bit Pointer Layout:
┌────────────┬─────┬─────┬─────┬─────┬──────────────────────┐
│  Unused    │ Fin │ Rem │ M1  │ M0  │     Address          │
│  63-48     │ 47  │ 46  │ 45  │ 44  │       43-0           │
└────────────┴─────┴─────┴─────┴─────┴──────────────────────┘

Color Bits:
- M0 (Marked0): Object marked in even GC cycle
- M1 (Marked1): Object marked in odd GC cycle  
- Rem (Remapped): Pointer already updated after relocation
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

Physical memory mapped to 3 different virtual addresses.
Color bits select which view to access.
```

## Performance

### Target Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Pause Time | < 10ms | Regardless of heap size |
| Throughput | > 90% | Application time vs GC time |
| Allocation Speed | O(1) | Bump pointer + TLAB |
| Heap Range | 8MB - 16TB | Scalable |

### Expected Overhead

| Operation | Expected Time |
|-----------|---------------|
| TLAB Allocation (hit) | < 10ns |
| TLAB Allocation (miss) | ~100ns |
| Load Barrier (fast path) | < 5ns |
| Load Barrier (slow path) | ~50ns |
| GC Pause (mark start) | < 1ms |
| GC Pause (mark end) | < 1ms |

### Tuning Guidelines

**For Low-Latency Applications:**
```rust
let config = GcConfig {
    target_pause_time_ms: 5,      // Aggressive pause target
    gc_threads: Some(8),          // More parallelism
    generational: true,           // Reduce old-gen collections
    young_ratio: 0.4,             // Larger young generation
    ..Default::default()
};
```

**For High-Throughput Applications:**
```rust
let config = GcConfig {
    target_pause_time_ms: 50,     // Relaxed pause target
    gc_threads: Some(4),          // Fewer GC threads
    tlab_size: 512 * 1024,        // Larger TLABs
    ..Default::default()
};
```

**For Memory-Constrained Environments:**
```rust
let config = GcConfig {
    max_heap_size: 512 * 1024 * 1024,  // 512MB max
    min_heap_size: 64 * 1024 * 1024,   // 64MB min
    initial_heap_size: 128 * 1024 * 1024, // 128MB start
    ..Default::default()
};
```

## Limitations

### Current Limitations

- **Stack Scanning**: Conservative stack scanning may keep garbage alive longer than necessary
- **Platform Support**: Full multi-mapping only available on Linux; macOS uses shm_open fallback
- **Windows Support**: Limited functionality on Windows platforms
- **Precise GC**: Requires explicit root registration; automatic root detection is conservative

### Known Issues

See [GitHub Issues](https://github.com/your-org/fgc/issues) for current known issues and workarounds.

## Monitoring and Debugging

### GC Statistics

```rust
use fgc::{GarbageCollector, FgcConfig};

let config = FgcConfig::default();
let gc = GarbageCollector::new(config)?;

// Get GC statistics
let stats = gc.stats();
let summary = stats.summary();

println!("Total GC cycles: {}", summary.total_cycles);
println!("Average pause time: {:.2}ms", summary.avg_pause_ms);
println!("Max pause time: {:.2}ms", summary.max_pause_ms);
println!("Heap used: {:.2}MB", summary.heap_used_mb);
```

### Verbose Logging

Enable GC logging via configuration:

```rust
let config = GcConfig {
    verbose: true,
    ..Default::default()
};
```

Or via environment variable:

```bash
export FGC_VERBOSE=1
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `FGC_MAX_HEAP` | Maximum heap size in bytes |
| `FGC_MIN_HEAP` | Minimum heap size in bytes |
| `FGC_PAUSE_TIME_MS` | Target pause time in milliseconds |
| `FGC_GC_THREADS` | Number of GC threads |
| `FGC_VERBOSE` | Enable verbose logging (1 = enabled) |

## Testing

### Run Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --tests

# All tests
cargo test

# With verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_allocate_tlab_memory_basic
```

### Run Benchmarks

```bash
# Requires nightly Rust
cargo +nightly bench
```

### Sanitizer Testing

```bash
# Address Sanitizer
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test

# Thread Sanitizer  
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

```bash
# Clone the repository
git clone https://github.com/your-org/fgc
cd fgc

# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

FGC is inspired by:

- **ZGC** (Z Garbage Collector) from OpenJDK - Colored pointers, load barriers, concurrent marking
- **Shenandoah GC** - Region-based collection, concurrent compaction
- **Go GC** - Simple API design, practical performance tuning

## Getting Help

- [API Documentation](https://docs.rs/fgc)
- [GitHub Issues](https://github.com/your-org/fgc/issues)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
