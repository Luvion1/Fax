//! Relocate Module - Object Relocation & Compaction
//!
//! This module implements object relocation for concurrent compaction.
//! Relocation moves objects from one location to another for:
//! - Memory compaction (eliminate fragmentation)
//! - Improve locality
//! - Enable fast bump-pointer allocation
//!
//! Concurrent Relocation Strategy:
//! 1. Setup forwarding tables
//! 2. Copy objects concurrently
//! 3. Load barriers handle pointer healing on-demand
//!
//! Self-Healing Pointers:
//! When a thread reads a pointer to an object that has been moved,
//! the load barrier will:
//! 1. Lookup forwarding table
//! 2. Get new address
//! 3. Update pointer atomically (CAS)
//! 4. Return object from new location

pub mod compaction;
pub mod copy;
pub mod forwarding;

pub use compaction::Compactor;
pub use copy::ObjectCopier;
pub use forwarding::ForwardingTable;

use crate::error::{FgcError, Result};
use crate::heap::{Heap, Region};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Relocator - manager for object relocation
///
/// Relocator manages:
/// - Relocation set selection
/// - Forwarding table management
/// - Concurrent object copying
/// - Pointer healing coordination
pub struct Relocator {
    /// Reference to heap
    heap: Arc<Heap>,

    /// Relocation set: regions selected for relocation
    relocation_set: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Forwarding tables per region
    forwarding_tables: std::sync::Mutex<std::collections::HashMap<usize, Arc<ForwardingTable>>>,

    /// Destination regions for relocated objects
    destination_regions: std::sync::Mutex<Vec<Arc<Region>>>,

    /// Object copier
    copier: ObjectCopier,

    /// Progress tracker
    relocated_count: AtomicU64,
    total_count: AtomicU64,
    bytes_relocated: AtomicUsize,

    /// Relocation in progress
    in_progress: AtomicBool,
}

impl Relocator {
    /// Create new relocator
    ///
    /// # Arguments
    /// * `heap` - Heap reference
    pub fn new(heap: Arc<Heap>) -> Self {
        Self {
            heap,
            relocation_set: std::sync::Mutex::new(Vec::new()),
            forwarding_tables: std::sync::Mutex::new(std::collections::HashMap::new()),
            destination_regions: std::sync::Mutex::new(Vec::new()),
            copier: ObjectCopier::new(),
            relocated_count: AtomicU64::new(0),
            total_count: AtomicU64::new(0),
            bytes_relocated: AtomicUsize::new(0),
            in_progress: AtomicBool::new(false),
        }
    }

    /// Prepare relocation phase
    ///
    /// Called after marking is complete.
    /// Setup relocation set and forwarding tables.
    pub fn prepare_relocation(&self) -> Result<()> {
        let regions = self.heap.get_active_regions();
        let mut relocation_set = Vec::new();

        for region in regions {
            if region.garbage_ratio() > 0.5 {
                relocation_set.push(region.clone());
                let _ = region.setup_forwarding();
            }
        }

        *self.relocation_set.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Relocator relocation_set lock poisoned: {}", e))
        })? = relocation_set;
        self.in_progress.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// Start concurrent relocation
    ///
    /// Spawn GC threads to copy objects.
    pub fn start_relocation(&self) -> Result<()> {
        let relocation_set = self.relocation_set.lock()
            .map_err(|e| FgcError::LockPoisoned(format!("Relocator relocation_set lock poisoned: {}", e)))?
            .clone();

        let mut dest_regions = Vec::new();
        for region in &relocation_set {
            let dest = self
                .heap
                .allocate_region(region.size(), region.generation())?;
            dest_regions.push(dest);
        }

        *self.destination_regions.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Relocator destination_regions lock poisoned: {}", e))
        })? = dest_regions;

        Ok(())
    }

    /// Relocate single object with actual memory copy
    ///
    /// Copy object to new location and update forwarding table.
    ///
    /// # Arguments
    /// * `old_address` - Source object address
    /// * `size` - Object size
    ///
    /// # Returns
    /// New address after relocation
    pub fn relocate_object(&self, old_address: usize, size: usize) -> Result<usize> {
        let relocation_set = self.relocation_set.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Relocator relocation_set lock poisoned: {}", e))
        })?;
        let source_region = relocation_set
            .iter()
            .find(|r| old_address >= r.start() && old_address < r.end());

        if source_region.is_none() {
            return Ok(old_address);
        }

        let dest_regions = self.destination_regions.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Relocator destination_regions lock poisoned: {}", e))
        })?;

        let new_address = if let Some(dest_region) = dest_regions.first() {
            match dest_region.allocate(size, 8) {
                Ok(addr) => addr,
                Err(_) => return Ok(old_address),
            }
        } else {
            let offset = self.bytes_relocated.load(Ordering::Relaxed);
            self.heap.base_address() + offset + size
        };

        if new_address != old_address && size > 0 {
            self.copier.copy_object(old_address, new_address, size)?;

            self.bytes_relocated.fetch_add(size, Ordering::Relaxed);
        }

        if let Some(region) = source_region {
            if let Some(ft) = region.forwarding_table() {
                ft.add_entry(old_address, new_address);
            }
        }

        self.relocated_count.fetch_add(1, Ordering::Relaxed);

        Ok(new_address)
    }

    /// Relocate object with pre-allocated destination
    ///
    /// For use when destination is already allocated.
    pub fn relocate_to(&self, old_address: usize, new_address: usize, size: usize) -> Result<()> {
        if size == 0 || old_address == new_address {
            return Ok(());
        }

        self.copier.copy_object(old_address, new_address, size)?;

        let relocation_set = self.relocation_set.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Relocator relocation_set lock poisoned: {}", e))
        })?;
        for region in relocation_set.iter() {
            if old_address >= region.start() && old_address < region.end() {
                if let Some(ft) = region.forwarding_table() {
                    ft.add_entry(old_address, new_address);
                }
                break;
            }
        }

        self.relocated_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_relocated.fetch_add(size, Ordering::Relaxed);

        Ok(())
    }

    /// Batch relocate multiple objects
    ///
    /// Efficient for relocating many objects at once.
    pub fn relocate_batch(&self, objects: &[(usize, usize)]) -> Result<Vec<usize>> {
        let mut new_addresses = Vec::with_capacity(objects.len());

        for &(old_address, size) in objects {
            let new_addr = self.relocate_object(old_address, size)?;
            new_addresses.push(new_addr);
        }

        Ok(new_addresses)
    }

    /// Lookup forwarding for address
    pub fn lookup_forwarding(&self, old_address: usize) -> Option<usize> {
        let relocation_set = match self.relocation_set.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Relocator relocation_set lock poisoned: {}", e);
                return None;
            }
        };

        for region in relocation_set.iter() {
            if let Some(ft) = region.forwarding_table() {
                if let Some(new_addr) = ft.lookup(old_address) {
                    return Some(new_addr);
                }
            }
        }

        None
    }

    /// Check if address is in relocation set
    pub fn in_relocation_set(&self, address: usize) -> bool {
        let relocation_set = match self.relocation_set.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Relocator relocation_set lock poisoned: {}", e);
                return false;
            }
        };

        for region in relocation_set.iter() {
            if address >= region.start() && address < region.end() {
                return true;
            }
        }

        false
    }

    /// Wait until relocation complete
    pub fn wait_relocation_complete(&self) -> Result<()> {
        while self.relocated_count.load(Ordering::Relaxed)
            < self.total_count.load(Ordering::Relaxed)
        {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        Ok(())
    }

    /// Complete relocation
    pub fn complete_relocation(&self) -> Result<()> {
        self.in_progress.store(false, Ordering::SeqCst);

        self.forwarding_tables.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Relocator forwarding_tables lock poisoned: {}", e))
        })?.clear();

        let relocation_set = self.relocation_set.lock()
            .map_err(|e| FgcError::LockPoisoned(format!("Relocator relocation_set lock poisoned: {}", e)))?
            .clone();
        for region in relocation_set {
            self.heap.return_region(region);
        }

        Ok(())
    }

    /// Get relocation progress
    pub fn progress(&self) -> RelocationProgress {
        RelocationProgress {
            relocated: self.relocated_count.load(Ordering::Relaxed),
            total: self.total_count.load(Ordering::Relaxed),
            bytes_relocated: self.bytes_relocated.load(Ordering::Relaxed),
            in_progress: self.in_progress.load(Ordering::Relaxed),
        }
    }

    /// Get copier statistics
    pub fn copy_stats(&self) -> copy::CopyStats {
        self.copier.stats()
    }

    /// Set total objects to relocate
    pub fn set_total_count(&self, total: u64) {
        self.total_count.store(total, Ordering::Relaxed);
    }

    /// Get bytes relocated
    pub fn bytes_relocated(&self) -> usize {
        self.bytes_relocated.load(Ordering::Relaxed)
    }
}

/// Relocation progress
#[derive(Debug, Default, Clone)]
pub struct RelocationProgress {
    /// Objects relocated
    pub relocated: u64,
    /// Total objects to relocate
    pub total: u64,
    /// Bytes relocated
    pub bytes_relocated: usize,
    /// Relocation in progress
    pub in_progress: bool,
}

impl std::fmt::Display for RelocationProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RelocationProgress {{ relocated: {}/{}, bytes: {}, in_progress: {} }}",
            self.relocated, self.total, self.bytes_relocated, self.in_progress
        )
    }
}
