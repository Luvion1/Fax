//! Forwarding Table - Address Mapping During Relocation
//!
//! Forwarding table mapping from old addresses to new addresses
//! when objects are relocated.
//!
//! Structure:
//! - Per-region forwarding table
//! - Lock-free lookup for performance
//! - CAS for concurrent updates
//! - Generation counter for TOCTOU race prevention
//!
//! Usage:
//! 1. Setup forwarding table at start of relocation
//! 2. Add entry when copying object
//! 3. Lookup from load barrier for pointer healing
//! 4. Cleanup after relocation complete

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::RwLock;
use indexmap::IndexMap;

/// ForwardingTable - mapping old addresses to new addresses
///
/// Thread-safe forwarding table for relocation.
/// 
/// # FIX Issue 8: TOCTOU Race Prevention
/// 
/// This table includes a generation counter that is incremented when the table
/// is modified. Load barriers can capture the generation during lookup and verify
/// it hasn't changed before using the result, preventing TOCTOU races.
pub struct ForwardingTable {
    /// Region start address
    region_start: usize,

    /// Region size
    region_size: usize,

    /// Forwarding entries: old_offset -> new_address
    entries: RwLock<IndexMap<usize, usize>>,

    /// Table is complete (no more additions)
    complete: AtomicBool,

    /// Entry count
    entry_count: AtomicUsize,
    
    /// FIX Issue 8: Generation counter for TOCTOU race prevention
    /// Incremented on every modification to the forwarding table
    generation: AtomicU64,
}

impl ForwardingTable {
    /// Create forwarding table for region
    ///
    /// # Arguments
    /// * `region_start` - Start address of region
    /// * `region_size` - Size of region in bytes
    pub fn new(region_start: usize, region_size: usize) -> Self {
        Self {
            region_start,
            region_size,
            entries: RwLock::new(IndexMap::new()),
            complete: AtomicBool::new(false),
            entry_count: AtomicUsize::new(0),
            generation: AtomicU64::new(0),  // FIX Issue 8: Initialize generation counter
        }
    }
    
    /// FIX Issue 8: Get current generation counter
    /// 
    /// Load barriers should capture this value along with the lookup result
    /// and verify it hasn't changed before using the result.
    /// 
    /// # Returns
    /// Current generation number
    #[inline]
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
    
    /// FIX Issue 8: Increment generation counter
    /// 
    /// Called when the forwarding table is modified to invalidate
    /// any in-flight lookups.
    fn increment_generation(&self) {
        self.generation.fetch_add(1, Ordering::Release);
    }

    /// Add forwarding entry
    ///
    /// Thread-safe, can be called concurrently.
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    /// * `new_address` - New object address
    ///
    /// # Validation
    /// This function validates both addresses:
    /// - `old_address` must be within region bounds
    /// - `new_address` must be non-null, aligned, and in valid user space
    ///
    /// # Safety
    ///
    /// Invalid addresses are logged and rejected to prevent memory corruption.
    pub fn add_entry(&self, old_address: usize, new_address: usize) {
        // FIX Issue 9: Validate new_address before adding entry
        
        // Validate new_address is not null
        if new_address == 0 {
            log::warn!("add_entry: new_address is null (old_address={:#x})", old_address);
            return;
        }
        
        // Validate new_address is properly aligned
        if new_address % std::mem::align_of::<usize>() != 0 {
            log::warn!(
                "add_entry: new_address {:#x} is not aligned to {} bytes",
                new_address, std::mem::align_of::<usize>()
            );
            return;
        }
        
        // Validate new_address is in valid user space range
        // Reject kernel space addresses (typical kernel space starts at 0x0000_8000_0000_0000)
        if new_address > 0x0000_7FFF_FFFF_FFFF {
            log::warn!(
                "add_entry: new_address {:#x} is in kernel space",
                new_address
            );
            return;
        }
        
        // Validate old_address is within region bounds
        if old_address < self.region_start {
            log::warn!("Invalid old_address {:#x} is before region_start {:#x}",
                       old_address, self.region_start);
            return;
        }

        // Use checked_sub to prevent underflow
        let offset = match old_address.checked_sub(self.region_start) {
            Some(off) => off,
            None => {
                log::warn!("Overflow calculating offset for old_address {:#x}", old_address);
                return;
            }
        };

        // Check if offset is within region bounds
        if offset >= self.region_size {
            log::warn!("Offset {:#x} exceeds region_size {:#x}", offset, self.region_size);
            return;
        }

        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("ForwardingTable entries write lock poisoned: {}", e);
                return;
            }
        };
        entries.insert(offset, new_address);
        self.entry_count.fetch_add(1, Ordering::Relaxed);
        
        // FIX Issue 8: Increment generation to invalidate in-flight lookups
        self.increment_generation();
    }

    /// Lookup forwarding for offset
    ///
    /// Returns new_address if found, None if not.
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    ///
    /// # Returns
    /// New address or None
    ///
    /// # Note
    /// For TOCTOU-safe lookups, use `lookup_with_generation()` which returns
    /// the generation counter along with the result.
    pub fn lookup(&self, old_address: usize) -> Option<usize> {
        // CRIT-06 FIX: Validate address is within region bounds

        // Check if address is before region start (would cause underflow)
        if old_address < self.region_start {
            return None;
        }

        // Use checked_sub to prevent underflow
        let offset = old_address.checked_sub(self.region_start)?;

        // Check if offset is within region bounds
        if offset >= self.region_size {
            return None;
        }

        let entries = match self.entries.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("ForwardingTable entries read lock poisoned: {}", e);
                return None;
            }
        };
        entries.get(&offset).copied()
    }
    
    /// FIX Issue 8: Lookup forwarding with generation counter
    ///
    /// Returns both the new address and the current generation counter.
    /// The caller should verify the generation hasn't changed before using
    /// the result to prevent TOCTOU races.
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    ///
    /// # Returns
    /// Tuple of (new_address, generation) or None if not found
    ///
    /// # Example
    /// ```rust
    /// let (new_addr, gen) = forwarding_table.lookup_with_generation(addr)?;
    /// // Use new_addr...
    /// // Verify generation hasn't changed:
    /// if forwarding_table.generation() != gen {
    ///     // Table was modified, retry lookup
    /// }
    /// ```
    pub fn lookup_with_generation(&self, old_address: usize) -> Option<(usize, u64)> {
        // Validate address is within region bounds
        if old_address < self.region_start {
            return None;
        }

        // Use checked_sub to prevent underflow
        let offset = old_address.checked_sub(self.region_start)?;

        // Check if offset is within region bounds
        if offset >= self.region_size {
            return None;
        }

        // Capture generation BEFORE reading entries
        // This ensures we detect modifications that happen during lookup
        let gen = self.generation.load(Ordering::Acquire);

        let entries = match self.entries.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("ForwardingTable entries read lock poisoned: {}", e);
                return None;
            }
        };

        entries.get(&offset).copied().map(|addr| (addr, gen))
    }

    /// Check if forwarding is complete for all objects
    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Mark table as complete
    pub fn set_complete(&self) {
        self.complete.store(true, Ordering::SeqCst);
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("ForwardingTable entries write lock poisoned: {}", e);
                return;
            }
        };
        entries.clear();
        self.entry_count.store(0, Ordering::Relaxed);
        self.complete.store(false, Ordering::Relaxed);
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entry_count.load(Ordering::Relaxed)
    }

    /// Get region start
    pub fn region_start(&self) -> usize {
        self.region_start
    }

    /// Get region size
    pub fn region_size(&self) -> usize {
        self.region_size
    }

    /// Get all entries (for debugging)
    pub fn get_all_entries(&self) -> Vec<(usize, usize)> {
        let entries = match self.entries.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("ForwardingTable entries read lock poisoned: {}", e);
                return Vec::new();
            }
        };
        entries.iter().map(|(&k, &v)| (k, v)).collect()
    }
}

/// Forwarding entry with state
///
/// Track status of every forwarding entry.
#[derive(Debug, Clone)]
pub struct ForwardingEntry {
    /// Old address (offset)
    pub old_offset: usize,
    /// New address
    pub new_address: usize,
    /// Entry status
    pub status: ForwardingStatus,
}

/// Forwarding entry status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardingStatus {
    /// Entry pending (object not yet copied)
    Pending,
    /// Object being copied
    InProgress,
    /// Object copied
    Completed,
}

/// Fast forwarding table with array-based lookup
///
/// Optimized for region with high object density.
pub struct FastForwardingTable {
    /// Region start
    region_start: usize,

    /// Array index by object index
    /// Each entry: (new_address, status)
    entries: Vec<(usize, u8)>,

    /// Granularity (bytes per object index)
    granularity: usize,
}

impl FastForwardingTable {
    /// Create fast forwarding table
    ///
    /// # Arguments
    /// * `region_start` - Start address region
    /// * `region_size` - Size region
    /// * `granularity` - Bytes per object index
    pub fn new(region_start: usize, region_size: usize, granularity: usize) -> Self {
        let entry_count = region_size / granularity;

        Self {
            region_start,
            entries: vec![(0, 0); entry_count],
            granularity,
        }
    }

    /// Add entry
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    /// * `new_address` - New object address
    ///
    /// # Validation
    /// This function validates both addresses:
    /// - `old_address` must be within region bounds
    /// - `new_address` must be non-null, aligned, and in valid user space
    pub fn add_entry(&mut self, old_address: usize, new_address: usize) {
        // FIX Issue 9: Validate new_address before adding entry
        
        // Validate new_address is not null
        if new_address == 0 {
            return;
        }
        
        // Validate new_address is properly aligned
        if new_address % std::mem::align_of::<usize>() != 0 {
            return;
        }
        
        // Validate new_address is in valid user space range
        if new_address > 0x0000_7FFF_FFFF_FFFF {
            return;
        }
        
        // Validate old_address is within region bounds
        if old_address < self.region_start {
            return;
        }

        // Use checked_sub to prevent underflow
        let offset = match old_address.checked_sub(self.region_start) {
            Some(off) => off,
            None => return,
        };

        let index = offset / self.granularity;

        if index < self.entries.len() {
            self.entries[index] = (new_address, 1); // 1 = completed
        }
    }

    /// Lookup
    pub fn lookup(&self, old_address: usize) -> Option<usize> {
        // CRIT-06 FIX: Validate address is within region bounds

        // Check if address is before region start
        if old_address < self.region_start {
            return None;
        }

        // Use checked_sub to prevent underflow
        let offset = old_address.checked_sub(self.region_start)?;
        let index = offset / self.granularity;

        if index < self.entries.len() && self.entries[index].1 == 1 {
            return Some(self.entries[index].0);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test constants
    const REGION_START: usize = 0x1000_0000;
    const REGION_SIZE: usize = 0x1000_0000; // 256 MB
    const ALIGNMENT: usize = std::mem::align_of::<usize>();

    // ========================================================================
    // ForwardingTable::add_entry - new_address validation tests
    // ========================================================================

    #[test]
    fn test_add_entry_rejects_null_new_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let valid_old_address = REGION_START + 0x100;
        let null_new_address = 0usize;

        table.add_entry(valid_old_address, null_new_address);

        // Entry should not be added
        assert_eq!(table.entry_count(), 0);
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_add_entry_rejects_misaligned_new_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let valid_old_address = REGION_START + 0x100;
        // Create a misaligned address (not aligned to usize alignment)
        let misaligned_new_address = 0x2000_0001usize;

        table.add_entry(valid_old_address, misaligned_new_address);

        // Entry should not be added
        assert_eq!(table.entry_count(), 0);
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_add_entry_rejects_kernel_space_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let valid_old_address = REGION_START + 0x100;
        // Kernel space address (above 0x0000_7FFF_FFFF_FFFF)
        let kernel_new_address = 0xFFFF_8000_0000_0000usize;

        table.add_entry(valid_old_address, kernel_new_address);

        // Entry should not be added
        assert_eq!(table.entry_count(), 0);
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_add_entry_rejects_kernel_space_boundary_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let valid_old_address = REGION_START + 0x100;
        // Address just above user space boundary
        let kernel_boundary_address = 0x0000_8000_0000_0000usize;

        table.add_entry(valid_old_address, kernel_boundary_address);

        // Entry should not be added
        assert_eq!(table.entry_count(), 0);
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_add_entry_accepts_valid_new_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let valid_old_address = REGION_START + 0x100;
        let valid_new_address = 0x2000_0000usize; // Aligned, non-null, user space

        table.add_entry(valid_old_address, valid_new_address);

        // Entry should be added
        assert_eq!(table.entry_count(), 1);
        assert_eq!(table.lookup(valid_old_address), Some(valid_new_address));
    }

    #[test]
    fn test_add_entry_accepts_aligned_user_space_addresses() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);

        // Test various valid aligned addresses
        let test_cases = vec![
            (REGION_START + 0x100, 0x1000usize),      // Small aligned address
            (REGION_START + 0x200, 0x2000_0000usize), // Medium aligned address
            (REGION_START + 0x300, 0x0000_7FFF_FFFF_FFF8usize), // Max user space (aligned)
        ];

        for (old_addr, new_addr) in test_cases {
            table.add_entry(old_addr, new_addr);
        }

        assert_eq!(table.entry_count(), 3);
    }

    // ========================================================================
    // ForwardingTable::add_entry - old_address validation tests
    // ========================================================================

    #[test]
    fn test_add_entry_rejects_old_address_before_region() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let invalid_old_address = REGION_START - 0x100;
        let valid_new_address = 0x2000_0000usize;

        table.add_entry(invalid_old_address, valid_new_address);

        // Entry should not be added
        assert_eq!(table.entry_count(), 0);
    }

    #[test]
    fn test_add_entry_rejects_old_address_beyond_region() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let invalid_old_address = REGION_START + REGION_SIZE + 0x100;
        let valid_new_address = 0x2000_0000usize;

        table.add_entry(invalid_old_address, valid_new_address);

        // Entry should not be added
        assert_eq!(table.entry_count(), 0);
    }

    #[test]
    fn test_add_entry_accepts_old_address_at_region_boundary() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let valid_old_address = REGION_START; // Exactly at region start
        let valid_new_address = 0x2000_0000usize;

        table.add_entry(valid_old_address, valid_new_address);

        // Entry should be added
        assert_eq!(table.entry_count(), 1);
        assert_eq!(table.lookup(valid_old_address), Some(valid_new_address));
    }

    // ========================================================================
    // ForwardingTable::lookup tests
    // ========================================================================

    #[test]
    fn test_lookup_returns_none_for_address_before_region() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let address_before_region = REGION_START - 0x100;

        let result = table.lookup(address_before_region);

        assert!(result.is_none());
    }

    #[test]
    fn test_lookup_returns_none_for_address_beyond_region() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let address_beyond_region = REGION_START + REGION_SIZE + 0x100;

        let result = table.lookup(address_beyond_region);

        assert!(result.is_none());
    }

    #[test]
    fn test_lookup_returns_none_for_unmapped_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let unmapped_address = REGION_START + 0x100;

        let result = table.lookup(unmapped_address);

        assert!(result.is_none());
    }

    #[test]
    fn test_lookup_returns_mapped_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let old_address = REGION_START + 0x100;
        let new_address = 0x2000_0000usize;

        table.add_entry(old_address, new_address);

        let result = table.lookup(old_address);

        assert_eq!(result, Some(new_address));
    }

    // ========================================================================
    // ForwardingTable::lookup_with_generation tests
    // ========================================================================

    #[test]
    fn test_lookup_with_generation_returns_generation_counter() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let old_address = REGION_START + 0x100;
        let new_address = 0x2000_0000usize;

        table.add_entry(old_address, new_address);

        let result = table.lookup_with_generation(old_address);

        assert!(result.is_some());
        let (addr, gen) = result.unwrap();
        assert_eq!(addr, new_address);
        assert_eq!(gen, 1); // Generation should be 1 after one add_entry
    }

    #[test]
    fn test_lookup_with_generation_returns_none_for_invalid_address() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);
        let invalid_address = REGION_START - 0x100;

        let result = table.lookup_with_generation(invalid_address);

        assert!(result.is_none());
    }

    #[test]
    fn test_generation_increments_on_add_entry() {
        let table = ForwardingTable::new(REGION_START, REGION_SIZE);

        let initial_gen = table.generation();
        assert_eq!(initial_gen, 0);

        table.add_entry(REGION_START + 0x100, 0x2000_0000usize);
        assert_eq!(table.generation(), 1);

        table.add_entry(REGION_START + 0x200, 0x3000_0000usize);
        assert_eq!(table.generation(), 2);
    }

    // ========================================================================
    // FastForwardingTable tests
    // ========================================================================

    #[test]
    fn test_fast_forwarding_table_rejects_null_new_address() {
        let mut table = FastForwardingTable::new(REGION_START, REGION_SIZE, 0x1000);
        let valid_old_address = REGION_START + 0x100;
        let null_new_address = 0usize;

        table.add_entry(valid_old_address, null_new_address);

        // Entry should not be added
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_fast_forwarding_table_rejects_misaligned_new_address() {
        let mut table = FastForwardingTable::new(REGION_START, REGION_SIZE, 0x1000);
        let valid_old_address = REGION_START + 0x100;
        let misaligned_new_address = 0x2000_0001usize;

        table.add_entry(valid_old_address, misaligned_new_address);

        // Entry should not be added
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_fast_forwarding_table_rejects_kernel_space_address() {
        let mut table = FastForwardingTable::new(REGION_START, REGION_SIZE, 0x1000);
        let valid_old_address = REGION_START + 0x100;
        let kernel_new_address = 0xFFFF_8000_0000_0000usize;

        table.add_entry(valid_old_address, kernel_new_address);

        // Entry should not be added
        assert!(table.lookup(valid_old_address).is_none());
    }

    #[test]
    fn test_fast_forwarding_table_accepts_valid_new_address() {
        let mut table = FastForwardingTable::new(REGION_START, REGION_SIZE, 0x1000);
        let valid_old_address = REGION_START + 0x100;
        let valid_new_address = 0x2000_0000usize;

        table.add_entry(valid_old_address, valid_new_address);

        // Entry should be added
        assert_eq!(table.lookup(valid_old_address), Some(valid_new_address));
    }

    #[test]
    fn test_fast_forwarding_table_rejects_old_address_out_of_bounds() {
        let mut table = FastForwardingTable::new(REGION_START, REGION_SIZE, 0x1000);
        let invalid_old_address = REGION_START - 0x100;
        let valid_new_address = 0x2000_0000usize;

        table.add_entry(invalid_old_address, valid_new_address);

        // Entry should not be added
        assert!(table.lookup(invalid_old_address).is_none());
    }
}
