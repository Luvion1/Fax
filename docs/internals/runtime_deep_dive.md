# Runtime & Fgc Deep Dive

The Fax Runtime is a lightweight layer that provides memory management and basic I/O primitives. Its core is **Fgc**, implemented in `faxc/src/runtime/fgc.zig`.

## Memory Layout

Fgc manages memory in **Pages**:
- **Small Pages (2MB):** For objects <= 2KB.
- **Medium Pages (32MB):** For objects <= 256KB.
- **Large Pages:** Custom sized for massive allocations.

### Object Header
Every allocated object has a 16-byte aligned header:
- `magic` (8 bytes): `0xDEADC0DECAFEBABE` (for validation).
- `forwarding_ptr` (8 bytes): Used during the relocation phase.
- `size` (usize): Size of the data payload.
- `color` (u8): Current GC color (M0, M1, or Remapped).
- `obj_type` (u8): Integer, Struct, Array, or String.

## ZGC-inspired Tracing

Fgc uses a **Mark-Relocate** algorithm:

1. **Marking Phase:**
   - Traverses the stack (via `StackFrame` linked list) and global roots.
   - Uses a "color" bit. If an object's color doesn't match the current global color, it is marked as live and pushed to the marking stack.

2. **Relocation Phase:**
   - Live objects are copied to brand new pages.
   - A `forwarding_ptr` is left behind in the old location pointing to the new one.

3. **Load Barrier:**
   - When the mutator (the running Fax program) accesses a pointer, it checks the object's color.
   - If the color is "stale", the load barrier triggers, follows the `forwarding_ptr`, and updates the pointer to the new location.

## Stack Root Tracking

Since LLVM does not natively provide a GC-safe stack map without complex plugins, Fax uses a manual **Shadow Stack**:
- Functions push a `StackFrame` onto a global linked list at the start.
- Local variables that hold pointers are registered in this frame.
- Functions pop the frame before returning.

This ensures Fgc can always find live pointers even if they are stored in CPU registers or stack slots.
