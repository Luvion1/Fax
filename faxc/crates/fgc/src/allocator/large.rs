//! Large Object Allocator
//!
//! Allocator khusus untuk object besar (> 4KB).
//! Large objects mendapat dedicated region untuk menghindari fragmentasi.
//!
//! Karakteristik large objects:
//! - Ukuran > 4KB (configurable)
//! - Jarang dialokasikan
//! - Seringkali long-lived
//! - Tidak cocok untuk bump pointer (waste space)
//!
//! Strategy:
//! - Setiap large object mendapat dedicated region
//! - Region size = object size + overhead
//! - Region di-manage terpisah dari small/medium objects

use crate::error::{FgcError, Result};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Threshold untuk large object (default 4KB)
pub const LARGE_THRESHOLD: usize = 4 * 1024;

/// Minimum alignment untuk large objects
pub const LARGE_ALIGNMENT: usize = 4096; // Page aligned

/// LargeObjectAllocator - allocator untuk object besar
///
/// Mengelola allocation untuk object > LARGE_THRESHOLD.
/// Setiap object mendapat dedicated region.
pub struct LargeObjectAllocator {
    /// Free regions yang bisa digunakan
    /// Map: size -> list of addresses
    free_regions: std::sync::Mutex<BTreeMap<usize, Vec<usize>>>,

    /// Allocated objects: address -> size
    allocated: std::sync::Mutex<BTreeMap<usize, usize>>,

    /// Total bytes allocated
    total_allocated: AtomicUsize,

    /// Count allocated objects
    object_count: AtomicUsize,

    /// Minimum region size (page aligned)
    min_region_size: usize,
}

impl LargeObjectAllocator {
    /// Create new large object allocator
    pub fn new() -> Self {
        Self {
            free_regions: std::sync::Mutex::new(BTreeMap::new()),
            allocated: std::sync::Mutex::new(BTreeMap::new()),
            total_allocated: AtomicUsize::new(0),
            object_count: AtomicUsize::new(0),
            min_region_size: LARGE_ALIGNMENT,
        }
    }

    /// Allocate large object
    ///
    /// # Arguments
    /// * `size` - Size dalam bytes (harus > LARGE_THRESHOLD)
    ///
    /// # Returns
    /// Address dari allocated memory
    pub fn allocate(&self, size: usize) -> Result<usize> {
        if size < LARGE_THRESHOLD {
            return Err(FgcError::TlabError(
                format!("Size {} too small for large allocator", size)
            ));
        }

        // Align size ke page boundary
        let aligned_size = self.align_size(size);

        // Try reuse existing free region
        let address = self.find_or_create_region(aligned_size)?;

        // Track allocation
        {
            let mut allocated = self.allocated.lock().unwrap();
            allocated.insert(address, aligned_size);
        }

        self.total_allocated.fetch_add(aligned_size, Ordering::Relaxed);
        self.object_count.fetch_add(1, Ordering::Relaxed);

        Ok(address)
    }

    /// Free large object
    ///
    /// # Arguments
    /// * `address` - Address dari object yang akan di-free
    pub fn free(&self, address: usize) -> Result<()> {
        let size = {
            let mut allocated = self.allocated.lock().unwrap();
            allocated.remove(&address).ok_or_else(|| {
                FgcError::InvalidPointer { address }
            })?
        };

        // Add ke free list untuk reuse
        {
            let mut free_regions = self.free_regions.lock().unwrap();
            free_regions
                .entry(size)
                .or_insert_with(Vec::new)
                .push(address);
        }

        self.total_allocated.fetch_sub(size, Ordering::Relaxed);
        self.object_count.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }

    /// Find atau create region untuk size tertentu
    fn find_or_create_region(&self, size: usize) -> Result<usize> {
        let mut free_regions = self.free_regions.lock().unwrap();

        // Cari free region yang cukup besar
        for (&region_size, addresses) in free_regions.iter_mut() {
            if region_size >= size && !addresses.is_empty() {
                let address = addresses.pop().unwrap();

                // Jika region lebih besar, split (optional optimization)
                if region_size > size {
                    // TODO: Implement region splitting
                    // Untuk sekarang, return entire region
                }

                // Clean up empty entries
                if addresses.is_empty() {
                    free_regions.remove(&region_size);
                }

                return Ok(address);
            }
        }

        // No suitable free region, allocate new
        self.allocate_new_region(size)
    }

    /// Allocate region baru dari heap
    fn allocate_new_region(&self, size: usize) -> Result<usize> {
        // Note: Dalam implementasi nyata, ini request dari heap manager
        // Untuk sekarang, return dummy address
        static NEXT_ADDRESS: AtomicUsize = AtomicUsize::new(0x1000_0000);

        let base = NEXT_ADDRESS.fetch_add(size, Ordering::SeqCst);

        // Align ke page boundary
        let aligned_base = (base + LARGE_ALIGNMENT - 1) & !(LARGE_ALIGNMENT - 1);

        Ok(aligned_base)
    }

    /// Align size ke page boundary
    fn align_size(&self, size: usize) -> usize {
        (size + self.min_region_size - 1) & !(self.min_region_size - 1)
    }

    /// Get total bytes allocated
    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    /// Get count allocated objects
    pub fn object_count(&self) -> usize {
        self.object_count.load(Ordering::Relaxed)
    }

    /// Get statistics tentang free regions
    pub fn free_region_stats(&self) -> (usize, usize) {
        let free_regions = self.free_regions.lock().unwrap();
        let total_free: usize = free_regions
            .iter()
            .map(|(&size, addrs)| size * addrs.len())
            .sum();
        let count_free: usize = free_regions.values().map(|v| v.len()).sum();
        (count_free, total_free)
    }

    /// Defragment large object heap
    ///
    /// Compact free regions untuk reduce fragmentation.
    pub fn defragment(&self) -> Result<()> {
        // TODO: Implement defragmentation
        // Merge adjacent free regions
        Ok(())
    }
}

impl Default for LargeObjectAllocator {
    fn default() -> Self {
        Self::new()
    }
}
