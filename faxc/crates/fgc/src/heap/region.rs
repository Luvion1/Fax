//! Region Implementation - Unit of Heap Management

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Region type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    Small,
    Medium,
    Large,
}

impl RegionType {
    pub fn default_size(&self) -> usize {
        match self {
            RegionType::Small => 2 * 1024 * 1024,
            RegionType::Medium => 32 * 1024 * 1024,
            RegionType::Large => 0,
        }
    }
}

/// Region state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionState {
    Allocating,
    Allocated,
    Relocating,
    Relocated,
    Free,
}

/// Generation enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    Young,
    Old,
}

/// Region - unit of heap management
pub struct Region {
    start: AtomicUsize,
    size: usize,
    region_type: RegionType,
    state: std::sync::Mutex<RegionState>,
    top: AtomicUsize,
    end: usize,
    mark_bitmap: Vec<AtomicU64>,
    forwarding_table: std::sync::Mutex<Option<Arc<crate::relocate::ForwardingTable>>>,
    #[allow(dead_code)]
    numa_node: usize,
    generation: Generation,
    allocation_count: AtomicUsize,
    mark_bit: AtomicBool,
    #[allow(dead_code)]
    needs_commit: bool,
}

impl Region {
    pub fn with_address(
        start: usize,
        region_type: RegionType,
        size: usize,
        generation: Generation,
    ) -> Result<Arc<Self>> {
        if size == 0 {
            return Err(FgcError::InvalidArgument(
                "Region size must be greater than 0".to_string(),
            ));
        }
        if start == 0 {
            return Err(FgcError::InvalidArgument(
                "Region start address cannot be null".to_string(),
            ));
        }

        let bitmap_words = size.div_ceil(64);
        let mark_bitmap: Vec<AtomicU64> = (0..bitmap_words).map(|_| AtomicU64::new(0)).collect();

        Ok(Arc::new(Self {
            start: AtomicUsize::new(start),
            size,
            region_type,
            state: std::sync::Mutex::new(RegionState::Free),
            top: AtomicUsize::new(start),
            end: start + size,
            mark_bitmap,
            forwarding_table: std::sync::Mutex::new(None),
            numa_node: 0,
            generation,
            allocation_count: AtomicUsize::new(0),
            mark_bit: AtomicBool::new(false),
            needs_commit: false,
        }))
    }

    #[cfg(test)]
    pub fn for_testing(start: usize, size: usize) -> Arc<Self> {
        Self::with_address(start, RegionType::Small, size, Generation::Young)
            .expect("Failed to create test region")
    }

    #[inline]
    pub fn allocate(&self, size: usize, alignment: usize) -> Result<usize> {
        {
            let state = self.state.lock().map_err(|e| {
                FgcError::LockPoisoned(format!("Region state lock poisoned: {}", e))
            })?;
            if *state != RegionState::Allocating && *state != RegionState::Free {
                return Err(FgcError::InvalidState {
                    expected: "Allocating or Free".to_string(),
                    actual: format!("{:?}", *state),
                });
            }
        }

        let aligned_size = (size + alignment - 1) & !(alignment - 1);
        let mut current_top = self.top.load(Ordering::Relaxed);

        loop {
            let new_top = current_top
                .checked_add(aligned_size)
                .ok_or(FgcError::OutOfMemory {
                    requested: size,
                    available: 0,
                })?;

            if new_top > self.end {
                return Err(FgcError::OutOfMemory {
                    requested: size,
                    available: self.end.saturating_sub(current_top),
                });
            }

            match self.top.compare_exchange_weak(
                current_top,
                new_top,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.allocation_count.fetch_add(1, Ordering::Relaxed);
                    return Ok(current_top);
                },
                Err(actual) => current_top = actual,
            }
        }
    }

    pub fn mark_object(&self, address: usize, _size: usize) {
        let offset = address.saturating_sub(self.start.load(Ordering::Relaxed));
        if offset >= self.size {
            return;
        }
        let bit_index = offset / 64;
        let bit_offset = offset % 64;
        if bit_index < self.mark_bitmap.len() {
            self.mark_bitmap[bit_index].fetch_or(1 << bit_offset, Ordering::Relaxed);
        }
    }

    pub fn is_marked(&self, address: usize) -> bool {
        let offset = address.saturating_sub(self.start.load(Ordering::Relaxed));
        if offset >= self.size {
            return false;
        }
        let bit_index = offset / 64;
        let bit_offset = offset % 64;
        if bit_index >= self.mark_bitmap.len() {
            return false;
        }
        (self.mark_bitmap[bit_index].load(Ordering::Relaxed) & (1 << bit_offset)) != 0
    }

    pub fn count_marked(&self) -> usize {
        self.mark_bitmap
            .iter()
            .map(|word| word.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }

    pub fn reset(&self) -> Result<()> {
        let marked_count = self.count_marked();
        if marked_count > 0 {
            return Err(FgcError::InvalidState {
                expected: "no live objects".to_string(),
                actual: format!("{} live objects", marked_count),
            });
        }

        {
            let mut state = self.state.lock().map_err(|e| {
                FgcError::LockPoisoned(format!("Region state lock poisoned: {}", e))
            })?;
            *state = RegionState::Free;
        }

        self.top
            .store(self.start.load(Ordering::Relaxed), Ordering::SeqCst);
        for word in &self.mark_bitmap {
            word.store(0, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn region_type(&self) -> RegionType {
        self.region_type
    }

    pub fn generation(&self) -> Generation {
        self.generation
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn start(&self) -> usize {
        self.start.load(Ordering::Relaxed)
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn used(&self) -> usize {
        self.top
            .load(Ordering::Relaxed)
            .saturating_sub(self.start.load(Ordering::Relaxed))
    }

    pub fn contains(&self, address: usize) -> bool {
        address >= self.start() && address < self.end()
    }

    pub fn garbage_ratio(&self) -> f32 {
        let total = self.used();
        if total == 0 {
            return 0.0;
        }
        let marked = self.count_marked();
        1.0 - (marked as f32 / total as f32)
    }

    pub fn state(&self) -> RegionState {
        match self.state.lock() {
            Ok(guard) => *guard,
            Err(e) => {
                log::error!("Region state lock poisoned: {}", e);
                RegionState::Free
            },
        }
    }

    pub fn set_state(&self, state: RegionState) {
        match self.state.lock() {
            Ok(mut guard) => {
                *guard = state;
            },
            Err(e) => {
                log::error!("Region state lock poisoned: {}", e);
            },
        }
    }

    pub fn flip_mark_bits(&self) {
        let current = self.mark_bit.load(Ordering::Relaxed);
        self.mark_bit.store(!current, Ordering::Relaxed);
    }

    pub fn setup_forwarding(&self) -> Result<()> {
        let mut guard = self.forwarding_table.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("Forwarding table lock poisoned: {}", e))
        })?;
        *guard = Some(Arc::new(crate::relocate::ForwardingTable::new(
            self.start(),
            self.size,
        )));
        Ok(())
    }

    pub fn forwarding_table(&self) -> Option<Arc<crate::relocate::ForwardingTable>> {
        match self.forwarding_table.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("Region forwarding_table lock poisoned: {}", e);
                None
            },
        }
    }

    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    pub fn mark_bit(&self) -> bool {
        self.mark_bit.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_type_default_size() {
        assert_eq!(RegionType::Small.default_size(), 2 * 1024 * 1024);
        assert_eq!(RegionType::Medium.default_size(), 32 * 1024 * 1024);
        assert_eq!(RegionType::Large.default_size(), 0);
    }

    #[test]
    fn test_region_creation() {
        let region = Region::for_testing(0x1000, 4096);
        assert_eq!(region.start(), 0x1000);
        assert_eq!(region.size(), 4096);
        assert_eq!(region.end(), 0x1000 + 4096);
        assert_eq!(region.region_type(), RegionType::Small);
        assert_eq!(region.generation(), Generation::Young);
    }

    #[test]
    fn test_region_creation_invalid_params() {
        assert!(Region::with_address(0x1000, RegionType::Small, 0, Generation::Young).is_err());
        assert!(Region::with_address(0, RegionType::Small, 4096, Generation::Young).is_err());
    }

    #[test]
    fn test_region_allocate() {
        let region = Region::for_testing(0x1000, 4096);
        region.set_state(RegionState::Allocating);

        let addr1 = region.allocate(64, 8).unwrap();
        let addr2 = region.allocate(128, 8).unwrap();

        assert!(addr1 >= 0x1000 && addr1 < 0x2000);
        assert!(addr2 > addr1);
        assert_eq!(addr1 % 8, 0);
        assert_eq!(addr2 % 8, 0);
    }

    #[test]
    fn test_region_allocate_full() {
        let region = Region::for_testing(0x1000, 256);
        region.set_state(RegionState::Allocating);
        let _ = region.allocate(200, 8).unwrap();
        let result = region.allocate(100, 8);
        assert!(matches!(result, Err(FgcError::OutOfMemory { .. })));
    }

    #[test]
    fn test_region_mark_and_check() {
        let region = Region::for_testing(0x1000, 4096);
        let addr = region.allocate(64, 8).unwrap();
        region.mark_object(addr, 64);
        assert!(region.is_marked(addr));
        assert!(!region.is_marked(addr + 100));
    }

    #[test]
    fn test_region_count_marked() {
        let region = Region::for_testing(0x1000, 4096);
        let addr1 = region.allocate(64, 8).unwrap();
        let addr2 = region.allocate(64, 8).unwrap();
        let addr3 = region.allocate(64, 8).unwrap();
        region.mark_object(addr1, 64);
        region.mark_object(addr3, 64);
        assert_eq!(region.count_marked(), 2);
    }

    #[test]
    fn test_region_reset_empty() {
        let region = Region::for_testing(0x1000, 4096);
        region.set_state(RegionState::Allocating);
        let result = region.reset();
        assert!(result.is_ok());
        assert_eq!(region.state(), RegionState::Free);
    }

    #[test]
    fn test_region_reset_with_live_objects() {
        let region = Region::for_testing(0x1000, 4096);
        region.set_state(RegionState::Allocating);
        let addr = region.allocate(64, 8).unwrap();
        region.mark_object(addr, 64);
        let result = region.reset();
        assert!(matches!(result, Err(FgcError::InvalidState { .. })));
    }

    #[test]
    fn test_region_contains() {
        let region = Region::for_testing(0x1000, 4096);
        assert!(region.contains(0x1000));
        assert!(region.contains(0x1500));
        assert!(!region.contains(0x0FFF));
        assert!(!region.contains(0x2000));
    }

    #[test]
    fn test_region_used() {
        let region = Region::for_testing(0x1000, 4096);
        region.set_state(RegionState::Allocating);
        assert_eq!(region.used(), 0);
        let _ = region.allocate(100, 8).unwrap();
        assert!(region.used() > 0);
    }

    #[test]
    fn test_region_garbage_ratio() {
        let region = Region::for_testing(0x1000, 4096);
        region.set_state(RegionState::Allocating);
        assert_eq!(region.garbage_ratio(), 0.0);
        let addr1 = region.allocate(100, 8).unwrap();
        let addr2 = region.allocate(100, 8).unwrap();
        region.mark_object(addr1, 100);
        let ratio = region.garbage_ratio();
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn test_region_state_transitions() {
        let region = Region::for_testing(0x1000, 4096);
        assert_eq!(region.state(), RegionState::Free);
        region.set_state(RegionState::Allocating);
        assert_eq!(region.state(), RegionState::Allocating);
        region.set_state(RegionState::Allocated);
        assert_eq!(region.state(), RegionState::Allocated);
    }

    #[test]
    fn test_region_flip_mark_bits() {
        let region = Region::for_testing(0x1000, 4096);
        assert!(!region.mark_bit());
        region.flip_mark_bits();
        assert!(region.mark_bit());
        region.flip_mark_bits();
        assert!(!region.mark_bit());
    }

    #[test]
    fn test_region_allocation_count() {
        let region = Region::for_testing(0x1000, 4096);
        region.set_state(RegionState::Allocating);
        assert_eq!(region.allocation_count(), 0);
        let _ = region.allocate(64, 8).unwrap();
        let _ = region.allocate(64, 8).unwrap();
        assert_eq!(region.allocation_count(), 2);
    }

    #[test]
    fn test_region_setup_forwarding() {
        let region = Region::for_testing(0x1000, 4096);
        let result = region.setup_forwarding();
        assert!(result.is_ok());
    }

    #[test]
    fn test_region_forwarding_table() {
        let region = Region::for_testing(0x1000, 4096);
        assert!(region.forwarding_table().is_none());
        region.setup_forwarding().unwrap();
        assert!(region.forwarding_table().is_some());
    }

    #[test]
    fn test_region_concurrent_allocation() {
        use std::sync::Arc;
        use std::thread;

        let region = Region::for_testing(0x1000, 64 * 1024);
        region.set_state(RegionState::Allocating);
        let region = Arc::new(region);
        let mut handles = Vec::new();

        for _ in 0..4 {
            let region = Arc::clone(&region);
            let handle = thread::spawn(move || {
                let mut addrs = Vec::new();
                for _ in 0..10 {
                    if let Ok(addr) = region.allocate(64, 8) {
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

        use std::collections::HashSet;
        let unique: HashSet<_> = all_addrs.iter().collect();
        assert_eq!(
            unique.len(),
            all_addrs.len(),
            "Concurrent allocations should be unique"
        );
    }
}
