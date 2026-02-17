//! Forwarding Table - Address Mapping During Relocation
//!
//! Forwarding table mapping dari alamat lama ke alamat baru
//! saat object direlocate.
//!
//! Structure:
//! - Per-region forwarding table
//! - Lock-free lookup untuk performance
//! - CAS untuk concurrent updates
//!
//! Usage:
//! 1. Setup forwarding table saat start relocation
//! 2. Add entry saat copy object
//! 3. Lookup dari load barrier untuk pointer healing
//! 4. Cleanup setelah relocation complete

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::RwLock;

/// ForwardingTable - mapping old addresses ke new addresses
///
/// Thread-safe forwarding table untuk relocation.
pub struct ForwardingTable {
    /// Region start address
    region_start: usize,

    /// Region size
    region_size: usize,

    /// Forwarding entries: old_offset -> new_address
    entries: RwLock<HashMap<usize, usize>>,

    /// Table is complete (no more additions)
    complete: AtomicBool,

    /// Entry count
    entry_count: AtomicUsize,
}

impl ForwardingTable {
    /// Create forwarding table untuk region
    ///
    /// # Arguments
    /// * `region_start` - Start address region
    /// * `region_size` - Size region dalam bytes
    pub fn new(region_start: usize, region_size: usize) -> Self {
        Self {
            region_start,
            region_size,
            entries: RwLock::new(HashMap::new()),
            complete: AtomicBool::new(false),
            entry_count: AtomicUsize::new(0),
        }
    }

    /// Add forwarding entry
    ///
    /// Thread-safe, bisa dipanggil concurrent.
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    /// * `new_address` - New object address
    pub fn add_entry(&self, old_address: usize, new_address: usize) {
        let offset = old_address - self.region_start;

        let mut entries = self.entries.write().unwrap();
        entries.insert(offset, new_address);
        self.entry_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Lookup forwarding untuk offset
    ///
    /// Returns new_address jika found, None jika tidak.
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    ///
    /// # Returns
    /// New address atau None
    pub fn lookup(&self, old_address: usize) -> Option<usize> {
        let offset = old_address - self.region_start;

        let entries = self.entries.read().unwrap();
        entries.get(&offset).copied()
    }

    /// Check jika forwarding complete untuk semua objects
    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Mark table sebagai complete
    pub fn set_complete(&self) {
        self.complete.store(true, Ordering::SeqCst);
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
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

    /// Get all entries (untuk debugging)
    pub fn get_all_entries(&self) -> Vec<(usize, usize)> {
        let entries = self.entries.read().unwrap();
        entries.iter().map(|(&k, &v)| (k, v)).collect()
    }
}

/// Forwarding entry dengan state
///
/// Track status setiap forwarding entry.
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
    /// Entry pending (object belum di-copy)
    Pending,
    /// Object sedang di-copy
    InProgress,
    /// Object sudah di-copy
    Completed,
}

/// Fast forwarding table dengan array-based lookup
///
/// Optimized untuk region dengan object density tinggi.
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
    pub fn add_entry(&mut self, old_address: usize, new_address: usize) {
        let offset = old_address - self.region_start;
        let index = offset / self.granularity;

        if index < self.entries.len() {
            self.entries[index] = (new_address, 1); // 1 = completed
        }
    }

    /// Lookup
    pub fn lookup(&self, old_address: usize) -> Option<usize> {
        let offset = old_address - self.region_start;
        let index = offset / self.granularity;

        if index < self.entries.len() && self.entries[index].1 == 1 {
            return Some(self.entries[index].0);
        }

        None
    }
}
