---
sidebar_position: 7
---

# Runtime

**Location**: `faxc/packages/runtime/`

The Runtime is implemented in **Zig 0.14.1** and provides the execution environment.

## Features

- **FGC (Fax Garbage Collector)**: Generational GC
- **Memory Management**: Allocation, deallocation
- **FFI Exports**: C ABI compatibility
- **Built-in Functions**: print, etc.

## FGC - Fax Garbage Collector

FGC is a **generational garbage collector** with:

- **Young Generation**: Short-lived objects
- **Old Generation**: Long-lived objects
- **Mark-Sweep**: For old generation
- **Copying**: For young generation

## Build

```bash
cd faxc/packages/runtime
zig build -Doptimize=ReleaseSafe
```

## Key Files

- `src/gc/fgc.zig` - GC implementation
- `src/main.zig` - Entry point
- `src/api/exports.zig` - C exports
