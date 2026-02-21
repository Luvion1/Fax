//! Page Management - Fine-Grained Memory Pages
//!
//! A Page is a smaller unit than a region for fine-grained memory management.
//! Each region consists of multiple pages.
//!
//! Page size: 4KB (standard page) or 2MB (large page)
//!
//! Pages are used for:
//! - Fine-grained tracking memory usage
//! - Efficient memory commit/uncommit
//! - NUMA-aware allocation
//! - Page-level garbage collection

use crate::error::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Standard page size (4KB)
pub const PAGE_SIZE: usize = 4 * 1024;

/// Large page size (2MB)
pub const LARGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// System page size (cached)
static SYSTEM_PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);

/// Get system page size dynamically
///
/// Returns actual system page size from OS.
/// Caches result for performance.
pub fn get_page_size() -> usize {
    let cached = SYSTEM_PAGE_SIZE.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }

    let size = page_size::get();
    SYSTEM_PAGE_SIZE.store(size, Ordering::Relaxed);
    size
}

/// Align size to page boundary (round up)
pub fn align_to_page(size: usize) -> usize {
    let ps = get_page_size();
    (size + ps - 1) & !(ps - 1)
}

/// Align address to page boundary (round down)
pub fn align_down_to_page(addr: usize) -> usize {
    let ps = get_page_size();
    addr & !(ps - 1)
}

/// Align address to page boundary (round up)
pub fn align_up_to_page(addr: usize) -> usize {
    let ps = get_page_size();
    (addr + ps - 1) & !(ps - 1)
}

/// Convert bytes to pages (round up)
pub fn bytes_to_pages(bytes: usize) -> usize {
    let ps = get_page_size();
    bytes.div_ceil(ps)
}

/// Convert pages to bytes
pub fn pages_to_bytes(pages: usize) -> usize {
    pages * get_page_size()
}

/// Check if address is page-aligned
pub fn is_page_aligned(addr: usize) -> bool {
    addr.is_multiple_of(get_page_size())
}

/// Calculate offset within page
pub fn page_offset(addr: usize) -> usize {
    addr & (get_page_size() - 1)
}

/// Calculate page number from address
pub fn page_number(addr: usize) -> usize {
    addr / get_page_size()
}

/// Page - small unit of memory management
///
/// A Page is a contiguous block of memory with a fixed size.
/// Multiple pages form a region.
pub struct Page {
    /// Page address
    address: usize,

    /// Page size
    size: usize,

    /// Committed status
    committed: AtomicBool,

    /// Accessed status (for GC heuristics)
    accessed: AtomicBool,

    /// Modified status (for GC heuristics)
    modified: AtomicBool,

    /// NUMA node
    numa_node: usize,
}

impl Page {
    /// Create new page
    ///
    /// # Arguments
    /// * `address` - Page address
    /// * `size` - Page size
    /// * `numa_node` - NUMA node where page is allocated
    pub fn new(address: usize, size: usize, numa_node: usize) -> Self {
        Self {
            address,
            size,
            committed: AtomicBool::new(false),
            accessed: AtomicBool::new(false),
            modified: AtomicBool::new(false),
            numa_node,
        }
    }

    /// Commit page (allocate physical memory)
    pub fn commit(&self) -> Result<()> {
        // Note: In real implementation, this would use mmap or VirtualAlloc
        self.committed.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Uncommit page (return physical memory to OS)
    pub fn uncommit(&self) -> Result<()> {
        // Note: In real implementation, this would use munmap or VirtualFree
        self.committed.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Check if page is committed
    pub fn is_committed(&self) -> bool {
        self.committed.load(Ordering::Relaxed)
    }

    /// Mark page as accessed
    pub fn mark_accessed(&self) {
        self.accessed.store(true, Ordering::Relaxed);
    }

    /// Mark page as modified
    pub fn mark_modified(&self) {
        self.modified.store(true, Ordering::Relaxed);
    }

    /// Reset accessed/modified flags
    pub fn reset_flags(&self) {
        self.accessed.store(false, Ordering::Relaxed);
        self.modified.store(false, Ordering::Relaxed);
    }

    /// Get page address
    pub fn address(&self) -> usize {
        self.address
    }

    /// Get page size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get NUMA node
    pub fn numa_node(&self) -> usize {
        self.numa_node
    }
}

/// PageTable - manages pages in a region
///
/// Tracks status of each page in a region.
pub struct PageTable {
    /// Pages in table
    pages: Vec<Page>,

    /// Page size
    page_size: usize,

    /// Total pages
    page_count: usize,
}

impl PageTable {
    /// Create new page table
    ///
    /// # Arguments
    /// * `base_address` - Base address of region
    /// * `region_size` - Size of region in bytes
    /// * `page_size` - Size of each page
    /// * `numa_node` - NUMA node
    pub fn new(
        base_address: usize,
        region_size: usize,
        page_size: usize,
        numa_node: usize,
    ) -> Self {
        let page_count = region_size.div_ceil(page_size);
        let mut pages = Vec::with_capacity(page_count);

        for i in 0..page_count {
            let address = base_address + (i * page_size);
            pages.push(Page::new(address, page_size, numa_node));
        }

        Self {
            pages,
            page_size,
            page_count,
        }
    }

    /// Get page for address
    pub fn get_page(&self, address: usize) -> Option<&Page> {
        let offset = address % (self.page_count * self.page_size);
        let page_index = offset / self.page_size;

        self.pages.get(page_index)
    }

    /// Commit range pages
    pub fn commit_range(&self, start: usize, size: usize) -> Result<()> {
        let start_page = start / self.page_size;
        let end_page = (start + size).div_ceil(self.page_size);

        for i in start_page..end_page.min(self.page_count) {
            self.pages[i].commit()?;
        }

        Ok(())
    }

    /// Uncommit range pages
    pub fn uncommit_range(&self, start: usize, size: usize) -> Result<()> {
        let start_page = start / self.page_size;
        let end_page = (start + size).div_ceil(self.page_size);

        for i in start_page..end_page.min(self.page_count) {
            self.pages[i].uncommit()?;
        }

        Ok(())
    }

    /// Get committed page count
    pub fn committed_count(&self) -> usize {
        self.pages.iter().filter(|p| p.is_committed()).count()
    }

    /// Get total committed bytes
    pub fn committed_bytes(&self) -> usize {
        self.committed_count() * self.page_size
    }

    /// Get page count
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Get page size
    pub fn page_size(&self) -> usize {
        self.page_size
    }
}

/// Page allocator for tracking allocations
///
/// Simple tracker for page allocations.
/// Does not allocate actual memory, only tracks statistics.
pub struct PageAllocator {
    /// Total pages allocated
    allocated: AtomicUsize,

    /// Total pages freed
    freed: AtomicUsize,

    /// Peak pages allocated
    peak: AtomicUsize,

    /// Page size used
    page_size: usize,
}

impl PageAllocator {
    /// Create new page allocator
    pub fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            freed: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            page_size: get_page_size(),
        }
    }

    /// Record page allocation
    pub fn allocate(&self, pages: usize) {
        let current = self.allocated.fetch_add(pages, Ordering::Relaxed) + pages;

        let mut peak = self.peak.load(Ordering::Relaxed);
        while current > peak {
            match self.peak.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    /// Record page free
    pub fn free(&self, pages: usize) {
        self.freed.fetch_add(pages, Ordering::Relaxed);
    }

    /// Get current allocated pages
    pub fn current_pages(&self) -> usize {
        self.allocated
            .load(Ordering::Relaxed)
            .saturating_sub(self.freed.load(Ordering::Relaxed))
    }

    /// Get current allocated bytes
    pub fn current_bytes(&self) -> usize {
        self.current_pages() * self.page_size
    }

    /// Get total pages ever allocated
    pub fn total_allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get total pages ever freed
    pub fn total_freed(&self) -> usize {
        self.freed.load(Ordering::Relaxed)
    }

    /// Get peak pages allocated
    pub fn peak_pages(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }

    /// Get peak bytes allocated
    pub fn peak_bytes(&self) -> usize {
        self.peak_pages() * self.page_size
    }

    /// Get page size
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.freed.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
    }
}

impl Default for PageAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory range representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRange {
    /// Start page number
    pub start: usize,
    /// Number of pages
    pub count: usize,
}

impl PageRange {
    /// Create new page range
    pub fn new(start: usize, count: usize) -> Self {
        Self { start, count }
    }

    /// Create from byte addresses
    pub fn from_addresses(start_addr: usize, end_addr: usize) -> Self {
        let ps = get_page_size();
        let start = align_down_to_page(start_addr) / ps;
        let end = align_up_to_page(end_addr) / ps;
        Self {
            start,
            count: end.saturating_sub(start),
        }
    }

    /// Get start address
    pub fn start_address(&self) -> usize {
        self.start * get_page_size()
    }

    /// Get end address (exclusive)
    pub fn end_address(&self) -> usize {
        (self.start + self.count) * get_page_size()
    }

    /// Get size in bytes
    pub fn size(&self) -> usize {
        self.count * get_page_size()
    }

    /// Check if address is in range
    pub fn contains(&self, addr: usize) -> bool {
        let page = page_number(addr);
        page >= self.start && page < self.start + self.count
    }

    /// Check if page is in range
    pub fn contains_page(&self, page: usize) -> bool {
        page >= self.start && page < self.start + self.count
    }
}
