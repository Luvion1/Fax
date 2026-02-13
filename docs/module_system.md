# Sistem Modul dalam Bahasa Pemrograman Fax

## Ikhtisar

Sistem modul dalam bahasa pemrograman Fax memungkinkan pengorganisasian kode ke dalam unit-unit yang dapat digunakan kembali. Sistem ini mendukung deklarasi modul, ekspor item publik, dan impor dari modul lain.

## Sintaks

### Deklarasi Modul
```fax
module nama_modul;
```

Contoh:
```fax
module math;

pub fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

pub const PI = 3.14159;
```

### Ekspor Item
Gunakan kata kunci `pub` untuk membuat fungsi, konstanta, atau variabel tersedia untuk modul lain:

```fax
pub fn fungsi_publik() {
    // ...
}

pub const KONSTANTA_PUBLIK = 42;
```

### Impor dari Modul Lain
Gunakan pernyataan `use` untuk mengimpor item dari modul lain:

```fax
use nama_modul::item_yang_diimpor;
use nama_modul::*;  // Impor semua item publik (wildcard)
```

Contoh:
```fax
use math::add;
use math::PI;

fn main() {
    let result = add(10, 20);
    print(result);
    print(PI);
}
```

## Implementasi

Sistem modul diimplementasikan di seluruh pipeline kompilasi:

1. **Lexer (Rust)**: Mengenali token `module`, `use`, dan `pub`
2. **Parser (Zig)**: Menguraikan pernyataan modul dan impor menjadi AST
3. **Semantic Analyzer (Haskell)**: Memvalidasi visibilitas dan resolusi nama
4. **Code Generator (C++)**: Menghasilkan kode yang sesuai untuk referensi lintas modul

## Status Implementasi

Fitur-fitur berikut telah diimplementasikan:
- ✅ Deklarasi modul dengan pernyataan `module`
- ✅ Ekspor item publik dengan kata kunci `pub`
- ✅ Impor item tunggal dengan pernyataan `use`
- ✅ Impor semua item publik dengan wildcard `use modul::*`
- ✅ Integrasi dengan pipeline kompilasi

Catatan: Resolusi simbol lintas modul secara penuh masih dalam pengembangan.

## Contoh Penggunaan

File `math.fax`:
```fax
module math;

pub fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

pub const PI = 3.14159;
```

File `main.fax`:
```fax
use math::add;
use math::PI;

fn main() {
    let result = add(10, 20);
    print(result);
    print(PI);
}
```