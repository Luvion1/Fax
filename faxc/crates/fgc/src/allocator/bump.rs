//! Allocator Submodule - Bump Pointer Allocation
//!
//! Bump pointer allocator is the fastest allocation technique.
//! Allocation only requires a single atomic increment operation.

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

/// BumpPointerAllocator - fast bump pointer allocator
pub struct BumpPointerAllocator {
    start: AtomicUsize,
    top: AtomicUsize,
    end: AtomicUsize,
    alignment: usize,
}

impl BumpPointerAllocator {
    pub fn new(start: usize, end: usize, alignment: usize) -> Result<Self> {
        if start >= end {
            return Err(FgcError::InvalidArgument(format!(
                "start ({:#x}) must be less than end ({:#x})",
                start, end
            )));
        }
        if !alignment.is_power_of_two() {
            return Err(FgcError::InvalidArgument(format!(
                "alignment ({}) must be a power of two",
                alignment
            )));
        }
        if alignment > 1024 {
            return Err(FgcError::InvalidArgument(format!(
                "alignment ({}) is too large (max 1024)",
                alignment
            )));
        }
        let region_size = end - start;
        if region_size < alignment {
            return Err(FgcError::InvalidArgument(format!(
                "region size ({:#x}) must be at least alignment ({:#x})",
                region_size, alignment
            )));
        }

        Ok(Self {
            start: AtomicUsize::new(start),
            top: AtomicUsize::new(start),
            end: AtomicUsize::new(end),
            alignment,
        })
    }

    pub fn allocate(&self, size: usize) -> Result<usize> {
        let aligned_size = self.align_size(size)?;
        let mut current_top = self.top.load(Ordering::Relaxed);

        loop {
            let new_top = self.new_top(current_top, aligned_size, size)?;

            match self.try_update_top(current_top, new_top) {
                Ok(_) => return Ok(current_top),
                Err(_) => {
                    current_top = self.top.load(Ordering::Relaxed);
                },
            }
        }
    }

    #[inline]
    pub fn try_allocate_fast(&self, size: usize) -> Option<usize> {
        let aligned_size = (size + self.alignment - 1) & !(self.alignment - 1);

        let current_top = self.top.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        let new_top = current_top.checked_add(aligned_size)?;
        if new_top > end {
            return None;
        }

        if self
            .top
            .compare_exchange(current_top, new_top, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            Some(current_top)
        } else {
            None
        }
    }

    fn new_top(
        &self,
        current_top: usize,
        aligned_size: usize,
        requested_size: usize,
    ) -> Result<usize> {
        let new_top = current_top
            .checked_add(aligned_size)
            .ok_or(FgcError::OutOfMemory {
                requested: requested_size,
                available: 0,
            })?;

        let end_val = self.end.load(Ordering::Relaxed);
        if new_top > end_val {
            return Err(FgcError::OutOfMemory {
                requested: requested_size,
                available: end_val.saturating_sub(current_top),
            });
        }

        Ok(new_top)
    }

    fn try_update_top(&self, current: usize, new: usize) -> Result<()> {
        self.top
            .compare_exchange_weak(current, new, Ordering::SeqCst, Ordering::Relaxed)
            .map(|_| ())
            .map_err(|actual| {
                FgcError::LockPoisoned(format!("CAS failed, actual value: {:#x}", actual))
            })
    }

    pub fn reset(&self) {
        self.top
            .store(self.start.load(Ordering::Relaxed), Ordering::SeqCst);
    }

    pub fn remaining(&self) -> usize {
        let current_top = self.top.load(Ordering::Relaxed);
        let end_val = self.end.load(Ordering::Relaxed);
        end_val - current_top
    }

    pub fn capacity(&self) -> usize {
        let end_val = self.end.load(Ordering::Relaxed);
        let start_val = self.start.load(Ordering::Relaxed);
        end_val - start_val
    }

    pub fn allocated(&self) -> usize {
        let current_top = self.top.load(Ordering::Relaxed);
        let start_val = self.start.load(Ordering::Relaxed);
        current_top - start_val
    }

    pub fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    fn align_size(&self, size: usize) -> Result<usize> {
        const MAX_ALLOCATION: usize = 1024 * 1024 * 1024;
        if size > MAX_ALLOCATION {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        let mask = self.alignment.wrapping_sub(1);
        let aligned = size.wrapping_add(mask) & !mask;

        if aligned < size {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        Ok(aligned)
    }

    pub fn set_top(&self, address: usize) {
        let start_val = self.start.load(Ordering::Relaxed);
        let end_val = self.end.load(Ordering::Relaxed);
        if address >= start_val && address <= end_val {
            self.top.store(address, Ordering::SeqCst);
        }
    }
}

/// MultiBumpAllocator - multiple bump regions for concurrency
pub struct MultiBumpAllocator {
    regions: std::sync::Mutex<Vec<BumpPointerAllocator>>,
    region_size: usize,
    alignment: usize,
    max_regions: usize,
}

impl MultiBumpAllocator {
    pub fn new(region_size: usize, alignment: usize, max_regions: usize) -> Self {
        Self {
            regions: std::sync::Mutex::new(Vec::new()),
            region_size,
            alignment,
            max_regions,
        }
    }

    pub fn allocate(&self, size: usize) -> Result<usize> {
        {
            let regions = self.regions.lock().map_err(|e| {
                FgcError::LockPoisoned(format!("MultiBumpAllocator regions lock poisoned: {}", e))
            })?;
            for region in regions.iter() {
                if let Ok(addr) = region.allocate(size) {
                    return Ok(addr);
                }
            }
        }
        self.allocate_new_region(size)
    }

    fn allocate_new_region(&self, size: usize) -> Result<usize> {
        let mut regions = self.regions.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("MultiBumpAllocator regions lock poisoned: {}", e))
        })?;

        if regions.len() >= self.max_regions {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        let base_address = 0x1000 * (regions.len() + 1);
        let region = BumpPointerAllocator::new(
            base_address,
            base_address + self.region_size,
            self.alignment,
        )?;

        let addr = region.allocate(size)?;
        regions.push(region);
        Ok(addr)
    }

    pub fn reset_all(&self) -> Result<()> {
        let regions = self.regions.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("MultiBumpAllocator regions lock poisoned: {}", e))
        })?;
        for region in regions.iter() {
            region.reset();
        }
        Ok(())
    }

    pub fn total_allocated(&self) -> Result<usize> {
        let regions = self.regions.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("MultiBumpAllocator regions lock poisoned: {}", e))
        })?;
        Ok(regions.iter().map(|r| r.allocated()).sum())
    }

    pub fn total_capacity(&self) -> Result<usize> {
        let regions = self.regions.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("MultiBumpAllocator regions lock poisoned: {}", e))
        })?;
        Ok(regions.len() * self.region_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // BumpPointerAllocator Constructor Tests
    // ========================================================================

    #[test]
    fn test_bump_allocator_new_valid() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        assert_eq!(allocator.capacity(), 0x1000);
        assert_eq!(allocator.allocated(), 0);
        assert_eq!(allocator.remaining(), 0x1000);
    }

    #[test]
    fn test_bump_allocator_new_start_ge_end() {
        assert!(BumpPointerAllocator::new(0x2000, 0x1000, 8).is_err());
        assert!(BumpPointerAllocator::new(0x1000, 0x1000, 8).is_err());
    }

    #[test]
    fn test_bump_allocator_new_invalid_alignment_not_power_of_two() {
        assert!(BumpPointerAllocator::new(0x1000, 0x2000, 3).is_err());
        assert!(BumpPointerAllocator::new(0x1000, 0x2000, 15).is_err());
    }

    #[test]
    fn test_bump_allocator_new_invalid_alignment_too_large() {
        assert!(BumpPointerAllocator::new(0x1000, 0x2000, 2048).is_err());
    }

    #[test]
    fn test_bump_allocator_new_region_size_less_than_alignment() {
        assert!(BumpPointerAllocator::new(0x1000, 0x1004, 8).is_err());
    }

    // ========================================================================
    // BumpPointerAllocator Allocation Tests
    // ========================================================================

    #[test]
    fn test_bump_allocator_allocate_basic() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        let addr = allocator.allocate(64).unwrap();
        assert_eq!(addr, 0x1000);
        assert_eq!(allocator.allocated(), 64);
    }

    #[test]
    fn test_bump_allocator_allocate_multiple() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        let addr1 = allocator.allocate(32).unwrap();
        let addr2 = allocator.allocate(64).unwrap();
        let addr3 = allocator.allocate(128).unwrap();

        assert!(addr1 < addr2);
        assert!(addr2 < addr3);
        assert_eq!(addr1 % 8, 0);
        assert_eq!(addr2 % 8, 0);
        assert_eq!(addr3 % 8, 0);
    }

    #[test]
    fn test_bump_allocator_allocate_alignment() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 16).unwrap();
        let addr = allocator.allocate(1).unwrap();
        assert_eq!(addr % 16, 0);
    }

    #[test]
    fn test_bump_allocator_allocate_zero_size() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        let addr = allocator.allocate(0).unwrap();
        assert!(addr >= 0x1000 && addr < 0x2000);
    }

    #[test]
    fn test_bump_allocator_allocate_out_of_memory() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x1100, 8).unwrap();
        let _ = allocator.allocate(200).unwrap();
        let result = allocator.allocate(100);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_bump_allocator_allocate_exactly_full() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x1040, 8).unwrap();
        let _ = allocator.allocate(64).unwrap();
        assert!(allocator.is_full());
    }

    #[test]
    fn test_bump_allocator_allocate_too_large() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        let result = allocator.allocate(2 * 1024 * 1024 * 1024);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_bump_allocator_allocate_overflow() {
        let allocator = BumpPointerAllocator::new(0x1000, usize::MAX, 8).unwrap();
        let result = allocator.allocate(usize::MAX - 100);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    // ========================================================================
    // BumpPointerAllocator Reset and State Tests
    // ========================================================================

    #[test]
    fn test_bump_allocator_reset() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        let _ = allocator.allocate(100).unwrap();
        assert!(allocator.allocated() > 0);

        allocator.reset();
        assert_eq!(allocator.allocated(), 0);
        assert_eq!(allocator.remaining(), allocator.capacity());
    }

    #[test]
    fn test_bump_allocator_remaining() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        assert_eq!(allocator.remaining(), 0x1000);

        let _ = allocator.allocate(100).unwrap();
        assert!(allocator.remaining() < 0x1000);
    }

    #[test]
    fn test_bump_allocator_is_full() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x1040, 8).unwrap();
        assert!(!allocator.is_full());

        let _ = allocator.allocate(64).unwrap();
        assert!(allocator.is_full());
    }

    #[test]
    fn test_bump_allocator_set_top() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        allocator.set_top(0x1500);
        assert_eq!(allocator.allocated(), 0x500);
    }

    #[test]
    fn test_bump_allocator_set_top_out_of_bounds() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        allocator.set_top(0x0500); // Before start
        assert_eq!(allocator.allocated(), 0); // Should not change

        allocator.set_top(0x3000); // After end
        assert_eq!(allocator.allocated(), 0); // Should not change
    }

    // ========================================================================
    // BumpPointerAllocator Edge Cases
    // ========================================================================

    #[test]
    fn test_bump_allocator_various_sizes() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
        let sizes = [1, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 255, 256];

        for &size in &sizes {
            let addr = allocator.allocate(size).unwrap();
            assert!(addr >= 0x1000 && addr < 0x2000);
            assert_eq!(addr % 8, 0);
        }
    }

    #[test]
    fn test_bump_allocator_concurrent_allocation() {
        use std::sync::Arc;
        use std::thread;

        let allocator = Arc::new(BumpPointerAllocator::new(0x1000, 0x10000, 8).unwrap());
        let mut handles = Vec::new();

        for _ in 0..4 {
            let allocator = Arc::clone(&allocator);
            let handle = thread::spawn(move || {
                let mut addrs = Vec::new();
                for _ in 0..10 {
                    if let Ok(addr) = allocator.allocate(64) {
                        addrs.push(addr);
                    }
                }
                addrs
            });
            handles.push(handle);
        }

        let mut all_addrs = Vec::new();
        for handle in handles {
            all_addrs.extend(handle.join().unwrap());
        }

        // All addresses should be unique
        use std::collections::HashSet;
        let unique: HashSet<_> = all_addrs.iter().collect();
        assert_eq!(
            unique.len(),
            all_addrs.len(),
            "Concurrent allocations should be unique"
        );
    }

    // ========================================================================
    // MultiBumpAllocator Tests
    // ========================================================================

    #[test]
    fn test_multi_bump_allocator_new() {
        let alloc = MultiBumpAllocator::new(4096, 8, 10);
        assert_eq!(alloc.total_capacity().unwrap(), 0);
        assert_eq!(alloc.total_allocated().unwrap(), 0);
    }

    #[test]
    fn test_multi_bump_allocator_allocate() {
        let alloc = MultiBumpAllocator::new(4096, 8, 10);
        let addr = alloc.allocate(64).unwrap();
        assert!(addr > 0);
        assert_eq!(alloc.total_allocated().unwrap(), 64);
    }

    #[test]
    fn test_multi_bump_allocator_allocate_multiple_regions() {
        let alloc = MultiBumpAllocator::new(256, 8, 10);

        // Fill first region
        for _ in 0..4 {
            let _ = alloc.allocate(64).unwrap();
        }

        // This should create a new region
        let addr = alloc.allocate(64).unwrap();
        assert!(addr > 0);
        assert!(alloc.total_capacity().unwrap() > 256);
    }

    #[test]
    fn test_multi_bump_allocator_max_regions() {
        let alloc = MultiBumpAllocator::new(256, 8, 2);

        // Fill both regions
        for _ in 0..8 {
            let _ = alloc.allocate(64).unwrap();
        }

        // This should fail - max regions reached
        let result = alloc.allocate(64);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_multi_bump_allocator_reset_all() {
        let alloc = MultiBumpAllocator::new(4096, 8, 10);
        let _ = alloc.allocate(100).unwrap();
        assert!(alloc.total_allocated().unwrap() > 0);

        alloc.reset_all().unwrap();
        assert_eq!(alloc.total_allocated().unwrap(), 0);
    }

    #[test]
    fn test_multi_bump_allocator_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let alloc = Arc::new(MultiBumpAllocator::new(4096, 8, 100));
        let mut handles = Vec::new();

        for _ in 0..4 {
            let alloc = Arc::clone(&alloc);
            let handle = thread::spawn(move || {
                let mut count = 0;
                for _ in 0..10 {
                    if alloc.allocate(64).is_ok() {
                        count += 1;
                    }
                }
                count
            });
            handles.push(handle);
        }

        let total: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert!(total > 0);
    }
}
