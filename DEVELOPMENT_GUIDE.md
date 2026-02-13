# Fax-lang Development Guide

Fax-lang adalah sistem bahasa pemrograman modular berkinerja tinggi yang memanfaatkan kekuatan dari berbagai bahasa sistem modern: Rust, Zig, Haskell, C++, dan Python.

## Arsitektur

Compiler Fax-lang terdiri dari beberapa komponen independen:

- **Lexer** (Rust): Tokenisasi berkecepatan tinggi dan penanganan UTF-8
- **Parser** (Zig): Parsing Recursive Descent yang efisien dalam hal memori
- **Semantic Analysis** (Haskell): Pemeriksaan tipe dan validasi semantik yang ketat
- **Optimizer** (Rust): Transformasi AST berbasis graf
- **Code Generation** (C++): Generasi LLVM IR dan emisi Stack Map
- **Runtime** (Zig): **Fgc**: Garbage Collector bergaya ZGC dengan Mark-Relocate

## Instalasi Dependensi

Jalankan skrip berikut untuk menginstal semua dependensi yang diperlukan:

```bash
chmod +x install_dependencies.sh
./install_dependencies.sh
```

## Pembuatan Compiler

Setelah dependensi terinstal, jalankan:

```bash
chmod +x build_compiler.sh
./build_compiler.sh
```

## Pengujian

Uji compiler dengan menjalankan:

```bash
./run_pipeline.sh simple_test.fax
```

## Struktur Direktori

```
faxc/                   # Root compiler
├── packages/           # Komponen compiler
│   ├── lexer/         # Rust - Lexer
│   ├── parser/        # Zig - Parser
│   ├── sema/          # Haskell - Semantic Analysis
│   ├── optimizer/     # Rust - Optimizer
│   ├── codegen/       # C++ - Code Generator
│   └── runtime/       # Zig - Runtime
└── ...
```

## Kontribusi

Lihat CONTRIBUTING.md untuk pedoman kontribusi lebih lanjut.