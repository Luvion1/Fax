//! NUMA (Non-Uniform Memory Access) Management
//!
//! Module ini mengelola NUMA-aware allocation untuk sistem multi-socket.
//! Pada sistem NUMA, memory access ke local node lebih cepat dari remote node.
//!
//! NUMA Architecture:
//! ```
//! ┌─────────────┐     ┌─────────────┐
//! │  CPU Node 0 │     │  CPU Node 1 │
//! │  ┌───────┐  │     │  ┌───────┐  │
//! │  │ Cores │  │     │  │ Cores │  │
//! │  └───┬───┘  │     │  └───┬───┘  │
//! │      │     │     │      │     │
//! │  ┌───▼───┐  │     │  ┌───▼───┐  │
//! │  │ Local │  │     │  │ Local │  │
//! │  │ Memory│  │     │  │ Memory│  │
//! │  └───────┘  │     │  └───────┘  │
//! └─────────────┘     └─────────────┘
//!       │                   │
//!       └────────┬──────────┘
//!                │
//!         Interconnect (QPI/UPI)
//!         (slower access)
//! ```
//!
//! FGC NUMA Strategy:
//! 1. Thread mengalokasi memory di NUMA node tempat thread berjalan
//! 2. GC threads NUMA-aware saat collection
//! 3. Region di-allocate dari node-local free list
//! 4. Object migration jika thread pindah node

use crate::error::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// NumaManager - mengelola NUMA-aware allocation
///
/// Mengelola binding thread dan memory ke NUMA nodes.
pub struct NumaManager {
    /// Jumlah NUMA nodes di sistem
    node_count: usize,

    /// Current node untuk thread ini
    current_node: AtomicUsize,

    /// Region pools per NUMA node
    node_pools: HashMap<usize, NumaNodePool>,
}

impl NumaManager {
    /// Create new NUMA manager
    ///
    /// Detect NUMA topology dan initialize pools.
    pub fn new() -> Self {
        // Detect NUMA nodes
        // Note: Dalam implementasi nyata, ini baca dari /sys/devices/system/node/
        let node_count = detect_numa_nodes();

        let mut node_pools = HashMap::new();
        for node_id in 0..node_count {
            node_pools.insert(node_id, NumaNodePool::new(node_id));
        }

        Self {
            node_count,
            current_node: AtomicUsize::new(0),
            node_pools,
        }
    }

    /// Get current NUMA node untuk thread
    pub fn current_node(&self) -> usize {
        self.current_node.load(Ordering::Relaxed)
    }

    /// Set current NUMA node
    pub fn set_current_node(&self, node: usize) {
        if node < self.node_count {
            self.current_node.store(node, Ordering::Relaxed);
        }
    }

    /// Allocate dari NUMA node lokal
    pub fn allocate_local(&self, size: usize) -> Option<usize> {
        let node = self.current_node.load(Ordering::Relaxed);
        self.node_pools.get(&node)?.allocate(size)
    }

    /// Allocate dari NUMA node spesifik
    pub fn allocate_on_node(&self, node: usize, size: usize) -> Option<usize> {
        self.node_pools.get(&node)?.allocate(size)
    }

    /// Return memory ke NUMA node pool
    pub fn free_to_node(&self, node: usize, address: usize, size: usize) {
        if let Some(pool) = self.node_pools.get(&node) {
            pool.free(address, size);
        }
    }

    /// Get NUMA node count
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Get statistics untuk node
    pub fn node_stats(&self, node: usize) -> Option<NumaNodeStats> {
        self.node_pools.get(&node).map(|p| p.stats())
    }

    /// Bind thread ke NUMA node
    pub fn bind_thread_to_node(&self, node: usize) -> Result<()> {
        // Note: Dalam implementasi nyata:
        // Linux: numa_run_on_node(node)
        // Windows: SetThreadGroupAffinity
        self.set_current_node(node);
        Ok(())
    }

    /// Bind memory ke NUMA node
    pub fn bind_memory_to_node(&self, address: usize, size: usize, node: usize) -> Result<()> {
        // Note: Dalam implementasi nyata:
        // Linux: mbind(address, size, ...)
        // Windows: VirtualAllocNuma
        Ok(())
    }

    /// Migrate memory antar nodes
    pub fn migrate_memory(
        &self,
        from_node: usize,
        to_node: usize,
        address: usize,
        size: usize,
    ) -> Result<()> {
        // Note: Dalam implementasi nyata:
        // Linux: move_pages()
        Ok(())
    }
}

impl Default for NumaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// NumaNodePool - free list per NUMA node
///
/// Mengelola free regions untuk node tertentu.
struct NumaNodePool {
    /// Node ID
    node_id: usize,

    /// Free regions: size -> list of addresses
    free_regions: std::sync::Mutex<HashMap<usize, Vec<usize>>>,

    /// Total allocated bytes
    allocated_bytes: AtomicUsize,

    /// Allocation count
    allocation_count: AtomicUsize,
}

impl NumaNodePool {
    fn new(node_id: usize) -> Self {
        Self {
            node_id,
            free_regions: std::sync::Mutex::new(HashMap::new()),
            allocated_bytes: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
        }
    }

    /// Allocate dari pool
    fn allocate(&self, size: usize) -> Option<usize> {
        let mut free_regions = self.free_regions.lock().unwrap();

        // Cari free region yang cukup besar
        for (&region_size, addresses) in free_regions.iter_mut() {
            if region_size >= size && !addresses.is_empty() {
                let address = addresses.pop().unwrap();

                if addresses.is_empty() {
                    free_regions.remove(&region_size);
                }

                self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
                self.allocation_count.fetch_add(1, Ordering::Relaxed);

                return Some(address);
            }
        }

        None
    }

    /// Free ke pool
    fn free(&self, address: usize, size: usize) {
        let mut free_regions = self.free_regions.lock().unwrap();

        free_regions
            .entry(size)
            .or_insert_with(Vec::new)
            .push(address);

        self.allocated_bytes.fetch_sub(size, Ordering::Relaxed);
        self.allocation_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get statistics
    fn stats(&self) -> NumaNodeStats {
        NumaNodeStats {
            node_id: self.node_id,
            allocated_bytes: self.allocated_bytes.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
        }
    }
}

/// Statistics untuk NUMA node
#[derive(Debug, Default)]
pub struct NumaNodeStats {
    /// Node ID
    pub node_id: usize,
    /// Total allocated bytes
    pub allocated_bytes: usize,
    /// Allocation count
    pub allocation_count: usize,
}

/// Detect jumlah NUMA nodes di sistem
fn detect_numa_nodes() -> usize {
    // Note: Dalam implementasi nyata, detect dari:
    // Linux: /sys/devices/system/node/online
    // Windows: GetNumaNodeProcessorMask

    // Default: 1 node (UMA system)
    1
}

/// Get current thread's NUMA node
pub fn get_current_numa_node() -> usize {
    // Note: Dalam implementasi nyata:
    // Linux: numa_node_of_cpu(sched_getcpu())
    // Windows: GetCurrentProcessorNumberEx + GetNumaProcessorNode

    0 // Default
}
