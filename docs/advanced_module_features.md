# Dokumentasi Fitur-Fitur Lanjutan Sistem Modul Fax

## 1. Namespace Lengkap

Sistem namespace memungkinkan penggunaan struktur modul hirarkis yang kompleks:

```fax
use std::collections::HashMap;
use std::io::print;
```

### Aliasing

Anda dapat memberikan alias pada modul atau simbol yang diimpor:

```fax
use std::io::print as display;
use std::collections as coll;
```

## 2. Submodule

Modul dapat memiliki submodule yang disusun dalam struktur hirarkis:

```
std/
├── io.fax
├── collections/
│   ├── vector.fax
│   └── hashmap.fax
└── util.fax
```

```fax
// Dalam file main.fax
use std::collections::vector::Vec;
use std::collections::hashmap::Map;
```

## 3. Visibilitas Tingkat Lanjut

Selain `pub`, sistem sekarang mendukung visibilitas tingkat lanjut:

```fax
pub(crate) fn internal_function() { ... }  // Hanya visible dalam crate yang sama
pub(super) fn parent_visible() { ... }    // Hanya visible dalam modul parent
pub(self) fn self_visible() { ... }       // Hanya visible dalam modul saat ini
```

## 4. Re-export Modul

Modul dapat mengekspor ulang simbol dari modul lain:

```fax
// Dalam file collections.fax
pub use std::collections::vector::Vec;
pub use std::collections::hashmap::Map as HashMap;
pub use std::collections::list::List as LinkedList;
```

## 5. Resolusi Simbol Lintas Modul

Sistem sekarang lebih baik dalam menangani referensi simbol dari modul lain:

```fax
use std::io::print;
use math::constants::PI;

fn main() {
    print(PI);
}
```

## 6. Deteksi Siklus Dependensi

Sistem sekarang dapat mendeteksi dan melaporkan siklus dependensi antar modul:

```fax
// Module A.fax
use B::function_from_b;

// Module B.fax
use A::function_from_a;
```

Kompiler akan melaporkan kesalahan: "Circular dependency detected: A -> B -> A"

## 7. Caching Modul

Modul yang telah dikompilasi akan di-cache untuk meningkatkan kecepatan kompilasi berikutnya. Modul hanya akan dikompilasi ulang jika file sumber berubah.

## 8. Standar Library

Fax sekarang memiliki standar library dengan modul-modul berikut:

- `std::io` - Fungsi-fungsi input/output
- `std::collections` - Struktur data koleksi
- `std::util` - Fungsi-fungsi utilitas umum

## 9. Generic Module (Dasar)

Sistem sekarang mendukung parameter tipe generik dalam definisi fungsi:

```fax
fn map<T, U>(arr: Vec<T>, func: fn(T) -> U) -> Vec<U> {
    // Implementasi fungsi generik
}
```

## Contoh Penggunaan Lengkap

Berikut adalah contoh penggunaan semua fitur baru:

```fax
// math/operations.fax
module math::operations;

pub fn add<T>(a: T, b: T) -> T {
    // Implementasi penjumlahan generik
}

pub fn multiply<T>(a: T, b: T) -> T {
    // Implementasi perkalian generik
}

// math/constants.fax
module math::constants;

pub const PI: f64 = 3.14159;
pub const E: f64 = 2.71828;

// main.fax
use math::operations::{add, multiply as mult};
use math::constants::{PI as pi_constant, E};
use std::io::print;

pub(crate) fn calculate_area(radius: f64) -> f64 {
    mult(mult(pi_constant, radius), radius)
}

fn main() {
    let area = calculate_area(5.0);
    print(area);
}
```

## Konfigurasi Pipeline

Pipeline kompilasi sekarang secara otomatis:

1. Mendeteksi dependensi modul dari pernyataan `use`
2. Memeriksa adanya siklus dependensi
3. Menggunakan cache jika modul belum berubah
4. Mengkompilasi modul dependensi terlebih dahulu
5. Menyertakan modul standar dari direktori `std/`