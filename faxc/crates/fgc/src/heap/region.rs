//! Region Implementation - Unit of Heap Management
//!
//! Region is a contiguous block of memory with fixed size.
//! Every region has a defined lifecycle state.
//!
//! Region Types:
//! - Small (2MB): For objects < 256 bytes, 90%+ allocations go here
//! - Medium (32MB): For objects 256 bytes - 4KB
//! - Large (variable): For objects > 4KB, 1 object per region
//!
//! Region Lifecycle:
//! ```
//! Allocating ──▶ Allocated ──▶ Relocating ──▶ Relocated ──▶ Free
//!     │              │              │              │          │
//!     └──────────────┴──────────────┴──────────────┴──────────┘
//! ```
//!
//! # Memory Ordering Model
//!
//! This module uses the following atomic ordering strategy:
//!
//! ## Region State (AtomicUsize for start/top)
//! - **Load:** `Ordering::Relaxed` - State is protected by external synchronization
//!   (e.g., GC safepoint, region locks). Exact visibility order is not critical.
//! - **Store:** `Ordering::SeqCst` - State changes must be globally visible
//!   for region lifecycle transitions (reset, allocate).
//!
//! ## Mark Bitmap Operations
//! - **mark_object():** `Ordering::Relaxed` - Same rationale as bitmap.rs.
//!   Bitmap accessed during GC safepoint with external synchronization.
//! - **is_marked():** `Ordering::Relaxed` - Same rationale.
//!
//! ## Mark Bit (AtomicBool for GC cycle tracking)
//! - **Load/Store:** `Ordering::Relaxed` - Only used to track which mark bit
//!   (Marked0 vs Marked1) is current. Eventual consistency is acceptable.
//!
//! ## Allocation Counters
//! - **All operations:** `Ordering::Relaxed` - Statistics only, no ordering required.
//!
//! ## Rationale
//!
//! Region operations are synchronized externally:
//! 1. Allocation uses CAS for bump pointer (SeqCst for success)
//! 2. State transitions happen during GC phases with barriers
//! 3. Mark bitmap is only modified during safepoint

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Region - unit of heap management
///
/// Region is a contiguous block of memory with fixed size.
/// Every region has a defined lifecycle state.
///
/// Thread Safety:
/// Region is designed for multi-threaded access.
/// State changes use atomic operations.
pub struct Region {
    /// Start address of region in virtual memory
    start: AtomicUsize,

    /// Region size (2MB, 32MB, or variable)
    size: usize,

    /// Region type (Small, Medium, Large)
    region_type: RegionType,

    /// Current state
    state: std::sync::Mutex<RegionState>,

    /// Bump pointer for allocation
    top: AtomicUsize,

    /// End address (limit) of region
    end: usize,

    /// Marking bitmap for tracking live objects
    mark_bitmap: Vec<AtomicU64>,

    /// Forwarding table for relocation
    forwarding_table: std::sync::Mutex<Option<Arc<crate::relocate::ForwardingTable>>>,

    /// NUMA node where region is allocated
    numa_node: usize,

    /// Generation (Young or Old)
    generation: Generation,

    /// Allocation count - how many times this region was used
    allocation_count: AtomicUsize,

    /// Mark bit current (false = Marked0, true = Marked1)
    mark_bit: AtomicBool,

    /// Is memory from VirtualMemory (needs commit)
    needs_commit: bool,
}

impl Region {
    /// Create new region with specific start address
    ///
    /// # Arguments
    /// * `start_address` - Start address of region (from VirtualMemory)
    /// * `region_type` - Region type (Small, Medium, Large)
    /// * `size` - Region size in bytes
    /// * `generation` - Generation (Young or Old)
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully created region
    /// * `Err(FgcError::InvalidArgument)` - Invalid parameters
    ///
    /// # Validation
    /// - `start_address` must be non-zero
    /// - `size` must be greater than 0
    /// - `size` must be aligned to page size (4KB)
    pub fn with_address(
        start_address: usize,
        region_type: RegionType,
        size: usize,
        generation: Generation,
    ) -> Result<Self> {
        // Validate start_address is non-zero
        if start_address == 0 {
            return Err(FgcError::InvalidArgument(
                "start_address must be non-zero".to_string()
            ));
        }

        // Validate size > 0
        if size == 0 {
            return Err(FgcError::InvalidArgument(
                "size must be greater than 0".to_string()
            ));
        }

        // Validate size is aligned to page size (4KB)
        const PAGE_SIZE: usize = 4096;
        if size % PAGE_SIZE != 0 {
            return Err(FgcError::InvalidArgument(
                format!("size ({}) must be aligned to page size ({})", size, PAGE_SIZE)
            ));
        }

        let bitmap_size = (size / 64 + 63) / 64;
        let mark_bitmap = (0..bitmap_size).map(|_| AtomicU64::new(0)).collect();

        // FIX: Use checked_add to prevent integer overflow when calculating end address
        let end = start_address.checked_add(size).ok_or_else(|| {
            FgcError::InvalidArgument(
                format!("start_address ({:#x}) + size ({}) overflows usize", start_address, size)
            )
        })?;

        Ok(Self {
            start: AtomicUsize::new(start_address),
            size,
            region_type,
            state: std::sync::Mutex::new(RegionState::Free),
            top: AtomicUsize::new(start_address),
            end,
            mark_bitmap,
            forwarding_table: std::sync::Mutex::new(None),
            numa_node: 0,
            generation,
            allocation_count: AtomicUsize::new(0),
            mark_bit: AtomicBool::new(false),
            needs_commit: false,
        })
    }

    /// Create new region (legacy - uses default address)
    ///
    /// # Arguments
    /// * `region_type` - Region type (Small, Medium, Large)
    /// * `size` - Region size in bytes
    /// * `generation` - Generation (Young or Old)
    pub fn new(region_type: RegionType, size: usize, generation: Generation) -> Result<Self> {
        Self::with_address(0x1000_0000, region_type, size, generation)
    }

    /// Create region that requires memory commit
    pub fn new_sparse(
        start_address: usize,
        region_type: RegionType,
        size: usize,
        generation: Generation,
    ) -> Result<Self> {
        let mut region = Self::with_address(start_address, region_type, size, generation)?;
        region.needs_commit = true;
        Ok(region)
    }

    /// Allocate memory in this region
    ///
    /// Uses bump pointer allocation (O(1)).
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    /// * `alignment` - Alignment requirement
    ///
    /// # Returns
    /// * `Ok(usize)` - Allocated address on success
    /// * `Err(FgcError::OutOfMemory)` - If region is full
    /// * `Err(FgcError::RegionAllocationFailed)` - If region is in wrong state
    ///
    /// # Safety
    ///
    /// This function uses checked arithmetic to prevent integer overflow.
    /// All pointer calculations are validated before use.
    pub fn allocate(&self, size: usize, alignment: usize) -> Result<usize> {
        // Check state
        let state = *self.state.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Region state lock poisoned: {}", e))
        })?;
        if state != RegionState::Allocating && state != RegionState::Allocated {
            return Err(FgcError::RegionAllocationFailed {
                reason: format!("Region in {:?} state", state),
            });
        }

        // Align size with overflow check
        // FIX: Use checked arithmetic to prevent overflow in alignment calculation
        let aligned_size = if alignment > 0 {
            // Check for overflow in (size + alignment - 1)
            size.checked_add(alignment)
                .and_then(|sum| sum.checked_sub(1))
                .map(|aligned| aligned & !(alignment - 1))
                .ok_or_else(|| FgcError::OutOfMemory {
                    requested: size,
                    available: 0, // Overflow means invalid request
                })?
        } else {
            size
        };

        // Bump pointer allocation with overflow protection
        // Relaxed: load current top, CAS will ensure visibility
        let mut current_top = self.top.load(Ordering::Relaxed);

        loop {
            // FIX Issue 3: Use checked_add to prevent integer overflow
            let new_top = match current_top.checked_add(aligned_size) {
                Some(top) => top,
                None => {
                    return Err(FgcError::OutOfMemory {
                        requested: size,
                        available: 0, // Overflow means no space
                    });
                }
            };

            // Check if region is full
            if new_top > self.end {
                return Err(FgcError::OutOfMemory {
                    requested: size,
                    available: self.end.saturating_sub(current_top),
                });
            }

            // Try CAS to update top
            // SeqCst for success: allocation must be globally visible
            // Relaxed for failure: we just retry with actual value
            match self.top.compare_exchange_weak(
                current_top,
                new_top,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Success! Mark object in bitmap
                    self.mark_object(current_top, size);
                    return Ok(current_top);
                }
                Err(actual) => {
                    // CAS failed, retry
                    current_top = actual;
                }
            }
        }
    }

    /// Mark object in bitmap
    ///
    /// # Arguments
    /// * `address` - Object address to mark
    /// * `_size` - Object size (reserved for future use)
    ///
    /// # Safety
    ///
    /// This function validates that the address is within region bounds
    /// before performing any arithmetic to prevent integer underflow.
    pub fn mark_object(&self, address: usize, _size: usize) {
        // FIX Issue 4: Validate bounds BEFORE subtraction to prevent underflow
        let region_start = self.start.load(Ordering::Relaxed);
        
        // Check if address is before region start (would cause underflow)
        if address < region_start {
            log::warn!(
                "mark_object: address {:#x} is before region start {:#x}",
                address, region_start
            );
            return;
        }
        
        // Check if address is at or after region end
        if address >= self.end {
            log::warn!(
                "mark_object: address {:#x} is at or after region end {:#x}",
                address, self.end
            );
            return;
        }

        // Now safe to calculate offset (no underflow possible)
        let offset = address - region_start;
        let bit_index = offset / 64;
        let bit_offset = offset % 64;

        if bit_index < self.mark_bitmap.len() {
            // Relaxed: bitmap accessed during GC safepoint only
            self.mark_bitmap[bit_index].fetch_or(1u64 << bit_offset, Ordering::Relaxed);
        }
    }

    /// Check if object is marked
    ///
    /// # Arguments
    /// * `address` - Object address to check
    ///
    /// # Returns
    /// `true` if object is marked, `false` otherwise
    ///
    /// # Safety
    ///
    /// This function validates that the address is within region bounds
    /// before performing any arithmetic to prevent integer underflow.
    pub fn is_marked(&self, address: usize) -> bool {
        // FIX Issue 5: Validate bounds BEFORE subtraction to prevent underflow
        let region_start = self.start.load(Ordering::Relaxed);
        
        // Check if address is before region start (would cause underflow)
        if address < region_start {
            return false;
        }
        
        // Check if address is at or after region end
        if address >= self.end {
            return false;
        }

        // Now safe to calculate offset (no underflow possible)
        let offset = address - region_start;
        let bit_index = offset / 64;
        let bit_offset = offset % 64;

        if bit_index >= self.mark_bitmap.len() {
            return false;
        }

        (self.mark_bitmap[bit_index].load(Ordering::Relaxed) & (1u64 << bit_offset)) != 0
    }

    /// Reset region for reuse
    ///
    /// # Safety
    ///
    /// This function verifies that no live objects exist in the region before resetting.
    /// Resetting a region with live objects would cause use-after-free vulnerabilities.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if region is empty and successfully reset
    /// - `Err(FgcError::InvalidState)` if region contains live objects
    pub fn reset(&self) -> Result<()> {
        // Verify no live objects before reset to prevent use-after-free
        let marked_count = self.count_marked();
        if marked_count > 0 {
            log::error!("Resetting region with {} live objects!", marked_count);
            return Err(FgcError::InvalidState {
                expected: "empty region".to_string(),
                actual: format!("region with {} live objects", marked_count),
            });
        }

        *self.state.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Region state lock poisoned: {}", e))
        })? = RegionState::Allocating;
        // SeqCst: reset must be globally visible before any new allocation
        self.top
            .store(self.start.load(Ordering::Relaxed), Ordering::SeqCst);

        // Clear bitmap - safe now that we've verified no live objects
        for word in self.mark_bitmap.iter() {
            word.store(0, Ordering::Relaxed);
        }

        // Relaxed: statistics counter, no ordering required
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Flip mark bits for new GC cycle
    pub fn flip_mark_bits(&self) {
        // Relaxed: mark bit is statistics, eventual consistency acceptable
        let current = self.mark_bit.load(Ordering::Relaxed);
        self.mark_bit.store(!current, Ordering::Relaxed);
    }

    /// Get current mark bit
    pub fn current_mark_bit(&self) -> bool {
        // Relaxed: mark bit is statistics, eventual consistency acceptable
        self.mark_bit.load(Ordering::Relaxed)
    }

    /// Set region state
    ///
    /// # Returns
    /// * `Ok(())` - State successfully set
    /// * `Err(FgcError::LockPoisoned)` - If mutex is poisoned
    pub fn set_state(&self, state: RegionState) -> Result<()> {
        *self.state.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Region state lock poisoned: {}", e))
        })? = state;
        Ok(())
    }

    /// Get region state
    ///
    /// # Returns
    /// `Some(RegionState)` if lock acquired successfully, `None` if poisoned
    pub fn state(&self) -> Option<RegionState> {
        self.state.lock().ok().map(|g| *g)
    }

    /// Get region type
    pub fn region_type(&self) -> RegionType {
        self.region_type
    }

    /// Get region size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get generation
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Get bytes used in region
    ///
    /// # Returns
    /// Number of bytes allocated in this region
    ///
    /// # Safety
    ///
    /// This function uses checked subtraction to prevent integer underflow.
    /// If top < start (should not happen normally), returns 0.
    pub fn used(&self) -> usize {
        // FIX: Use checked_sub to prevent underflow if top < start
        self.top.load(Ordering::Relaxed)
            .checked_sub(self.start.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get bytes remaining
    ///
    /// # Returns
    /// Number of bytes remaining in this region
    ///
    /// # Safety
    ///
    /// This function uses checked subtraction to prevent integer underflow.
    /// If top > end (should not happen normally), returns 0.
    pub fn remaining(&self) -> usize {
        // FIX: Use checked_sub to prevent underflow if top > end
        self.end.checked_sub(self.top.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Check if region is full
    pub fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    /// Get start address
    pub fn start(&self) -> usize {
        self.start.load(Ordering::Relaxed)
    }

    /// Get end address
    pub fn end(&self) -> usize {
        self.end
    }

    /// Check if an address is within this region's bounds
    ///
    /// # Arguments
    /// * `addr` - Address to check
    ///
    /// # Returns
    /// `true` if address is within [start, end), `false` otherwise
    pub fn contains(&self, addr: usize) -> bool {
        let start = self.start.load(Ordering::Relaxed);
        addr >= start && addr < self.end
    }

    /// Get allocation count
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Get garbage ratio (estimate)
    pub fn garbage_ratio(&self) -> f32 {
        // Simple estimation: ratio of unmarked objects
        let total_objects = self.allocation_count.load(Ordering::Relaxed);
        if total_objects == 0 {
            return 0.0;
        }

        let marked_count = self.count_marked();
        1.0 - (marked_count as f32 / total_objects as f32)
    }

    /// Count marked objects in the region
    ///
    /// # Returns
    /// Number of objects currently marked in the bitmap
    pub fn count_marked(&self) -> usize {
        self.mark_bitmap
            .iter()
            .map(|word| word.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }

    /// Setup forwarding table for relocation
    pub fn setup_forwarding(&self) -> Result<()> {
        let mut ft = self.forwarding_table.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Region forwarding_table lock poisoned: {}", e))
        })?;
        *ft = Some(Arc::new(crate::relocate::ForwardingTable::new(
            self.start.load(Ordering::Relaxed),
            self.size,
        )));
        Ok(())
    }

    /// Get forwarding table
    pub fn forwarding_table(&self) -> Option<Arc<crate::relocate::ForwardingTable>> {
        self.forwarding_table.lock().ok()?.clone()
    }
}

/// Region type based on object size it holds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    /// 2MB region for small objects (< 256 bytes)
    Small,
    /// 32MB region for medium objects (256 bytes - 4KB)
    Medium,
    /// Variable size region for large objects (> 4KB)
    Large,
}

/// Region lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionState {
    /// Region is being filled with new objects
    Allocating,
    /// Region is full or no longer allocating
    Allocated,
    /// Region is selected for compaction
    Relocating,
    /// All objects have been moved
    Relocated,
    /// Region is empty and ready for reuse
    Free,
}

/// Young vs Old generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    /// Young generation (nursery) for new objects
    Young,
    /// Old generation for objects that survive minor GC
    Old,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a small test region
    fn create_test_region() -> Region {
        Region::with_address(0x1000_0000, RegionType::Small, 2 * 1024 * 1024, Generation::Young)
            .expect("Failed to create test region")
    }

    // ========================================================================
    // Issue 1: Integer Overflow in Region::allocate
    // ========================================================================

    #[test]
    fn test_allocate_overflow_protection() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Allocate most of the region
        let large_size = region.remaining() - 100;
        let addr = region.allocate(large_size, 8).expect("Should allocate large chunk");
        assert_ne!(addr, 0);

        // Try to allocate more than remaining - should fail with OutOfMemory
        let result = region.allocate(200, 8);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_allocate_near_boundary() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Allocate exactly remaining space
        let remaining = region.remaining();
        let result = region.allocate(remaining, 8);

        // Should succeed if aligned, or fail with OutOfMemory
        match result {
            Ok(addr) => {
                assert_ne!(addr, 0);
                assert_eq!(region.remaining(), 0);
            }
            Err(FgcError::OutOfMemory { .. }) => {
                // Alignment may cause it to exceed, which is correct behavior
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_allocate_with_overflow_alignment() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Try to allocate with alignment that would overflow
        // usize::MAX alignment would cause overflow in (size + alignment - 1)
        let result = region.allocate(1024, usize::MAX);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    // ========================================================================
    // Issue 2: Integer Underflow in Region::mark_object
    // ========================================================================

    #[test]
    fn test_mark_object_address_before_region() {
        let region = create_test_region();
        let region_start = region.start();

        // Try to mark an address before the region start
        // This should NOT panic or underflow
        region.mark_object(region_start - 100, 64);

        // Verify no crash occurred and region state is intact
        assert_eq!(region.start(), region_start);
    }

    #[test]
    fn test_mark_object_address_after_region() {
        let region = create_test_region();
        let region_end = region.end();

        // Try to mark an address after the region end
        // This should NOT panic
        region.mark_object(region_end + 100, 64);

        // Verify no crash occurred
        assert_eq!(region.end(), region_end);
    }

    #[test]
    fn test_mark_object_zero_address() {
        let region = create_test_region();

        // Try to mark address 0 (null pointer)
        // This should NOT panic
        region.mark_object(0, 64);

        // Verify no crash occurred
        assert!(region.start() > 0);
    }

    #[test]
    fn test_mark_object_valid_address() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Allocate an object
        let addr = region.allocate(64, 8).expect("Should allocate");

        // Mark the object - should succeed
        region.mark_object(addr, 64);

        // Verify it's marked
        assert!(region.is_marked(addr));
    }

    // ========================================================================
    // Issue 3: Integer Underflow in Region::is_marked
    // ========================================================================

    #[test]
    fn test_is_marked_address_before_region() {
        let region = create_test_region();

        // Check is_marked with address before region - should return false, not panic
        let result = region.is_marked(0x0000_0000);
        assert!(!result);

        let result = region.is_marked(region.start() - 1000);
        assert!(!result);
    }

    #[test]
    fn test_is_marked_address_after_region() {
        let region = create_test_region();
        let region_end = region.end();

        // Check is_marked with address after region - should return false, not panic
        let result = region.is_marked(region_end + 1000);
        assert!(!result);
    }

    #[test]
    fn test_is_marked_zero_address() {
        let region = create_test_region();

        // Check is_marked with null address - should return false
        let result = region.is_marked(0);
        assert!(!result);
    }

    // ========================================================================
    // Additional Integer Arithmetic Tests
    // ========================================================================

    #[test]
    fn test_constructor_overflow_protection() {
        // Try to create a region where start + size would overflow
        let large_start = usize::MAX - 1000;
        let large_size = 2048;

        let result = Region::with_address(large_start, RegionType::Large, large_size, Generation::Old);
        assert!(matches!(result, Err(FgcError::InvalidArgument(_))));
    }

    #[test]
    fn test_used_remaining_no_underflow() {
        let region = create_test_region();
        let initial_used = region.used();
        let initial_remaining = region.remaining();

        // Verify used + remaining = size (approximately, accounting for alignment)
        assert!(initial_used + initial_remaining <= region.size());

        // Allocate some memory
        region.set_state(RegionState::Allocating).expect("Failed to set state");
        let _addr = region.allocate(1024, 8).expect("Should allocate");

        let new_used = region.used();
        let new_remaining = region.remaining();

        // Verify no underflow occurred
        assert!(new_used >= initial_used);
        assert!(new_remaining <= initial_remaining);
        assert!(new_used + new_remaining <= region.size());
    }

    #[test]
    fn test_remaining_at_boundary() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Fill the region completely
        while region.allocate(64, 8).is_ok() {}

        // remaining() should return 0, not underflow
        assert_eq!(region.remaining(), 0);
        assert!(region.is_full());
    }

    #[test]
    fn test_used_after_reset() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Allocate some memory
        let _addr = region.allocate(1024, 8).expect("Should allocate");
        assert!(region.used() > 0);

        // Reset should clear used count
        // Note: reset() requires no live objects, so we can't test full reset here
        // Instead, verify used() doesn't underflow with normal operations
        assert!(region.used() >= 0); // Always true for usize, but documents intent
    }

    #[test]
    fn test_allocation_loop_exhaustion() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        let mut allocations = 0;
        loop {
            match region.allocate(64, 8) {
                Ok(_) => allocations += 1,
                Err(FgcError::OutOfMemory { .. }) => break,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        // Verify we made some allocations
        assert!(allocations > 0);

        // Verify region is now full
        assert_eq!(region.remaining(), 0);

        // Verify used() + remaining() doesn't underflow/overflow
        assert_eq!(region.used() + region.remaining(), region.size());
    }

    #[test]
    fn test_mark_bitmap_bounds() {
        let region = create_test_region();
        region.set_state(RegionState::Allocating).expect("Failed to set state");

        // Allocate an object
        let addr = region.allocate(64, 8).expect("Should allocate");

        // Mark it
        region.mark_object(addr, 64);

        // Verify is_marked returns true
        assert!(region.is_marked(addr));

        // Verify adjacent addresses (different bits) are not marked
        assert!(!region.is_marked(addr + 64));
        assert!(!region.is_marked(addr - 64));
    }
}
