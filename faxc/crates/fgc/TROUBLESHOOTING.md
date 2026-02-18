# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when using FGC (Fax Garbage Collector).

## Table of Contents

- [Common Issues](#common-issues)
- [Debugging](#debugging)
- [Platform-Specific Issues](#platform-specific-issues)
- [Performance Issues](#performance-issues)
- [Getting Help](#getting-help)

## Common Issues

### GC Crashes on Startup

**Symptoms:**
- Segmentation fault immediately when creating GC
- Panic with "multi-mapping not supported"
- Application exits with signal 11 (SIGSEGV)

**Possible Causes:**

1. **Multi-mapping not supported on platform**
   - Windows does not fully support the multi-mapping technique
   - Some Linux configurations may have restrictions

2. **Insufficient virtual address space**
   - System ulimit too low
   - 32-bit system (FGC requires 64-bit)

3. **Memory mapping failures**
   - `/dev/mem` not accessible
   - Insufficient permissions

**Solutions:**

```bash
# Check virtual memory limit
ulimit -v

# Increase if needed (unlimited recommended for development)
ulimit -v unlimited

# Check if running on 64-bit system
uname -m  # Should show x86_64 or aarch64

# Check available virtual address space
cat /proc/sys/vm/max_map_count
```

**Workaround:**
```rust
// Use shm_open fallback on macOS
let config = GcConfig {
    max_heap_size: 256 * 1024 * 1024, // Smaller heap for testing
    ..Default::default()
};
```

### High Memory Usage

**Symptoms:**
- RSS (Resident Set Size) much higher than expected
- Heap not shrinking after GC
- Memory usage keeps growing

**Possible Causes:**

1. **Conservative stack scanning**
   - Stack scanning may keep garbage references alive
   - False positives in pointer detection

2. **Large initial_heap_size**
   - Default initial heap may be too large for your workload

3. **Roots not unregistered**
   - Registered roots prevent collection of objects

4. **Memory not uncommitted**
   - Committed memory not returned to OS

**Solutions:**

```rust
// Reduce initial heap size
let config = GcConfig {
    initial_heap_size: 64 * 1024 * 1024,  // 64MB instead of default
    min_heap_size: 32 * 1024 * 1024,      // 32MB minimum
    ..Default::default()
};

// Ensure roots are unregistered when done
gc.register_root(addr)?;
// ... use object ...
gc.unregister_root(addr)?;  // Don't forget this!

// Force GC to reclaim memory
gc.collect();
```

**Debugging:**
```rust
// Check heap statistics
let stats = gc.heap().get_stats();
println!("Used: {} MB", stats.used / (1024 * 1024));
println!("Committed: {} MB", stats.committed / (1024 * 1024));
println!("Max: {} MB", stats.max / (1024 * 1024));
```

### GC Not Collecting

**Symptoms:**
- Heap keeps growing despite GC calls
- Objects not being reclaimed
- OutOfMemory errors despite calling collect()

**Possible Causes:**

1. **Roots not unregistered**
   - All objects still reachable from roots

2. **Stack scanning missing references**
   - Pointers on stack not detected

3. **GC threshold too high**
   - GC not triggered automatically

4. **Objects in TLAB not visible**
   - Thread-local buffers not flushed

**Solutions:**

```rust
// Verify root registration/unregistration
gc.register_root(addr)?;
// ... use ...
gc.unregister_root(addr)?;  // Must unregister!

// Force full GC
gc.request_gc(GcGeneration::Full, GcReason::Explicit);
gc.collect()?;

// Lower GC threshold
let config = GcConfig {
    gc_interval_ms: 1000,  // GC every second
    ..Default::default()
};

// Flush TLAB before GC (if using custom allocator)
runtime.check_safepoint();
```

### Allocation Failures

**Symptoms:**
- `FgcError::OutOfMemory` on allocation
- Allocation returns error despite available memory

**Possible Causes:**

1. **Heap exhausted**
   - Actual out of memory condition

2. **Alignment requirements**
   - Requested alignment not power of 2

3. **Size exceeds limits**
   - Single allocation larger than heap

4. **Fragmentation**
   - No contiguous space for large allocation

**Solutions:**

```rust
// Check alignment (must be power of 2)
let addr = gc.allocate_aligned(64, 32)?;  // OK
let addr = gc.allocate_aligned(64, 3)?;   // Error!

// Split large allocations
let addr1 = gc.allocate(1024 * 1024)?;  // 1MB
let addr2 = gc.allocate(1024 * 1024)?;  // Another 1MB

// Increase heap size
let config = GcConfig {
    max_heap_size: 512 * 1024 * 1024,  // 512MB
    ..Default::default()
};
```

### Thread Safety Issues

**Symptoms:**
- Data races detected by sanitizer
- Inconsistent GC state
- Panic with "lock poisoned"

**Possible Causes:**

1. **Concurrent root modification**
   - Roots modified without synchronization

2. **Lock ordering violation**
   - Deadlock from inconsistent lock acquisition

3. **Mutex poisoning**
   - Thread panicked while holding lock

**Solutions:**

```rust
// Use external synchronization for root operations
use std::sync::Mutex;

let roots = Mutex::new(Vec::new());

// Register root
{
    let mut roots = roots.lock().unwrap();
    gc.register_root(addr)?;
    roots.push(addr);
}

// Unregister all roots
{
    let roots = roots.lock().unwrap();
    for &addr in roots.iter() {
        gc.unregister_root(addr)?;
    }
}

// Handle poisoned lock
match roots.lock() {
    Ok(guard) => { /* use guard */ }
    Err(poisoned) => {
        // Recover from poisoned lock
        let guard = poisoned.into_inner();
        // Use guard
    }
}
```

## Debugging

### Enable Logging

Enable verbose GC logging:

```rust
use fgc::{GarbageCollector, FgcConfig};

let config = FgcConfig {
    verbose: true,
    ..Default::default()
};

let gc = GarbageCollector::new(config)?;
```

Or via environment variable:

```bash
export FGC_VERBOSE=1
cargo run
```

**Sample Output:**
```
[GC] Requesting Young GC, reason: Explicit
[GC] Pause Mark Start (STW)
[GC] Concurrent Mark with 4 threads
[GC] Pause Mark End (STW)
[GC] Prepare Relocation
[GC] Concurrent Relocate
[GC] Cleanup
[GC] Collection complete in 2.34ms
```

### GC Statistics

Access detailed GC statistics:

```rust
use fgc::{GarbageCollector, FgcConfig};

let config = FgcConfig::default();
let gc = GarbageCollector::new(config)?;

// Get statistics
let stats = gc.stats();
let summary = stats.summary();

println!("=== GC Statistics ===");
println!("Total cycles: {}", summary.total_cycles);
println!("Minor cycles: {}", summary.minor_cycles);
println!("Major cycles: {}", summary.major_cycles);
println!("Average pause: {:.2}ms", summary.avg_pause_ms);
println!("Max pause: {:.2}ms", summary.max_pause_ms);
println!("Heap used: {:.2}MB", summary.heap_used_mb);
println!("Uptime: {}s", summary.uptime_secs);

// Get pause time histogram
let histogram = stats.pause_histogram();
println!("P50 pause: {:.2}ms", histogram.percentile(50) as f64 / 1_000_000.0);
println!("P99 pause: {:.2}ms", histogram.percentile(99) as f64 / 1_000_000.0);
```

### Memory Dumps

Enable memory inspection for debugging:

```rust
use fgc::util::debug;

// Dump heap state
debug::dump_heap(gc.heap());

// Dump region information
for region in gc.heap().get_active_regions() {
    println!("Region: start={:#x}, end={:#x}, used={}", 
             region.start(), region.end(), region.used());
}

// Check if address is valid
if debug::is_valid_address(addr) {
    println!("Address is valid");
}
```

### Sanitizer Testing

Run with sanitizers to detect memory issues:

```bash
# Address Sanitizer (detects buffer overflows, use-after-free)
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test

# Thread Sanitizer (detects data races)
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test

# Memory Sanitizer (detects uninitialized memory reads)
RUSTFLAGS="-Zsanitizer=memory" cargo +nightly test

# Leak Sanitizer (detects memory leaks)
RUSTFLAGS="-Zsanitizer=leak" cargo +nightly test
```

### Core Dumps

Enable core dumps for crash analysis:

```bash
# Enable core dumps (Linux)
ulimit -c unlimited

# Set core pattern
echo "/tmp/core.%e.%p" > /proc/sys/kernel/core_pattern

# After crash, analyze with gdb
gdb target/debug/your_binary /tmp/core.your_binary.12345
```

**GDB Commands:**
```
(gdb) bt              # Backtrace
(gdb) info threads    # List threads
(gdb) thread 1        # Switch to thread 1
(gdb) print variable  # Print variable value
```

## Platform-Specific Issues

### Linux

**Issue: Multi-mapping fails**

```bash
# Check if /dev/mem is accessible
ls -la /dev/mem

# Check kernel parameters
cat /proc/sys/vm/max_map_count

# Increase if needed
sudo sysctl -w vm.max_map_count=262144
```

**Issue: Large pages not available**

```bash
# Check hugepage configuration
cat /proc/sys/vm/nr_hugepages

# Allocate hugepages (requires root)
sudo sysctl -w vm.nr_hugepages=1024

# Check mount point
mount | grep huge
```

### macOS

**Issue: shm_open failures**

```bash
# Check shared memory limits
sysctl -a | grep shm

# Increase shared memory (requires root)
sudo sysctl -w kern.sysv.shmmax=67108864
sudo sysctl -w kern.sysv.shmall=16384
```

**Issue: Limited multi-mapping support**

macOS uses `shm_open` fallback which has limitations:
- Reduced performance compared to Linux mmap
- Some colored pointer features may not work
- Consider using Linux for development

### Windows

**Issue: Shared memory not implemented**

Windows support is limited. Workarounds:

1. Use WSL2 (Windows Subsystem for Linux)
2. Use smaller heap sizes
3. Disable features requiring multi-mapping

```rust
let config = GcConfig {
    max_heap_size: 128 * 1024 * 1024,  // Smaller heap
    use_large_pages: false,             // Not supported
    numa_aware: false,                  // Not supported
    ..Default::default()
};
```

## Performance Issues

### High GC Overhead

**Symptoms:**
- Application spends > 10% time in GC
- Frequent GC cycles
- Poor throughput

**Solutions:**

```rust
// Increase heap size to reduce GC frequency
let config = GcConfig {
    max_heap_size: 1024 * 1024 * 1024,  // 1GB
    initial_heap_size: 512 * 1024 * 1024, // 512MB
    ..Default::default()
};

// Increase GC threshold
let config = GcConfig {
    gc_interval_ms: 5000,  // GC every 5 seconds instead of on-demand
    ..Default::default()
};

// Enable generational GC (reduces old-gen collections)
let config = GcConfig {
    generational: true,
    young_ratio: 0.4,  // Larger young generation
    ..Default::default()
};

// Increase TLAB size (reduces allocation contention)
let config = GcConfig {
    tlab_enabled: true,
    tlab_size: 512 * 1024,  // 512KB
    ..Default::default()
};
```

### Long Pause Times

**Symptoms:**
- GC pauses exceed target_pause_time_ms
- Application stutter during GC

**Solutions:**

```rust
// Reduce target pause time (triggers GC more frequently)
let config = GcConfig {
    target_pause_time_ms: 5,  // More aggressive
    ..Default::default()
};

// Increase GC threads for parallelism
let config = GcConfig {
    gc_threads: Some(8),  // More parallel GC threads
    ..Default::default()
};

// Reduce heap size (less work per GC)
let config = GcConfig {
    max_heap_size: 256 * 1024 * 1024,  // Smaller heap
    ..Default::default()
};
```

### Allocation Contention

**Symptoms:**
- Slow allocation in multi-threaded workloads
- TLAB refill contention

**Solutions:**

```rust
// Increase TLAB size (fewer refills)
let config = GcConfig {
    tlab_enabled: true,
    tlab_size: 1024 * 1024,  // 1MB TLAB
    tlab_max_size: 2 * 1024 * 1024,  // Allow up to 2MB
    ..Default::default()
};

// Enable dynamic TLAB resizing
let config = GcConfig {
    tlab_resize: true,  // Auto-adjust TLAB size
    ..Default::default()
};
```

### Poor Scalability

**Symptoms:**
- Performance degrades with more threads
- Lock contention in GC

**Solutions:**

```rust
// Increase GC threads
let config = GcConfig {
    gc_threads: Some(num_cpus::get()),  // Match CPU count
    ..Default::default()
};

// Enable NUMA awareness (multi-socket systems)
let config = GcConfig {
    numa_aware: true,
    ..Default::default()
};
```

## Error Reference

### FgcError Variants

| Error | Description | Common Cause |
|-------|-------------|--------------|
| `OutOfMemory` | Heap exhausted | Heap too small, memory leak |
| `InvalidPointer` | Invalid address used | Using freed/collected pointer |
| `HeapInitialization` | Heap setup failed | Platform不支持, permissions |
| `MarkingFailed` | Mark phase failed | Root scanning error |
| `RelocationFailed` | Relocate phase failed | Forwarding table error |
| `TlabError` | TLAB operation failed | Invalid alignment |
| `LockPoisoned` | Mutex poisoned | Thread panicked while holding lock |
| `AtomicUpdateFailed` | CAS operation failed | Concurrent modification |

### Error Handling Best Practices

```rust
use fgc::{FgcError, Result};

fn allocate_and_use(gc: &GarbageCollector) -> Result<()> {
    // Handle OutOfMemory gracefully
    let addr = match gc.allocate(1024) {
        Ok(addr) => addr,
        Err(FgcError::OutOfMemory { requested, available }) => {
            eprintln!("Out of memory: requested {}, available {}", requested, available);
            gc.collect()?;  // Try GC
            gc.allocate(1024)?  // Retry
        }
        Err(e) => return Err(e),
    };

    // Handle InvalidPointer
    gc.register_root(addr).map_err(|e| {
        if matches!(e, FgcError::InvalidPointer { .. }) {
            eprintln!("Invalid pointer: address may be corrupted");
        }
        e
    })?;

    Ok(())
}
```

## Getting Help

If you can't resolve your issue:

1. **Check Documentation**
   - [README.md](README.md) - Installation and usage
   - [API.md](API.md) - Complete API reference
   - [CONTRIBUTING.md](CONTRIBUTING.md) - Development guide

2. **Search Existing Issues**
   - [GitHub Issues](https://github.com/your-org/fgc/issues)

3. **Create a New Issue**
   - Include FGC version
   - Platform and OS version
   - Minimal reproduction code
   - Error messages and logs
   - Steps to reproduce

4. **Community Support**
   - Discord channel (link in README)
   - GitHub Discussions

### Issue Template

When creating an issue, include:

```markdown
**FGC Version:** 0.1.0

**Platform:** Linux x86_64, Ubuntu 22.04

**Description:**
Brief description of the issue.

**Steps to Reproduce:**
1. Create GC with config...
2. Allocate objects...
3. Trigger GC...
4. See error...

**Expected Behavior:**
What should happen.

**Actual Behavior:**
What actually happens.

**Code Sample:**
```rust
// Minimal reproduction code
```

**Logs:**
```
[GC] ...
```

**Additional Context:**
Any other relevant information.
```
