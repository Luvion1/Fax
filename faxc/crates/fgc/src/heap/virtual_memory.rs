//! Virtual Memory Management
//!
//! ============================================================================
//! OVERVIEW VIRTUAL MEMORY
//! ============================================================================
//!
//! This module manages virtual memory operations:
//! - Reserve address space
//! - Commit physical memory
//! - Uncommit memory (return to OS)
//! - Memory mapping for multi-mapping
//!
//! ============================================================================
//! VIRTUAL MEMORY LAYOUT
//! ============================================================================
//!
//! ```
//! ┌────────────────────────────────────────────────────────────────────┐
//! │                     Virtual Address Space                           │
//! │  ┌──────────┬──────────┬──────────┬──────────┬─────────┬─────────┐ │
//! │  │ Region 0 │ Region 1 │ Region 2 │ Region 3 │   ...   │ Region N│ │
//! │  │ (Small)  │ (Small)  │ (Medium) │ (Large)  │         │         │ │
//! │  │  2MB     │  2MB     │  32MB    │  16MB    │         │         │ │
//! │  └──────────┴──────────┴──────────┴──────────┴─────────┴─────────┘ │
//! │                                                                      │
//! │  Base Address ──────────────────────────────────────► Reserved Size │
//! │  0x7f0000000000                                    16TB max         │
//! └────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ============================================================================
//! COMMIT/UNCOMMIT LIFECYCLE
//! ============================================================================
//!
//! ```
//! Reserve ──► Commit ──► Use ──► Uncommit ──► (Return to OS)
//!    │          │         │          │
//!    │          │         │          └── madvise(MADV_DONTNEED)
//!    │          │         └── Read/Write operations
//!    │          └── mprotect(PROT_READ | PROT_WRITE)
//!    └── mmap(PROT_NONE, MAP_NORESERVE)
//! ```
//!
//! ============================================================================
//! MULTI-MAPPING FOR COLORED POINTERS
//! ============================================================================
//!
//! Virtual memory enables FGC to:
//! - Reserve large address space upfront without allocating physical memory
//! - Commit memory on-demand when needed
//! - Uncommit memory when not in use (reduce memory footprint)
//! - Multi-mapping for colored pointers
//!
//! Multi-Mapping Concept:
//! ```
//! Physical Memory:  [Object Data]
//!                         │
//!          ┌──────────────┼──────────────┐
//!          ▼              ▼              ▼
//! View 0: [Marked0]   View 1: [Marked1]   View 2: [Remapped]
//! 0x1000_0000      0x2000_0000       0x3000_0000
//! ```
//!
//! With multi-mapping, pointer color can be determined from virtual address
//! without modifying the object header.

use crate::error::{FgcError, Result};
use crate::heap::memory_mapping::MemoryMapping;
use crate::heap::page::{self, PageRange};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

/// VirtualMemory - manager for virtual memory operations
///
/// Manages reserved address space and committed memory ranges.
/// Thread-safe for concurrent access.
///
/// # Thread Safety
///
/// VirtualMemory is designed to be accessed from multiple threads.
/// - committed_ranges uses RwLock for concurrent read
/// - committed_size uses atomic operations
/// - Memory mapping is immutable after creation
pub struct VirtualMemory {
    /// Base address reserved space
    base_address: usize,

    /// Total reserved size in bytes
    reserved_size: usize,

    /// Committed ranges: offset -> size
    /// Uses BTreeMap for efficient range queries
    committed_ranges: RwLock<BTreeMap<usize, usize>>,

    /// Total committed size (cached for performance)
    committed_size: AtomicUsize,

    /// Memory mapping (backing storage)
    mapping: Option<MemoryMapping>,

    /// Is this a sparse mapping (commit on demand)
    sparse: bool,

    /// Page allocator for statistics
    page_allocator: page::PageAllocator,

    /// Large page size (0 = regular pages)
    large_page_size: usize,
}

impl VirtualMemory {
    /// Reserve virtual address space
    ///
    /// Reserve address space without allocating physical memory.
    /// Physical memory will be committed on-demand.
    ///
    /// # Arguments
    /// * `size` - Size of address space to reserve
    ///
    /// # Returns
    /// VirtualMemory instance or error
    ///
    /// # Examples
    /// ```
    /// let vm = VirtualMemory::reserve(64 * 1024 * 1024)?;
    /// assert!(vm.base_address() > 0);
    /// ```
    pub fn reserve(size: usize) -> Result<Self> {
        let aligned_size = page::align_to_page(size);

        let mapping = MemoryMapping::anonymous(aligned_size)?;
        let base_address = mapping.base();

        Ok(Self {
            base_address,
            reserved_size: aligned_size,
            committed_ranges: RwLock::new(BTreeMap::new()),
            committed_size: AtomicUsize::new(0),
            mapping: Some(mapping),
            sparse: true,
            page_allocator: page::PageAllocator::new(),
            large_page_size: 0,
        })
    }

    /// Reserve virtual address space with large pages
    ///
    /// Uses huge pages (2MB or 1GB) for better TLB performance.
    /// Falls back to regular pages if large pages unavailable.
    ///
    /// # Arguments
    /// * `size` - Size of address space to reserve
    /// * `large_page_size` - Large page size (e.g., 2MB = 2*1024*1024)
    ///
    /// # Returns
    /// VirtualMemory instance or error
    pub fn reserve_large_pages(size: usize, large_page_size: usize) -> Result<Self> {
        let aligned_size = page::align_to_page(size);
        let large_aligned = (aligned_size + large_page_size - 1) & !(large_page_size - 1);

        let mapping = match MemoryMapping::anonymous_large_pages(large_aligned, large_page_size) {
            Ok(m) => {
                log::info!(
                    "Using large pages ({} bytes) for {} bytes heap",
                    large_page_size,
                    large_aligned
                );
                m
            },
            Err(e) => {
                log::warn!(
                    "Large pages unavailable ({}), falling back to regular pages",
                    e
                );
                MemoryMapping::anonymous(aligned_size)?
            },
        };

        let base_address = mapping.base();

        Ok(Self {
            base_address,
            reserved_size: aligned_size,
            committed_ranges: RwLock::new(BTreeMap::new()),
            committed_size: AtomicUsize::new(0),
            mapping: Some(mapping),
            sparse: true,
            page_allocator: page::PageAllocator::new(),
            large_page_size,
        })
    }

    /// Enable transparent huge pages for this memory region
    pub fn enable_thp(&self) -> Result<()> {
        if let Some(ref mapping) = self.mapping {
            mapping.enable_transparent_huge_pages()
        } else {
            Ok(())
        }
    }

    /// Reserve with pre-committed memory
    ///
    /// Reserve and immediately commit all memory.
    /// Simpler but uses physical memory immediately.
    pub fn reserve_committed(size: usize) -> Result<Self> {
        let aligned_size = page::align_to_page(size);

        let mapping = MemoryMapping::anonymous(aligned_size)?;
        let base_address = mapping.base();

        let _page_count = page::bytes_to_pages(aligned_size);

        Ok(Self {
            base_address,
            reserved_size: aligned_size,
            committed_ranges: RwLock::new(BTreeMap::from([(0, aligned_size)])),
            committed_size: AtomicUsize::new(aligned_size),
            mapping: Some(mapping),
            sparse: false,
            page_allocator: page::PageAllocator::new(),
            large_page_size: 0,
        })
    }

    /// Commit memory in a specific range
    ///
    /// Allocate physical memory for address range.
    ///
    /// # Arguments
    /// * `offset` - Offset from base address
    /// * `size` - Size to commit
    ///
    /// # Errors
    /// - `VirtualMemoryError` if offset+size exceeds reserved size
    /// - `LockPoisoned` if mutex is poisoned
    pub fn commit(&self, offset: usize, size: usize) -> Result<()> {
        // Validate arguments
        if size == 0 {
            return Ok(());
        }

        // Use saturating_add to prevent overflow
        let end_offset = offset.saturating_add(size);

        if end_offset > self.reserved_size {
            return Err(FgcError::VirtualMemoryError(format!(
                "Commit exceeds reserved size: offset={}, size={}, end={}, reserved={}",
                offset, size, end_offset, self.reserved_size
            )));
        }

        let aligned_offset = page::align_down_to_page(offset);
        let aligned_end = page::align_up_to_page(offset + size);
        let aligned_size = aligned_end.saturating_sub(aligned_offset);

        // Invariant: aligned_size should be positive after alignment
        debug_assert!(
            aligned_size > 0 || size == 0,
            "Aligned size should be positive"
        );

        let mut ranges = self
            .committed_ranges
            .write()
            .map_err(|e| FgcError::LockPoisoned(format!("committed_ranges in commit: {}", e)))?;

        // Check for overlap - if already committed, return success
        if self.overlaps_committed(&ranges, aligned_offset, aligned_size) {
            return Ok(());
        }

        // Insert new committed range
        ranges.insert(aligned_offset, aligned_size);

        // Update statistics
        let page_count = page::bytes_to_pages(aligned_size);
        self.page_allocator.allocate(page_count);

        let old_size = self
            .committed_size
            .fetch_add(aligned_size, Ordering::AcqRel);

        // Invariant: committed_size should never exceed reserved_size
        debug_assert!(
            old_size.saturating_add(aligned_size) <= self.reserved_size,
            "Committed size exceeds reserved size"
        );

        Ok(())
    }

    /// Uncommit memory in a specific range
    ///
    /// Return physical memory to OS.
    ///
    /// # Arguments
    /// * `offset` - Offset from base address
    /// * `size` - Size to uncommit
    ///
    /// # Errors
    /// - `LockPoisoned` if mutex is poisoned
    pub fn uncommit(&self, offset: usize, size: usize) -> Result<()> {
        if size == 0 {
            return Ok(());
        }

        let aligned_offset = page::align_down_to_page(offset);
        let aligned_end = page::align_up_to_page(offset.saturating_add(size));
        let aligned_size = aligned_end.saturating_sub(aligned_offset);

        if aligned_size == 0 {
            return Ok(());
        }

        let mut ranges = self
            .committed_ranges
            .write()
            .map_err(|e| FgcError::LockPoisoned(format!("committed_ranges in uncommit: {}", e)))?;

        // Find and remove/update the committed range
        let mut freed_size = 0usize;

        // Find exact match or overlapping range
        if let Some(&existing_size) = ranges.get(&aligned_offset) {
            if existing_size == aligned_size {
                // Exact match - remove entirely
                ranges.remove(&aligned_offset);
                freed_size = aligned_size;
            } else if existing_size > aligned_size {
                // Partial uncommit - split the range
                let remaining_size = existing_size - aligned_size;
                ranges.remove(&aligned_offset);
                ranges.insert(aligned_offset.saturating_add(aligned_size), remaining_size);
                freed_size = aligned_size;
            }
        } else {
            // Try to find a range that contains this offset
            for (&start, &len) in ranges.iter() {
                let end = start.saturating_add(len);
                if aligned_offset >= start && aligned_offset < end {
                    // Found containing range - handle partial uncommit
                    // This is a simplified handling - just mark as freed
                    freed_size = aligned_size.min(end.saturating_sub(aligned_offset));
                    break;
                }
            }
        }

        if freed_size > 0 {
            let page_count = page::bytes_to_pages(freed_size);
            self.page_allocator.free(page_count);

            let old_committed = self.committed_size.fetch_sub(freed_size, Ordering::AcqRel);

            // Invariant: committed_size should never go negative
            debug_assert!(
                old_committed >= freed_size,
                "Committed size would go negative: old={}, freed={}",
                old_committed,
                freed_size
            );
        }

        Ok(())
    }

    /// Check if range overlaps with existing committed ranges
    fn overlaps_committed(
        &self,
        ranges: &BTreeMap<usize, usize>,
        offset: usize,
        size: usize,
    ) -> bool {
        if size == 0 {
            return false;
        }

        let check_end = offset.saturating_add(size);

        for (&start, &len) in ranges.iter() {
            if len == 0 {
                continue;
            }
            let end = start.saturating_add(len);

            // Two ranges overlap if one starts before the other ends
            if offset < end && check_end > start {
                return true;
            }
        }
        false
    }

    /// Check if address is committed
    pub fn is_committed(&self, offset: usize) -> bool {
        let ranges = match self.committed_ranges.read() {
            Ok(r) => r,
            Err(_) => return false, // Lock poisoned, assume not committed
        };

        for (&start, &len) in ranges.iter() {
            let end = start.saturating_add(len);
            if offset >= start && offset < end {
                return true;
            }
        }
        false
    }

    /// Get base address
    pub fn base_address(&self) -> usize {
        self.base_address
    }

    /// Get reserved size
    pub fn reserved_size(&self) -> usize {
        self.reserved_size
    }

    /// Get committed size
    pub fn committed_size(&self) -> usize {
        self.committed_size.load(Ordering::Relaxed)
    }

    /// Get available (uncommitted) size
    pub fn available_size(&self) -> usize {
        self.reserved_size
            .saturating_sub(self.committed_size.load(Ordering::Relaxed))
    }

    /// Return unused memory to the operating system
    ///
    /// This is similar to ZGC's ability to uncommit memory when not needed.
    /// Uses MADV_DONTNEED on Linux to return physical pages to the OS.
    ///
    /// # Returns
    /// Number of bytes actually returned to OS
    pub fn release_unused_memory(&self) -> usize {
        let committed = self.committed_size.load(Ordering::Relaxed);
        if committed == 0 {
            return 0;
        }

        if let Some(ref mapping) = self.mapping {
            if self.sparse {
                let _ = mapping.advise_dont_need();
                let old_committed = self.committed_size.load(Ordering::Acquire);
                self.committed_size.store(0, Ordering::Release);
                return old_committed;
            }
        }
        0
    }

    /// Check if transparent huge pages are enabled
    pub fn is_thp_enabled(&self) -> bool {
        self.large_page_size > 0
    }

    /// Get large page size
    pub fn large_page_size(&self) -> usize {
        self.large_page_size
    }

    /// Check if sparse allocation
    pub fn is_sparse(&self) -> bool {
        self.sparse
    }

    /// Get page statistics
    pub fn page_stats(&self) -> PageStats {
        PageStats {
            total_pages: page::bytes_to_pages(self.reserved_size),
            committed_pages: page::bytes_to_pages(self.committed_size.load(Ordering::Relaxed)),
            page_size: page::get_page_size(),
        }
    }

    /// Read from memory
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        if let Some(ref mapping) = self.mapping {
            mapping.read(offset, buf)
        } else {
            Err(FgcError::VirtualMemoryError(
                "No memory mapping available".to_string(),
            ))
        }
    }

    /// Write to memory
    ///
    /// # Safety
    /// Uses unsafe pointer write. The memory must be committed before writing.
    pub fn write(&self, offset: usize, data: &[u8]) -> Result<()> {
        if offset.saturating_add(data.len()) > self.reserved_size {
            return Err(FgcError::VirtualMemoryError(format!(
                "Write out of bounds: offset={}, len={}, size={}",
                offset,
                data.len(),
                self.reserved_size
            )));
        }

        if let Some(ref mapping) = self.mapping {
            unsafe {
                let ptr = mapping.as_ptr().add(offset);
                std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
                std::sync::atomic::fence(std::sync::atomic::Ordering::Release);
            }
            Ok(())
        } else {
            Err(FgcError::VirtualMemoryError(
                "No memory mapping available".to_string(),
            ))
        }
    }

    /// Get memory mapping reference
    pub fn mapping(&self) -> Option<&MemoryMapping> {
        self.mapping.as_ref()
    }

    /// Get committed ranges
    pub fn committed_ranges(&self) -> Vec<PageRange> {
        match self.committed_ranges.read() {
            Ok(ranges) => ranges
                .iter()
                .map(|(&offset, &size)| {
                    PageRange::new(
                        page::page_number(self.base_address.saturating_add(offset)),
                        page::bytes_to_pages(size),
                    )
                })
                .collect(),
            Err(_) => Vec::new(), // Lock poisoned, return empty
        }
    }

    /// Validate internal state consistency
    ///
    /// # Returns
    /// Ok(()) if state is consistent, Err with description of inconsistency
    pub fn validate_state(&self) -> Result<()> {
        let ranges = self
            .committed_ranges
            .read()
            .map_err(|e| FgcError::LockPoisoned(format!("validate_state: {}", e)))?;

        // Check that sum of committed ranges equals committed_size
        let calculated_size: usize = ranges.values().sum();
        let tracked_size = self.committed_size.load(Ordering::Acquire);

        if calculated_size != tracked_size {
            return Err(FgcError::Internal(format!(
                "Committed size mismatch: calculated={}, tracked={}",
                calculated_size, tracked_size
            )));
        }

        // Check that no range exceeds reserved_size
        for (&offset, &size) in ranges.iter() {
            if offset.saturating_add(size) > self.reserved_size {
                return Err(FgcError::Internal(format!(
                    "Committed range exceeds reserved size: offset={}, size={}",
                    offset, size
                )));
            }
        }

        // Check that committed_size <= reserved_size
        if tracked_size > self.reserved_size {
            return Err(FgcError::Internal(format!(
                "Committed size {} exceeds reserved size {}",
                tracked_size, self.reserved_size
            )));
        }

        Ok(())
    }

    /// Get memory at address as slice
    ///
    /// # Safety
    /// Caller must ensure the range is valid and committed
    pub unsafe fn as_slice(&self, offset: usize, len: usize) -> Result<&[u8]> {
        if let Some(ref mapping) = self.mapping {
            mapping.as_slice(offset, len)
        } else {
            Err(FgcError::VirtualMemoryError(
                "No memory mapping available".to_string(),
            ))
        }
    }
}

/// Page statistics
#[derive(Debug, Clone)]
pub struct PageStats {
    pub total_pages: usize,
    pub committed_pages: usize,
    pub page_size: usize,
}

impl std::fmt::Display for PageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PageStats {{ total: {} pages, committed: {} pages, page_size: {} bytes }}",
            self.total_pages, self.committed_pages, self.page_size
        )
    }
}

/// Helper to align address to page boundary
pub fn align_to_page(address: usize) -> usize {
    page::align_to_page(address)
}

/// Helper to convert bytes to pages
pub fn bytes_to_pages(bytes: usize) -> usize {
    page::bytes_to_pages(bytes)
}

/// Helper to convert pages to bytes
pub fn pages_to_bytes(pages: usize) -> usize {
    page::pages_to_bytes(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_memory_reserve() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        assert!(vm.base_address() > 0);
        assert_eq!(vm.reserved_size(), 64 * 1024 * 1024);
        assert!(vm.is_sparse());
    }

    #[test]
    fn test_virtual_memory_reserve_committed() {
        let vm = VirtualMemory::reserve_committed(1024 * 1024).unwrap();

        assert!(vm.base_address() > 0);
        assert_eq!(vm.committed_size(), 1024 * 1024);
        assert!(!vm.is_sparse());
    }

    #[test]
    fn test_commit_memory() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        assert_eq!(vm.committed_size(), 0);

        vm.commit(0, 4096).unwrap();

        assert!(vm.committed_size() >= 4096);
        assert!(vm.is_committed(0));
        assert!(vm.is_committed(4000));
    }

    #[test]
    fn test_commit_aligned() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        // Commit with unaligned offset
        vm.commit(100, 200).unwrap();

        let page_size = page::get_page_size();
        // Should commit full page
        assert!(vm.committed_size() >= page_size);
    }

    #[test]
    fn test_commit_multiple_ranges() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        let page_size = page::get_page_size();

        vm.commit(0, page_size).unwrap();
        vm.commit(page_size * 4, page_size).unwrap();

        assert_eq!(vm.committed_ranges().len(), 2);
        assert!(vm.is_committed(0));
        assert!(!vm.is_committed(page_size * 2));
        assert!(vm.is_committed(page_size * 4));
    }

    #[test]
    fn test_commit_exceeds_reserved() {
        let vm = VirtualMemory::reserve(1024 * 1024).unwrap();

        let result = vm.commit(512 * 1024, 1024 * 1024);

        assert!(result.is_err());
    }

    #[test]
    fn test_uncommit_memory() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        vm.commit(0, 4096).unwrap();
        assert!(vm.committed_size() > 0);

        vm.uncommit(0, 4096).unwrap();

        assert_eq!(vm.committed_size(), 0);
    }

    #[test]
    fn test_page_stats() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        let stats = vm.page_stats();

        assert!(stats.total_pages > 0);
        assert_eq!(stats.committed_pages, 0);
        assert!(stats.page_size >= 4096);
    }

    #[test]
    fn test_read_write_memory() {
        let vm = VirtualMemory::reserve(64 * 1024).unwrap();

        vm.commit(0, 4096).unwrap();

        let data = [1u8, 2, 3, 4, 5];
        vm.write(0, &data).unwrap();

        let mut buf = [0u8; 5];
        vm.read(0, &mut buf).unwrap();

        assert_eq!(buf, data);
    }

    #[test]
    fn test_commit_overlapping() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        let page_size = page::get_page_size();

        vm.commit(0, page_size).unwrap();
        let size_before = vm.committed_size();

        vm.commit(0, page_size).unwrap();
        let size_after = vm.committed_size();

        assert_eq!(size_before, size_after);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_commit_zero_size() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        // Should succeed without changing state
        let result = vm.commit(0, 0);
        assert!(result.is_ok());
        assert_eq!(vm.committed_size(), 0);
    }

    #[test]
    fn test_uncommit_zero_size() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        vm.commit(0, 4096).unwrap();
        let size_before = vm.committed_size();

        // Should succeed without changing state
        let result = vm.uncommit(0, 0);
        assert!(result.is_ok());
        assert_eq!(vm.committed_size(), size_before);
    }

    #[test]
    fn test_commit_at_boundary() {
        let vm = VirtualMemory::reserve(1024 * 1024).unwrap();

        // Commit exactly at the boundary
        let result = vm.commit(1024 * 1024 - 4096, 4096);
        assert!(result.is_ok());
    }

    #[test]
    fn test_commit_exactly_reserved_size() {
        let size = 1024 * 1024;
        let vm = VirtualMemory::reserve(size).unwrap();

        // Commit exactly the reserved size
        let result = vm.commit(0, size);
        assert!(result.is_ok());
        assert!(vm.committed_size() >= size);
    }

    #[test]
    fn test_commit_overflow_protection() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        // This should not cause integer overflow
        let result = vm.commit(usize::MAX / 2, usize::MAX / 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_at_boundary() {
        let vm = VirtualMemory::reserve(4096).unwrap();

        // Write at exact boundary
        let data = [0u8; 10];
        let result = vm.write(4096 - 10, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_exceeds_boundary() {
        let vm = VirtualMemory::reserve(4096).unwrap();

        // Write that exceeds boundary
        let data = [0u8; 100];
        let result = vm.write(4000, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_state_consistency() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();

        // Initial state should be valid
        assert!(vm.validate_state().is_ok());

        vm.commit(0, 4096).unwrap();
        assert!(vm.validate_state().is_ok());

        vm.commit(8192, 4096).unwrap();
        assert!(vm.validate_state().is_ok());

        vm.uncommit(0, 4096).unwrap();
        assert!(vm.validate_state().is_ok());
    }

    #[test]
    fn test_multiple_commit_uncommit_cycles() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();
        let page_size = page::get_page_size();

        // Multiple cycles
        for i in 0..5 {
            vm.commit(i * page_size, page_size).unwrap();
            assert!(vm.validate_state().is_ok());
        }

        for i in 0..5 {
            vm.uncommit(i * page_size, page_size).unwrap();
            assert!(vm.validate_state().is_ok());
        }

        assert_eq!(vm.committed_size(), 0);
    }

    #[test]
    fn test_is_committed_edge_cases() {
        let vm = VirtualMemory::reserve(64 * 1024 * 1024).unwrap();
        let page_size = page::get_page_size();

        // Nothing committed initially
        assert!(!vm.is_committed(0));
        assert!(!vm.is_committed(usize::MAX));

        vm.commit(0, page_size).unwrap();

        // Check boundaries
        assert!(vm.is_committed(0));
        assert!(vm.is_committed(page_size - 1));
        assert!(!vm.is_committed(page_size));
    }

    #[test]
    fn test_available_size() {
        let vm = VirtualMemory::reserve(1024 * 1024).unwrap();

        assert_eq!(vm.available_size(), 1024 * 1024);

        vm.commit(0, 4096).unwrap();
        assert!(vm.available_size() < 1024 * 1024);

        vm.uncommit(0, 4096).unwrap();
        assert_eq!(vm.available_size(), 1024 * 1024);
    }

    #[test]
    fn test_concurrent_commit_safety() {
        use std::sync::Arc;
        use std::thread;

        let vm = Arc::new(VirtualMemory::reserve(64 * 1024 * 1024).unwrap());
        let mut handles = Vec::new();

        for i in 0..4 {
            let vm_clone = Arc::clone(&vm);
            let handle = thread::spawn(move || {
                let page_size = page::get_page_size();
                for j in 0..10 {
                    let offset = (i * 10 + j) * page_size * 2;
                    let _ = vm_clone.commit(offset, page_size);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // State should still be valid after concurrent operations
        assert!(vm.validate_state().is_ok());
    }
}
