//! Allocator Submodule - Bump Pointer Allocation
//!
//! Bump pointer allocator adalah teknik allocation paling cepat.
//! Allocation hanya memerlukan satu atomic increment operation.
//!
//! Cara kerja:
//! 1. Region dialokasikan dari heap
//! 2. Pointer di-set ke awal region
//! 3. Untuk allocate: increment pointer, return alamat lama
//! 4. Region penuh: allocate region baru
//!
//! Kecepatan: O(1) - constant time terlepas dari ukuran atau fragmentation
//!
//! Keterbatasan:
//! - Tidak bisa free individual objects
//! - Region harus di-reset seluruhnya
//! - Cocok untuk generational GC (young gen)

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

/// BumpPointerAllocator - fast bump pointer allocator
///
/// Allocator untuk region tunggal menggunakan bump pointer technique.
/// Thread-safe menggunakan atomic operations.
pub struct BumpPointerAllocator {
    /// Start address dari region
    start: AtomicUsize,

    /// Current bump pointer (next free address)
    top: AtomicUsize,

    /// End address dari region
    end: AtomicUsize,

    /// Alignment requirement (biasanya 8 bytes)
    alignment: usize,
}

impl BumpPointerAllocator {
    /// Create new bump allocator untuk region [start, end)
    ///
    /// # Arguments
    /// * `start` - Start address region
    /// * `end` - End address region (exclusive)
    /// * `alignment` - Alignment requirement untuk allocated objects
    pub fn new(start: usize, end: usize, alignment: usize) -> Self {
        Self {
            start: AtomicUsize::new(start),
            top: AtomicUsize::new(start),
            end: AtomicUsize::new(end),
            alignment,
        }
    }

    /// Allocate memory dengan size tertentu
    ///
    /// Fast path: single atomic increment
    /// Slow path: alignment adjustment
    ///
    /// # Arguments
    /// * `size` - Size dalam bytes yang dialokasikan
    ///
    /// # Returns
    /// Address dari allocated memory, atau error jika region penuh
    pub fn allocate(&self, size: usize) -> Result<usize> {
        // Align size ke alignment boundary
        let aligned_size = self.align_size(size);

        // Fast path: try allocate dengan CAS
        let mut current_top = self.top.load(Ordering::Relaxed);

        loop {
            let new_top = current_top + aligned_size;

            // Check jika region penuh
            let end_val = self.end.load(Ordering::Relaxed);
            if new_top > end_val {
                return Err(FgcError::OutOfMemory {
                    requested: size,
                    available: end_val - current_top,
                });
            }

            // Try CAS untuk update top pointer
            match self.top.compare_exchange_weak(
                current_top,
                new_top,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Success! Return alamat lama (sebelum di-bump)
                    return Ok(current_top);
                }
                Err(actual) => {
                    // CAS gagal, thread lain allocate duluan
                    // Retry dengan nilai actual
                    current_top = actual;
                }
            }
        }
    }

    /// Reset allocator ke awal region
    ///
    /// Dipanggil setelah GC reclaim semua objects di region.
    /// Aman hanya jika tidak ada thread yang sedang allocate.
    pub fn reset(&self) {
        self.top.store(self.start.load(Ordering::Relaxed), Ordering::SeqCst);
    }

    /// Get remaining space di region
    pub fn remaining(&self) -> usize {
        let current_top = self.top.load(Ordering::Relaxed);
        let end_val = self.end.load(Ordering::Relaxed);
        end_val - current_top
    }

    /// Get total capacity region
    pub fn capacity(&self) -> usize {
        let end_val = self.end.load(Ordering::Relaxed);
        let start_val = self.start.load(Ordering::Relaxed);
        end_val - start_val
    }

    /// Get bytes already allocated
    pub fn allocated(&self) -> usize {
        let current_top = self.top.load(Ordering::Relaxed);
        let start_val = self.start.load(Ordering::Relaxed);
        current_top - start_val
    }

    /// Check jika allocator penuh
    pub fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    /// Align size ke boundary
    fn align_size(&self, size: usize) -> usize {
        // Round up ke multiple dari alignment
        (size + self.alignment - 1) & !(self.alignment - 1)
    }

    /// Set bump pointer ke address tertentu
    ///
    /// Hanya untuk internal GC use.
    /// Tidak thread-safe, harus dipanggil saat tidak ada allocation.
    pub fn set_top(&self, address: usize) {
        let start_val = self.start.load(Ordering::Relaxed);
        let end_val = self.end.load(Ordering::Relaxed);
        if address >= start_val && address <= end_val {
            self.top.store(address, Ordering::SeqCst);
        }
    }
}

/// MultiBumpAllocator - multiple bump regions untuk concurrency
///
/// Mengelola multiple bump pointer regions untuk mengurangi contention.
/// Setiap thread bisa dapat region sendiri untuk lock-free allocation.
pub struct MultiBumpAllocator {
    /// List regions yang tersedia
    regions: std::sync::Mutex<Vec<BumpPointerAllocator>>,

    /// Region size untuk setiap bump allocator
    region_size: usize,

    /// Alignment requirement
    alignment: usize,

    /// Maximum regions yang boleh dibuat
    max_regions: usize,
}

impl MultiBumpAllocator {
    /// Create new multi-region bump allocator
    ///
    /// # Arguments
    /// * `region_size` - Size setiap region dalam bytes
    /// * `alignment` - Alignment requirement
    /// * `max_regions` - Maximum jumlah regions
    pub fn new(region_size: usize, alignment: usize, max_regions: usize) -> Self {
        Self {
            regions: std::sync::Mutex::new(Vec::new()),
            region_size,
            alignment,
            max_regions,
        }
    }

    /// Allocate dari salah satu region
    ///
    /// Strategy:
    /// 1. Try allocate dari region existing (lock-free)
    /// 2. Jika semua penuh, create region baru
    /// 3. Jika max regions tercapai, return error
    pub fn allocate(&self, size: usize) -> Result<usize> {
        // Try existing regions first
        {
            let regions = self.regions.lock().unwrap();

            // Coba setiap region (bisa di-optimize dengan per-thread region)
            for region in regions.iter() {
                if let Ok(addr) = region.allocate(size) {
                    return Ok(addr);
                }
            }
        }

        // Semua region penuh, buat region baru
        self.allocate_new_region(size)
    }

    /// Allocate region baru dan coba allocate
    fn allocate_new_region(&self, size: usize) -> Result<usize> {
        let mut regions = self.regions.lock().unwrap();

        // Check max regions
        if regions.len() >= self.max_regions {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        // Create region baru
        // Note: Dalam implementasi nyata, address ini harus dari heap manager
        let base_address = 0x1000 * (regions.len() as usize + 1); // Dummy address
        let region = BumpPointerAllocator::new(
            base_address,
            base_address + self.region_size,
            self.alignment,
        );

        // Allocate dari region baru
        let addr = region.allocate(size)?;

        regions.push(region);

        Ok(addr)
    }

    /// Reset semua regions
    pub fn reset_all(&self) {
        let regions = self.regions.lock().unwrap();
        for region in regions.iter() {
            region.reset();
        }
    }

    /// Get total allocated bytes dari semua regions
    pub fn total_allocated(&self) -> usize {
        let regions = self.regions.lock().unwrap();
        regions.iter().map(|r| r.allocated()).sum()
    }

    /// Get total capacity dari semua regions
    pub fn total_capacity(&self) -> usize {
        let regions = self.regions.lock().unwrap();
        regions.len() * self.region_size
    }
}
