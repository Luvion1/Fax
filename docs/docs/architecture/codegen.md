---
sidebar_position: 6
---

# Codegen

**Location**: `faxc/packages/codegen/`

The Code Generator is implemented in **C++ (GCC 15.2.0)** and generates LLVM IR.

## Features

- **LLVM IR Generation**: Generates valid LLVM IR
- **Type Mapping**: Fax types → LLVM types
- **Function Calls**: Function call ABI
- **Memory Operations**: Allocations, access

## Output

```llvm
define i64 @main() {
entry:
  %0 = call i64 @add(i64 5, i64 3)
  ret i64 %0
}
```

## Build

```bash
cd faxc/packages/codegen
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make
```
