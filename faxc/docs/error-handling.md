# Error Handling Guide - FGC (Fax Garbage Collector)

Dokumentasi ini menjelaskan strategi error handling di FGC dan bagaimana error di-handle di seluruh codebase.

## Daftar Isi

1. [Error Types](#error-types)
2. [Error Handling Strategy](#error-handling-strategy)
3. [Error Propagation](#error-propagation)
4. [Error Recovery](#error-recovery)
5. [Best Practices](#best-practices)

---

## Error Types

### FgcError - Main Error Type

`FgcError` adalah error type utama untuk semua operasi FGC:

```rust
pub enum FgcError {
    /// Out of memory - allocation gagal
    OutOfMemory {
        requested: usize,
        available: usize,
    },

    /// Heap initialization gagal
    HeapInitialization(String),

    /// Invalid pointer address
    InvalidPointer { address: usize },

    /// Region allocation gagal
    RegionAllocationFailed { reason: String },

    /// Concurrent modification detected
    ConcurrentModification { operation: String },

    /// GC cycle gagal
    GcCycleFailed { reason: String },

    /// Marking phase gagal
    MarkingFailed(String),

    /// Relocation phase gagal
    RelocationFailed(String),

    /// Forwarding table error
    ForwardingTableError(String),

    /// TLAB error
    TlabError(String),

    /// NUMA error
    NumaError(String),

    /// Virtual memory error
    VirtualMemoryError(String),

    /// Configuration error
    Configuration(String),

    /// Internal error (bug)
    Internal(String),
}
```

### ConfigError - Configuration Errors

Error spesifik untuk konfigurasi:

```rust
pub enum ConfigError {
    InvalidHeapSize(String),
    InvalidYoungRatio(String),
    InvalidTlabSize(String),
    InvalidRegionSize(String),
    InvalidThreshold(String),
    InvalidGcThreads(String),
    InvalidPauseTime(String),
}
```

---

## Error Handling Strategy

### 1. Result Type

Semua operasi yang bisa gagal mengembalikan `Result<T, FgcError>`:

```rust
impl GarbageCollector {
    pub fn new(config: GcConfig) -> Result<Self, FgcError> {
        // ...
    }

    pub fn collect(&self) -> Result<GcStats, FgcError> {
        // ...
    }
}
```

### 2. Error Conversion

Gunakan `?` operator untuk error propagation:

```rust
pub fn allocate(&self, size: usize) -> Result<usize, FgcError> {
    let region = self.allocate_region(size)?; // ? propagate error
    Ok(region.start())
}
```

### 3. Error Context

Tambahkan context untuk debugging:

```rust
region.allocate(size).map_err(|e| {
    FgcError::RegionAllocationFailed {
        reason: format!("Failed to allocate in region: {}", e),
    }
})?;
```

---

## Error Propagation

### Module-Level Propagation

```
Runtime
    └── GC
        ├── Allocator
        │   ├── Bump Allocator
        │   ├── TLAB
        │   └── Large Object
        ├── Heap
        │   ├── Region
        │   └── Virtual Memory
        └── Marker
            ├── Mark Queue
            └── Bitmap
```

Error propagate dari bottom-up:

```rust
// Virtual Memory error
fn commit(&mut self, offset: usize, size: usize) -> Result<(), FgcError> {
    if offset + size > self.reserved_size {
        return Err(FgcError::VirtualMemoryError(
            "Commit exceeds reserved size".to_string()
        ));
    }
    Ok(())
}

// Heap error (propagates from Virtual Memory)
fn create_new_region(&self, region_type: RegionType) -> Result<Arc<Region>, FgcError> {
    self.virtual_memory.commit(offset, size)?; // Propagate VirtualMemoryError
    Ok(region)
}

// GC error (propagates from Heap)
fn allocate_region(&self, size: usize) -> Result<Arc<Region>, FgcError> {
    self.create_new_region(region_type)?; // Propagate Heap error
    Ok(region)
}
```

---

## Error Recovery

### Recoverable Errors

Beberapa error bisa di-recover:

1. **OutOfMemory** - Trigger GC dan retry
2. **RegionAllocationFailed** - Try different region type
3. **TLAB full** - Refill TLAB

```rust
pub fn allocate(&self, size: usize) -> Result<usize, FgcError> {
    // Try TLAB first
    if let Ok(addr) = self.tlab_allocate(size) {
        return Ok(addr);
    }

    // TLAB full, try bump allocator
    match self.bump_allocate(size) {
        Ok(addr) => Ok(addr),
        Err(FgcError::OutOfMemory { .. }) => {
            // GC dan retry
            self.gc.collect()?;
            self.bump_allocate(size)
        }
        Err(e) => Err(e),
    }
}
```

### Non-Recoverable Errors

Error yang tidak bisa di-recover:

1. **InvalidPointer** - Bug di program
2. **ConcurrentModification** - Race condition (bug)
3. **Internal** - Internal compiler error

Error ini harus di-panic atau return ke user.

---

## Best Practices

### 1. Gunakan Specific Error Types

```rust
// ❌ Bad: Generic error
fn allocate(&self, size: usize) -> Result<(), String> {
    Err("Failed".to_string())
}

// ✅ Good: Specific error
fn allocate(&self, size: usize) -> Result<(), FgcError> {
    Err(FgcError::OutOfMemory {
        requested: size,
        available: 0,
    })
}
```

### 2. Tambahkan Context

```rust
// ❌ Bad: No context
region.allocate(size)?;

// ✅ Good: With context
region.allocate(size).map_err(|e| {
    FgcError::RegionAllocationFailed {
        reason: format!("Region {:?}: {}", region.id(), e),
    }
})?;
```

### 3. Handle Errors di Appropriate Level

```rust
// Low-level: Return error
fn allocate_memory(&self, size: usize) -> Result<usize, FgcError> {
    // ...
}

// High-level: Handle or log
fn user_allocate(&self, size: usize) -> Option<usize> {
    match self.allocate_memory(size) {
        Ok(addr) => Some(addr),
        Err(FgcError::OutOfMemory { .. }) => {
            log::warn!("Out of memory, triggering GC");
            self.gc.collect().ok()?;
            self.allocate_memory(size).ok()
        }
        Err(e) => {
            log::error!("Allocation failed: {}", e);
            None
        }
    }
}
```

### 4. Document Error Cases

```rust
/// Allocate memory untuk object baru.
///
/// # Errors
///
/// Returns `Err(FgcError::OutOfMemory)` jika heap penuh.
/// Returns `Err(FgcError::InvalidSize)` jika size = 0.
///
/// # Examples
///
/// ```
/// let addr = allocator.allocate(64)?;
/// ```
pub fn allocate(&self, size: usize) -> Result<usize, FgcError> {
    // ...
}
```

---

## Error Handling Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      User Application                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                         Runtime                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Handle errors, log, retry, or return to user        │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    GarbageCollector                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Coordinate GC phases, propagate errors up           │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌─────────────────┐
│  Allocator    │   │     Heap        │   │    Marker       │
│               │   │                 │   │                 │
│ - OOM         │   │ - Heap Init     │   │ - Marking       │
│ - TLAB Full   │   │ - Region Alloc  │   │ - Root Scan     │
│ - Large Obj   │   │ - Virtual Mem   │   │ - Bitmap        │
└───────────────┘   └─────────────────┘   └─────────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
                  ┌───────────────────────┐
                  │    FgcError Types     │
                  │                       │
                  │ - OutOfMemory         │
                  │ - InvalidPointer      │
                  │ - RegionAllocFailed   │
                  │ - MarkingFailed       │
                  │ - etc...              │
                  └───────────────────────┘
```

---

## Testing Error Handling

### Unit Tests

```rust
#[test]
fn test_out_of_memory() {
    let config = GcConfig {
        max_heap_size: 1024,
        ..Default::default()
    };

    let gc = GarbageCollector::new(config).unwrap();

    // Allocate until OOM
    let result = gc.allocate(2048);
    assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
}

#[test]
fn test_invalid_config() {
    let config = GcConfig {
        max_heap_size: 0, // Invalid
        ..Default::default()
    };

    let result = GarbageCollector::new(config);
    assert!(matches!(result, Err(FgcError::Configuration(_))));
}
```

### Integration Tests

```rust
#[test]
fn test_gc_recovery_from_oom() {
    let runtime = Runtime::new(GcConfig::default()).unwrap();
    runtime.start().unwrap();

    // Fill heap
    for _ in 0..1000 {
        runtime.allocate(1024).unwrap();
    }

    // GC should recover
    runtime.request_gc(GcGeneration::Full);

    // Should be able to allocate again
    let addr = runtime.allocate(1024);
    assert!(addr.is_ok());
}
```

---

## Summary

1. **Gunakan `FgcError`** untuk semua error di FGC
2. **Propagate errors** dengan `?` operator
3. **Tambahkan context** untuk debugging
4. **Handle recoverable errors** di appropriate level
5. **Document error cases** di docstrings
6. **Test error scenarios** di unit dan integration tests
