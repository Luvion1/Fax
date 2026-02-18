//! NUMA (Non-Uniform Memory Access) Management
//!
//! This module manages NUMA-aware allocation for multi-socket systems.
//! On NUMA systems, memory access to local node is faster than remote node.
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
//! 1. Thread allocates memory on the NUMA node where the thread runs
//! 2. GC threads are NUMA-aware during collection
//! 3. Regions are allocated from node-local free list
//! 4. Object migration if thread moves to different node

use crate::error::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// NumaManager - manages NUMA-aware allocation
///
/// Manages thread and memory binding to NUMA nodes.
pub struct NumaManager {
    /// Number of NUMA nodes in system
    node_count: usize,

    /// Current node for this thread
    current_node: AtomicUsize,

    /// Region pools per NUMA node
    node_pools: HashMap<usize, NumaNodePool>,
}

impl NumaManager {
    /// Create new NUMA manager
    ///
    /// Detect NUMA topology and initialize pools.
    pub fn new() -> Self {
        // Detect NUMA nodes
        // Note: In real implementation, this would read from /sys/devices/system/node/
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

    /// Get current NUMA node for thread
    pub fn current_node(&self) -> usize {
        self.current_node.load(Ordering::Relaxed)
    }

    /// Set current NUMA node
    pub fn set_current_node(&self, node: usize) {
        if node < self.node_count {
            self.current_node.store(node, Ordering::Relaxed);
        }
    }

    /// Allocate from NUMA node local
    pub fn allocate_local(&self, size: usize) -> Option<usize> {
        let node = self.current_node.load(Ordering::Relaxed);
        self.node_pools.get(&node)?.allocate(size)
    }

    /// Allocate from specific NUMA node
    pub fn allocate_on_node(&self, node: usize, size: usize) -> Option<usize> {
        self.node_pools.get(&node)?.allocate(size)
    }

    /// Return memory to NUMA node pool
    pub fn free_to_node(&self, node: usize, address: usize, size: usize) {
        if let Some(pool) = self.node_pools.get(&node) {
            pool.free(address, size);
        }
    }

    /// Get NUMA node count
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Get statistics for node
    pub fn node_stats(&self, node: usize) -> Option<NumaNodeStats> {
        self.node_pools.get(&node).map(|p| p.stats())
    }

    /// Bind thread to NUMA node
    pub fn bind_thread_to_node(&self, node: usize) -> Result<()> {
        // Note: In real implementation:
        // Linux: numa_run_on_node(node)
        // Windows: SetThreadGroupAffinity
        self.set_current_node(node);
        Ok(())
    }

    /// Bind memory to NUMA node
    pub fn bind_memory_to_node(&self, address: usize, size: usize, node: usize) -> Result<()> {
        // Note: In real implementation:
        // Linux: mbind(address, size, ...)
        // Windows: VirtualAllocNuma
        Ok(())
    }

    /// Migrate memory between nodes
    pub fn migrate_memory(
        &self,
        from_node: usize,
        to_node: usize,
        address: usize,
        size: usize,
    ) -> Result<()> {
        // Note: In real implementation:
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
/// Manages free regions for a specific node.
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

    /// Allocate from pool
    fn allocate(&self, size: usize) -> Option<usize> {
        let mut free_regions = self.free_regions.lock().unwrap();

        // Find free region that is large enough
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

    /// Free to pool
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

/// Statistics for NUMA node
#[derive(Debug, Default)]
pub struct NumaNodeStats {
    /// Node ID
    pub node_id: usize,
    /// Total allocated bytes
    pub allocated_bytes: usize,
    /// Allocation count
    pub allocation_count: usize,
}

/// Detect number of NUMA nodes in system
fn detect_numa_nodes() -> usize {
    // Note: In real implementation, detect from:
    // Linux: /sys/devices/system/node/online
    // Windows: GetNumaNodeProcessorMask

    // Default: 1 node (UMA system)
    1
}

/// Get current thread's NUMA node
pub fn get_current_numa_node() -> usize {
    // Note: In real implementation:
    // Linux: numa_node_of_cpu(sched_getcpu())
    // Windows: GetCurrentProcessorNumberEx + GetNumaProcessorNode

    0 // Default
}
