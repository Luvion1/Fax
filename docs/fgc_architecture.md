# Fgc (Fax Garbage Collector) Architecture

Fgc is a custom ZGC-inspired Garbage Collector implemented in Zig for the Fax-lang project. It is a **Colored, Mark-Relocate, Concurrent-ready** collector.

## Key Features

- **ZGC-inspired Coloring:** Uses bit flags (`M0`, `M1`, `Remapped`) on pointers/objects to track state.
- **Mark-Relocate:** During collection, live objects are moved to new pages to eliminate fragmentation.
- **Load Barrier:** A critical component that ensures pointers are redirected to the new location of an object if it has been moved.
- **Polyglot Integration:** Exposed via C ABI (`fax_fgc_...`) to be used by LLVM-generated code (C++ codegen) and Rust tests.

## Lifecycle Phases

1. **Idle:** Normal execution.
2. **Marking:** Traverses roots (stack and globals) to identify live objects.
3. **Relocating:** Moves live objects to new memory pages.
4. **Remapping:** Updates pointers to point to the new object locations.

## Interface (C ABI)

- `fax_fgc_init()`: Initializes the global Fgc instance.
- `fax_fgc_alloc(size, ptr_map, len)`: Allocates a new managed object.
- `fax_fgc_collect()`: Manually triggers a GC cycle.
- `fax_fgc_register_root(ptr, slot)`: Registers a global pointer as a GC root.
- `fax_fgc_push_frame(frame)`: Pushes a stack frame for local root tracking.
- `fax_fgc_pop_frame()`: Pops the current stack frame.

## Implementation Details

- **File:** `faxc/src/runtime/fgc.zig`
- **Testing:** `faxc/src/runtime/fgc_test.rs` (Rust) and internal Zig tests.
