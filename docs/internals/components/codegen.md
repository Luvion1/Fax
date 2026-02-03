# Component: Codegen (C++)

The final stage of the compiler core, converting the Typed AST into LLVM Intermediate Representation (IR).

## Responsibility
- Map Fax constructs (Functions, Structs) to LLVM constructs.
- Emit **Fgc Load Barriers**.
- Manage the **Shadow Stack** for Garbage Collection.

## LLVM Integration
Codegen uses the LLVM C++ API (or manual IR emission) to generate `.ll` files. It handles:
- **Calling Conventions**: Ensuring Fax functions can call C functions.
- **Pointer Tagging**: Preparing pointers for Fgc.

## Fgc Integration (Critical)
The Codegen must emit calls to the runtime:
- `call i8* @fax_fgc_alloc(...)` for every object allocation.
- `call void @fax_fgc_register_root(...)` for stack variables.

## Example Output
```llvm
define i64 @main() {
entry:
    call void @fax_fgc_init()
    %obj = call i8* @fax_fgc_alloc(i64 16, i64* null, i64 0)
    ret i64 0
}
```
