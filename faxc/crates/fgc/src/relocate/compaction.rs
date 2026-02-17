//! Compaction - Region Compaction
//!
//! Module untuk region compaction strategy.
//! Memilih region mana yang perlu di-compact berdasarkan
//! garbage ratio dan fragmentation.

use crate::heap::Region;
use std::sync::Arc;

/// Compactor - manager untuk region compaction
///
/// Memilih regions untuk compaction dan manage prosesnya.
pub struct Compactor {
    /// Compaction in progress
    in_progress: std::sync::atomic::AtomicBool,
}

impl Compactor {
    pub fn new() -> Self {
        Self {
            in_progress: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Select regions untuk compaction
    pub fn select_regions(&self, regions: &[Arc<Region>], max_size: usize) -> Vec<Arc<Region>> {
        let mut candidates: Vec<_> = regions
            .iter()
            .filter(|r| r.garbage_ratio() > 0.3)
            .map(|r| (r.clone(), r.garbage_ratio()))
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut selected = Vec::new();
        let mut total_size = 0;

        for (region, _) in candidates {
            if total_size + region.size() > max_size {
                break;
            }
            selected.push(region.clone());
            total_size += region.size();
        }

        selected
    }

    /// Start compaction
    pub fn start(&self) {
        self.in_progress
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Complete compaction
    pub fn complete(&self) {
        self.in_progress
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if compaction in progress
    pub fn is_compacting(&self) -> bool {
        self.in_progress.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for Compactor {
    fn default() -> Self {
        Self::new()
    }
}
