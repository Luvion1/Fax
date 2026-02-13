# Memory Management

Fax implements a **Precise Generational Garbage Collector** (FGC) in Zig.

## Object Layout

Every heap object has a 16-byte header:
- `magic`: Integrity validation.
- `forward`: Forwarding pointer for moving GC.
- `metadata`: Size and color (for tri-color marking).

## Pointer Maps

Unlike conservative GCs, FGC is **precise**. The compiler generates a "Pointer Map" for every struct, telling the GC exactly where pointers are located. This prevents memory leaks from misidentified integers.

## Write Barriers

When a pointer in an old-generation object is updated to point to a young-generation object, a **Write Barrier** is triggered. This ensures the GC doesn't miss references during incremental marking.

```fax
struct Node { next: Node, val: i64 }

fn main() {
  let mut root = Node { next: null, val: 0 };
  collect_gc(); // promotes root to old generation
  
  // Write barrier triggered here
  root.next = Node { next: null, val: 1 }; 
}
```
