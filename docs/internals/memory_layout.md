# Memory Layout & ABI Specification

This document provides a low-level specification of how Fax-lang organizes data in memory. This is critical for maintaining compatibility between the C++ Codegen and the Zig Runtime (Fgc).

## 1. Object Header Layout (16 Bytes)

All objects allocated on the heap MUST follow this 16-byte header structure. Memory must be 16-byte aligned.

| Offset | Size | Field | Description |
| :--- | :--- | :--- | :--- |
| 0x00 | 8 | `magic` | Constant `0xDEADC0DECAFEBABE`. Changes to `0xF084A4D146000000` when forwarded. |
| 0x08 | 8 | `forwarding_ptr` | Pointer to the new location during GC relocation. Null otherwise. |
| 0x10 | 8 | `size` | Size of the data payload in bytes (excluding header). |
| 0x18 | 4 | `ptr_map_ptr` | Offset/Pointer to the metadata describing which fields are pointers. |
| 0x1C | 1 | `obj_type` | `0: Integer`, `1: Struct`, `2: Array`, `3: String`. |
| 0x1D | 1 | `color` | GC Color bits: `0x1 (M0)`, `0x2 (M1)`, `0x4 (Remapped)`. |
| 0x1E | 1 | `flags` | Bit 0: `IS_PTR_ARRAY`. |
| 0x1F | 1 | `unused` | Reserved for future alignment/padding. |

## 2. Primitive Types

| Type | Size (Bits) | LLVM Equivalent | Alignment |
| :--- | :--- | :--- | :--- |
| `Bool` | 1 | `i1` | 1 byte |
| `Byte` | 8 | `i8` | 1 byte |
| `Int` | 64 | `i64` | 8 bytes |
| `Float` | 64 | `double` | 8 bytes |
| `String` | 64 | `i8*` (Managed) | 8 bytes |

## 3. Struct Layout

Structs in Fax are always heap-allocated and managed by Fgc. 
- Fields are ordered exactly as declared in the source code.
- Padding is inserted by the C++ Codegen to ensure 8-byte alignment for all fields.
- Pointers within structs MUST be registered in the `ptr_map` so the GC can trace them.

## 4. The Shadow Stack (Stack Frame ABI)

Since Fax uses a moving GC, all stack roots must be tracked. The Codegen emits code to manage a linked list of `StackFrame` structures.

```cpp
struct StackFrame {
    StackFrame* next;       // Pointer to caller's frame
    void** roots;          // Array of pointers to managed objects
    uint64_t root_count;   // Number of roots in this frame
};
```

### Prologue Sequence:
1. Allocate `StackFrame` on the native stack.
2. Set `next` to the current `stack_top`.
3. Set `stack_top` to the new frame.
4. Initialize `roots` array with nulls.

### Epilogue Sequence:
1. Set `stack_top` to `current_frame->next`.
2. Return.
