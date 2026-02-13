# Sistem Modul Lengkap dalam Bahasa Pemrograman Fax

## Ikhtisar

Sistem modul dalam bahasa pemrograman Fax menyediakan mekanisme untuk mengorganisasi kode ke dalam unit-unit yang dapat digunakan kembali dan memungkinkan pembuatan proyek skala besar. Sistem ini mendukung deklarasi modul, ekspor/import simbol, dan resolusi dependensi lintas modul.

## Fitur Utama

### 1. Deklarasi Modul
```fax
module nama_modul;

// Isi modul di sini
```

### 2. Ekspor Simbol Publik
Gunakan kata kunci `pub` untuk membuat fungsi, konstanta, variabel, atau struktur tersedia untuk modul lain:

```fax
pub fn fungsi_publik(a: i64, b: i64) -> i64 {
    return a + b;
}

pub const KONSTANTA_PUBLIK = 42;

pub struct StrukturPublik {
    nilai: i64
}
```

### 3. Impor dari Modul Lain
Gunakan pernyataan `use` untuk mengimpor simbol dari modul lain:

```fax
use nama_modul::simbol_yang_diimpor;
use nama_modul::*;  // Impor semua simbol publik (wildcard)
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

## Arsitektur Implementasi

### 1. Parser (Zig)
- Mengenali pernyataan `module`, `use`, dan `pub`
- Membangun struktur data untuk melacak modul dan ekspor
- Menghasilkan AST yang mencerminkan struktur modul

### 2. Semantic Analyzer (Haskell)
- Memvalidasi visibilitas simbol
- Melakukan resolusi nama lintas modul
- Memastikan hanya simbol publik yang dapat diakses dari luar modul

### 3. Code Generator (C++)
- Menghasilkan kode LLVM IR untuk referensi lintas modul
- Menangani resolusi simbol yang diimpor

### 4. Module Resolver (Rust)
- Menyelesaikan jalur modul dari nama modul
- Membangun graf dependensi antar modul
- Menyediakan caching untuk modul yang telah diproses

### 5. Pipeline Compiler (TypeScript)
- Mengelola urutan kompilasi berdasarkan dependensi
- Memastikan modul dependensi dikompilasi terlebih dahulu
- Mengintegrasikan semua komponen dalam alur kerja kompilasi

## Resolusi Jalur Modul

Sistem mencari modul dalam urutan berikut:
1. Direktori yang sama dengan file sumber
2. Direktori `faxc/tests/fax`
3. Direktori `tests`
4. Direktori saat ini

Nama file modul harus sesuai dengan pola `{nama_modul}.fax`.

## Manajemen Dependensi

- Sistem otomatis mendeteksi dependensi dari pernyataan `use`
- Modul dependensi dikompilasi sebelum modul yang bergantung padanya
- Dukungan untuk mendeteksi siklus dependensi (belum diimplementasikan sepenuhnya)

## Contoh Penggunaan

### File `math.fax`:
```fax
module math;

pub fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

pub fn multiply(a: i64, b: i64) -> i64 {
    return a * b;
}

pub const PI = 3.14159;
```

### File `geometry.fax`:
```fax
module geometry;

use math::multiply;
use math::PI;

pub fn area_circle(radius: i64) -> i64 {
    return multiply(multiply(radius, radius), PI as i64);
}
```

### File `main.fax`:
```fax
use math::add;
use geometry::area_circle;

fn main() {
    let sum = add(10, 20);
    print(sum);
    
    let area = area_circle(5);
    print(area);
}
```

## Status Implementasi

Fitur-fitur berikut telah diimplementasikan:
- ✅ Deklarasi modul dengan pernyataan `module`
- ✅ Ekspor simbol publik dengan kata kunci `pub`
- ✅ Impor simbol tunggal dengan pernyataan `use`
- ✅ Impor semua simbol publik dengan wildcard `use modul::*`
- ✅ Resolusi jalur modul
- ✅ Manajemen dependensi dasar
- ✅ Integrasi dengan pipeline kompilasi
- ✅ Resolusi simbol lintas modul (dasar)

Catatan: Resolusi simbol lintas modul secara penuh masih dalam pengembangan, terutama dalam code generation.

## Kontribusi Terhadap Proyek

Implementasi sistem modul lengkap ini secara signifikan meningkatkan kemampuan bahasa pemrograman Fax:

1. **Skalabilitas**: Memungkinkan pembuatan proyek besar dengan organisasi kode yang baik
2. **Reusabilitas**: Kode dapat dikemas dalam modul dan digunakan kembali
3. **Maintainability**: Memisahkan tanggung jawab antar komponen kode
4. **Keamanan**: Mekanisme visibilitas mencegah akses ke implementasi internal

## Kesimpulan

Sistem modul yang diimplementasikan menyediakan fondasi kuat untuk pengembangan perangkat lunak berskala besar dalam bahasa pemrograman Fax. Dengan integrasi penuh ke seluruh pipeline kompilasi, sistem ini memungkinkan pengorganisasian kode yang efektif dan manajemen dependensi yang terkelola dengan baik.