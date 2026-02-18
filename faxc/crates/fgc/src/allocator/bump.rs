//! Allocator Submodule - Bump Pointer Allocation
//!
//! Bump pointer allocator is the fastest allocation technique.
//! Allocation only requires a single atomic increment operation.
//!
//! How it works:
//! 1. Region is allocated from heap
//! 2. Pointer is set to region start
//! 3. For allocate: increment pointer, return old address
//! 4. Region full: allocate new region
//!
//! Speed: O(1) - constant time regardless of size or fragmentation
//!
//! Limitations:
//! - Cannot free individual objects
//! - Region must be reset entirely
//! - Suitable for generational GC (young gen)

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

/// BumpPointerAllocator - fast bump pointer allocator
///
/// Allocator for a single region using bump pointer technique.
/// Thread-safe using atomic operations.
pub struct BumpPointerAllocator {
    /// Start address of region
    start: AtomicUsize,

    /// Current bump pointer (next free address)
    top: AtomicUsize,

    /// End address of region
    end: AtomicUsize,

    /// Alignment requirement (typically 8 bytes)
    alignment: usize,
}

impl BumpPointerAllocator {
    /// Create new bump allocator for region [start, end)
    ///
    /// # Arguments
    /// * `start` - Start address of region
    /// * `end` - End address of region (exclusive)
    /// * `alignment` - Alignment requirement for allocated objects
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully created allocator
    /// * `Err(FgcError::InvalidArgument)` - Invalid parameters
    ///
    /// # Validation
    /// - `start` must be less than `end`
    /// - `alignment` must be a power of two
    /// - `alignment` must not exceed 1024 bytes
    /// - Region size must be at least `alignment` bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fgc::allocator::bump::BumpPointerAllocator;
    ///
    /// // Valid allocator
    /// let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();
    ///
    /// // Invalid: start >= end
    /// assert!(BumpPointerAllocator::new(0x2000, 0x1000, 8).is_err());
    ///
    /// // Invalid: alignment not power of two
    /// assert!(BumpPointerAllocator::new(0, 1000, 3).is_err());
    /// ```
    pub fn new(start: usize, end: usize, alignment: usize) -> Result<Self> {
        // Validate start < end
        if start >= end {
            return Err(FgcError::InvalidArgument(
                format!("start ({:#x}) must be less than end ({:#x})", start, end)
            ));
        }

        // Validate alignment is power of two
        if !alignment.is_power_of_two() {
            return Err(FgcError::InvalidArgument(
                format!("alignment ({}) must be a power of two", alignment)
            ));
        }

        // Validate alignment is not too large
        if alignment > 1024 {
            return Err(FgcError::InvalidArgument(
                format!("alignment ({}) is too large (max 1024)", alignment)
            ));
        }

        // Validate region size is at least alignment
        let region_size = end - start;
        if region_size < alignment {
            return Err(FgcError::InvalidArgument(
                format!("region size ({:#x}) must be at least alignment ({:#x})",
                       region_size, alignment)
            ));
        }

        Ok(Self {
            start: AtomicUsize::new(start),
            top: AtomicUsize::new(start),
            end: AtomicUsize::new(end),
            alignment,
        })
    }

    /// Allocate memory with specific size
    ///
    /// Fast path: single atomic increment
    /// Slow path: alignment adjustment
    ///
    /// # Arguments
    /// * `size` - Size in bytes to allocate
    ///
    /// # Returns
    /// Address of allocated memory, or error if region is full
    ///
    /// # Safety
    ///
    /// Uses checked arithmetic to prevent integer overflow.
    /// The returned address is guaranteed to be:
    /// - Aligned to the allocator's alignment requirement
    /// - Within the region bounds [start, end)
    /// - Unique (no other thread will receive the same address)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fgc::allocator::bump::BumpPointerAllocator;
    ///
    /// let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8);
    /// let addr = allocator.allocate(64).unwrap();
    /// assert!(addr >= 0x1000 && addr < 0x2000);
    /// ```
    pub fn allocate(&self, size: usize) -> Result<usize> {
        let aligned_size = self.align_size(size)?;
        let mut current_top = self.top.load(Ordering::Relaxed);

        loop {
            let new_top = self.calculate_new_top(current_top, aligned_size, size)?;

            match self.try_update_top(current_top, new_top) {
                Ok(_) => return Ok(current_top),
                Err(_) => {
                    // CAS failed, retry with current value
                    current_top = self.top.load(Ordering::Relaxed);
                }
            }
        }
    }

    /// Calculate new top pointer with overflow and bounds checking
    ///
    /// # Arguments
    /// * `current_top` - Current bump pointer value
    /// * `aligned_size` - Size aligned to allocation boundary
    /// * `requested_size` - Original requested size for error reporting
    ///
    /// # Returns
    /// New top address if valid, or OutOfMemory error
    ///
    /// # Errors
    /// Returns `OutOfMemory` if:
    /// - Addition would overflow (checked_add fails)
    /// - New top exceeds region end
    fn calculate_new_top(&self, current_top: usize, aligned_size: usize, requested_size: usize) -> Result<usize> {
        let new_top = current_top.checked_add(aligned_size)
            .ok_or_else(|| FgcError::OutOfMemory {
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

    /// Try to update top pointer using CAS
    ///
    /// # Arguments
    /// * `current` - Expected current value
    /// * `new` - New value to set
    ///
    /// # Returns
    /// Ok(()) if CAS succeeded, Err(FgcError) if failed
    ///
    /// # Memory Ordering
    /// Uses `SeqCst` for success to ensure global visibility of allocation.
    /// Uses `Relaxed` for failure since we just retry with actual value.
    fn try_update_top(&self, current: usize, new: usize) -> Result<()> {
        self.top.compare_exchange_weak(
            current,
            new,
            Ordering::SeqCst,
            Ordering::Relaxed,
        ).map(|_| ()).map_err(|actual| {
            FgcError::LockPoisoned(format!("CAS failed, actual value: {:#x}", actual))
        })
    }

    /// Reset allocator to region start
    ///
    /// Called after GC reclaims all objects in region.
    /// Safe only when no thread is currently allocating.
    pub fn reset(&self) {
        self.top.store(self.start.load(Ordering::Relaxed), Ordering::SeqCst);
    }

    /// Get remaining space in region
    pub fn remaining(&self) -> usize {
        let current_top = self.top.load(Ordering::Relaxed);
        let end_val = self.end.load(Ordering::Relaxed);
        end_val - current_top
    }

    /// Get total region capacity
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

    /// Check if allocator is full
    pub fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    /// Align size to boundary
    ///
    /// # CRIT-03 FIX: Integer Overflow Prevention
    /// This method now validates size limits and uses checked arithmetic
    /// to prevent integer overflow attacks with malicious sizes.
    fn align_size(&self, size: usize) -> Result<usize> {
        // CRIT-03 FIX: Reject obviously malicious sizes
        const MAX_ALLOCATION: usize = 1024 * 1024 * 1024; // 1GB
        if size > MAX_ALLOCATION {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        // Safe alignment calculation with overflow check
        let mask = self.alignment.wrapping_sub(1);
        let aligned = size.wrapping_add(mask) & !mask;

        if aligned < size {
            // Overflow detected
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        Ok(aligned)
    }

    /// Set bump pointer to specific address
    ///
    /// For internal GC use only.
    /// Not thread-safe, must be called when no allocation is happening.
    pub fn set_top(&self, address: usize) {
        let start_val = self.start.load(Ordering::Relaxed);
        let end_val = self.end.load(Ordering::Relaxed);
        if address >= start_val && address <= end_val {
            self.top.store(address, Ordering::SeqCst);
        }
    }
}

/// MultiBumpAllocator - multiple bump regions for concurrency
///
/// Manages multiple bump pointer regions to reduce contention.
/// Each thread can get its own region for lock-free allocation.
pub struct MultiBumpAllocator {
    /// List of available regions
    regions: std::sync::Mutex<Vec<BumpPointerAllocator>>,

    /// Region size for each bump allocator
    region_size: usize,

    /// Alignment requirement
    alignment: usize,

    /// Maximum regions allowed
    max_regions: usize,
}

impl MultiBumpAllocator {
    /// Create new multi-region bump allocator
    ///
    /// # Arguments
    /// * `region_size` - Size of each region in bytes
    /// * `alignment` - Alignment requirement
    /// * `max_regions` - Maximum number of regions
    pub fn new(region_size: usize, alignment: usize, max_regions: usize) -> Self {
        Self {
            regions: std::sync::Mutex::new(Vec::new()),
            region_size,
            alignment,
            max_regions,
        }
    }

    /// Allocate from one of the regions
    ///
    /// Strategy:
    /// 1. Try allocate from existing region (lock-free)
    /// 2. If all full, create new region
    /// 3. If max regions reached, return error
    pub fn allocate(&self, size: usize) -> Result<usize> {
        // Try existing regions first
        {
            let regions = self.regions.lock().unwrap();

            // Try every region (can be optimized with per-thread region)
            for region in regions.iter() {
                if let Ok(addr) = region.allocate(size) {
                    return Ok(addr);
                }
            }
        }

        // All regions full, create new region
        self.allocate_new_region(size)
    }

    /// Allocate new region and try to allocate
    fn allocate_new_region(&self, size: usize) -> Result<usize> {
        let mut regions = self.regions.lock().unwrap();

        // Check max regions
        if regions.len() >= self.max_regions {
            return Err(FgcError::OutOfMemory {
                requested: size,
                available: 0,
            });
        }

        // Create new region
        // Note: In real implementation, address should come from heap manager
        let base_address = 0x1000 * (regions.len() as usize + 1); // Dummy address
        let region = BumpPointerAllocator::new(
            base_address,
            base_address + self.region_size,
            self.alignment,
        )?;

        // Allocate from new region
        let addr = region.allocate(size)?;

        regions.push(region);

        Ok(addr)
    }

    /// Reset all regions
    pub fn reset_all(&self) {
        let regions = self.regions.lock().unwrap();
        for region in regions.iter() {
            region.reset();
        }
    }

    /// Get total allocated bytes from all regions
    pub fn total_allocated(&self) -> usize {
        let regions = self.regions.lock().unwrap();
        regions.iter().map(|r| r.allocated()).sum()
    }

    /// Get total capacity from all regions
    pub fn total_capacity(&self) -> usize {
        let regions = self.regions.lock().unwrap();
        regions.len() * self.region_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_allocator_helper_functions() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();

        // Test calculate_new_top with valid values
        let result = allocator.calculate_new_top(0x1000, 64, 64);
        assert_eq!(result.unwrap(), 0x1040);

        // Test calculate_new_top with overflow
        let result = allocator.calculate_new_top(usize::MAX - 10, 100, 100);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));

        // Test calculate_new_top exceeding region end
        let result = allocator.calculate_new_top(0x1FF0, 64, 64);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_try_update_top_success() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();

        // Test successful CAS
        let result = allocator.try_update_top(0x1000, 0x1040);
        assert!(result.is_ok());

        // Verify top was updated
        assert_eq!(allocator.allocated(), 64);
    }

    #[test]
    fn test_try_update_top_failure() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();

        // First update succeeds
        let result = allocator.try_update_top(0x1000, 0x1040);
        assert!(result.is_ok());

        // Second update with old current value should fail
        let result = allocator.try_update_top(0x1000, 0x1080);
        assert!(result.is_err());
        // Error should be LockPoisoned with actual value in message
    }

    #[test]
    fn test_allocate_uses_helpers() {
        let allocator = BumpPointerAllocator::new(0x1000, 0x2000, 8).unwrap();

        // Multiple allocations should work
        let addr1 = allocator.allocate(32).unwrap();
        let addr2 = allocator.allocate(64).unwrap();
        let addr3 = allocator.allocate(128).unwrap();

        // Verify addresses are aligned and sequential
        assert!(addr1 >= 0x1000 && addr1 < 0x2000);
        assert!(addr2 > addr1);
        assert!(addr3 > addr2);

        // Verify alignment (8 bytes)
        assert_eq!(addr1 % 8, 0);
        assert_eq!(addr2 % 8, 0);
        assert_eq!(addr3 % 8, 0);
    }
}
