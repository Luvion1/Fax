# FFI & ABI Compatibility

Fax is designed to interoperate with C. This is essential for accessing system APIs, graphics libraries (like OpenGL), and the Fgc runtime itself.

## 1. Calling C from Fax

Fax uses the `extern` keyword to declare external C functions.

```fax
extern "C" {
    fn printf(format: String, ...): Int
}

fn main() {
    printf("Hello from C FFI
")
}
```

### ABI Mapping:
- **Fax `Int`** maps to **C `int64_t`**.
- **Fax `String`** maps to **C `char*`**. Note: Strings are null-terminated by Fgc.
- **Fax `Bool`** maps to **C `bool`** (usually `i8` or `i32` depending on platform).

## 2. Calling Fax from C

Fax functions are mangled by default. To make a Fax function callable from C, use the `export` attribute (planned).

Currently, all `fax_fgc_...` functions in Zig are exported with the `export` keyword to ensure they are visible to the C++ Codegen and LLVM Linker.

## 3. The "C" ABI Interface

The Fgc runtime is the best example of Fax's FFI capabilities.
- **Zig (Runtime)**: Exports symbols like `fax_fgc_alloc`.
- **C++ (Codegen)**: Declares these symbols as `extern "C"`.
- **LLVM**: Links them together into a single binary.

## 4. Platform Specifics

### Linux (x86_64):
- Follows the System V AMD64 ABI.
- First 6 integer arguments are passed in `rdi, rsi, rdx, rcx, r8, r9`.
- Return value in `rax`.

The C++ Codegen is responsible for ensuring the generated LLVM IR adheres to these conventions.
