//! Test Utilities for FGC Bug-Finding Test Suite
//!
//! This module provides test utilities that enforce STRICT assertions.
//! NO tolerances, NO excuses for stub behavior.
//!
//! ============================================================================
//! CRITICAL: These utilities are designed to FIND BUGS, not to have passing tests.
//! ============================================================================

use fgc::{GarbageCollector, GcConfig, GcGeneration, GcReason, GcState, Result};
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Default heap size for tests (64MB)
pub const DEFAULT_HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Default alignment for allocations (8 bytes)
pub const DEFAULT_ALIGNMENT: usize = 8;

/// Maximum test duration before timeout
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// ============================================================================
/// GC FIXTURE
/// ============================================================================

/// Test fixture for GC operations
/// 
/// Provides a clean GC instance for each test with automatic cleanup.
pub struct GcFixture {
    pub gc: Arc<GarbageCollector>,
    pub config: GcConfig,
}

impl GcFixture {
    /// Create fixture with default configuration
    ///
    /// **Bug this finds:** Configuration validation bugs, initialization failures
    pub fn with_defaults() -> Self {
        let config = GcConfig {
            max_heap_size: DEFAULT_HEAP_SIZE,
            min_heap_size: 16 * 1024 * 1024, // 16MB minimum, must be <= max_heap_size
            initial_heap_size: 16 * 1024 * 1024, // Must be between min and max
            soft_max_heap_size: DEFAULT_HEAP_SIZE,
            verbose: false,
            ..Default::default()
        };

        let gc = Arc::new(
            GarbageCollector::new(config.clone())
                .expect("GC initialization should succeed with valid config")
        );

        Self { gc, config }
    }

    /// Create fixture with custom heap size
    ///
    /// **Bug this finds:** Heap size validation bugs, OOM handling
    pub fn with_heap_size(heap_size: usize) -> Self {
        let min_size = (heap_size / 4).max(16 * 1024 * 1024);
        let actual_min = min_size.min(heap_size);
        let config = GcConfig {
            max_heap_size: heap_size,
            min_heap_size: actual_min,
            initial_heap_size: actual_min,
            soft_max_heap_size: heap_size,
            verbose: false,
            ..Default::default()
        };

        let gc = Arc::new(
            GarbageCollector::new(config.clone())
                .expect("GC initialization should succeed with valid heap size")
        );

        Self { gc, config }
    }

    /// Create fixture with minimal heap (for OOM tests)
    ///
    /// **Bug this finds:** Edge case handling, minimum heap validation
    pub fn with_minimal_heap() -> Self {
        let config = GcConfig {
            max_heap_size: 1024 * 1024, // 1MB minimum
            min_heap_size: 512 * 1024,  // 512KB min
            initial_heap_size: 512 * 1024,
            soft_max_heap_size: 1024 * 1024,
            verbose: false,
            ..Default::default()
        };

        let gc = Arc::new(
            GarbageCollector::new(config.clone())
                .expect("GC initialization should succeed with minimal heap")
        );

        Self { gc, config }
    }
    
    /// Allocate memory and return address
    /// 
    /// **Bug this finds:** Allocation failures, alignment bugs, duplicate addresses
    pub fn allocate(&self, size: usize) -> usize {
        self.gc.heap()
            .allocate_tlab_memory(size)
            .unwrap_or_else(|e| {
                panic!("Allocation of {} bytes failed: {:?}", size, e);
            })
    }
    
    /// Allocate multiple objects and return addresses
    /// 
    /// **Bug this finds:** Race conditions in allocation, duplicate addresses
    pub fn allocate_many(&self, count: usize, size: usize) -> Vec<usize> {
        (0..count)
            .map(|_| self.allocate(size))
            .collect()
    }
    
    /// Trigger GC and wait for completion
    ///
    /// **Bug this finds:** GC not completing, state machine bugs, deadlock
    pub fn trigger_gc(&self, generation: GcGeneration) {
        // Set the generation via request_gc, then run GC synchronously
        self.gc.request_gc(generation, GcReason::Explicit);
        self.gc.collect().expect("GC should complete successfully");
    }
    
    /// Get current GC state
    pub fn state(&self) -> GcState {
        self.gc.state()
    }
    
    /// Get GC cycle count
    pub fn cycle_count(&self) -> u64 {
        self.gc.cycle_count()
    }
}

impl Drop for GcFixture {
    fn drop(&mut self) {
        // Graceful shutdown
        let _ = self.gc.shutdown();
    }
}

/// ============================================================================
/// STRICT ASSERTION HELPERS
/// ============================================================================

/// Assert that all addresses are unique
/// 
/// **Bug this finds:** Race conditions in allocator, duplicate address bugs
/// **Tolerance:** ZERO - Any duplicate is a bug
#[track_caller]
pub fn assert_all_addresses_unique(addresses: &[usize], context: &str) {
    let unique: HashSet<_> = addresses.iter().collect();
    
    assert_eq!(
        unique.len(),
        addresses.len(),
        "{}: Found {} duplicate addresses out of {} - allocator is NOT thread-safe! \
         Duplicates indicate race condition in allocation logic.",
        context,
        addresses.len() - unique.len(),
        addresses.len()
    );
}

/// Assert that address is properly aligned
/// 
/// **Bug this finds:** Alignment bugs, memory corruption potential
/// **Tolerance:** ZERO - Misaligned access causes undefined behavior
#[track_caller]
pub fn assert_address_aligned(address: usize, alignment: usize, context: &str) {
    assert_eq!(
        address % alignment,
        0,
        "{}: Address {:#x} is not {}-byte aligned - alignment bug in allocator. \
         Misaligned access can cause SIGBUS or performance degradation.",
        context,
        address,
        alignment
    );
}

/// Assert that address is within heap bounds
/// 
/// **Bug this finds:** Heap overflow, address calculation bugs
/// **Tolerance:** ZERO - Out-of-bounds access is memory corruption
#[track_caller]
pub fn assert_address_in_bounds(address: usize, heap_base: usize, heap_size: usize, context: &str) {
    assert!(
        address >= heap_base,
        "{}: Address {:#x} is below heap base {:#x} - pointer calculation bug",
        context,
        address,
        heap_base
    );
    
    assert!(
        address < heap_base + heap_size,
        "{}: Address {:#x} is beyond heap end {:#x} - heap overflow bug",
        context,
        address,
        heap_base + heap_size
    );
}

/// Assert that addresses are monotonically increasing (for bump allocator)
/// 
/// **Bug this finds:** Bump pointer regression, allocation order bugs
/// **Tolerance:** ZERO - Non-monotonic allocation indicates serious bug
#[track_caller]
pub fn assert_addresses_monotonic(addresses: &[usize], context: &str) {
    for i in 1..addresses.len() {
        assert!(
            addresses[i] >= addresses[i - 1],
            "{}: Address regression detected - address[{}] = {:#x} < address[{}] = {:#x}. \
             Bump allocator should only move forward.",
            context,
            i, addresses[i],
            i - 1, addresses[i - 1]
        );
    }
}

/// Assert that GC completed successfully
/// 
/// **Bug this finds:** GC not completing, state machine stuck
#[track_caller]
pub fn assert_gc_completed(fixture: &GcFixture, context: &str) {
    assert!(
        !fixture.gc.is_collecting(),
        "{}: GC is still collecting - GC did not complete properly",
        context
    );
    
    assert_eq!(
        fixture.gc.state(),
        GcState::Idle,
        "{}: GC state is {:?}, expected Idle - state machine bug",
        context,
        fixture.gc.state()
    );
}

/// Assert that GC cycle count increased
/// 
/// **Bug this finds:** GC not actually running, cycle counter bug
#[track_caller]
pub fn assert_gc_cycle_increased(before: u64, after: u64, context: &str) {
    assert!(
        after > before,
        "{}: GC cycle count did not increase (before={}, after={}) - GC did not execute",
        context,
        before,
        after
    );
}

/// Assert that operation completed within timeout
/// 
/// **Bug this finds:** Deadlock, infinite loop, performance regression
#[track_caller]
pub fn assert_completed_within_timeout<F, R>(operation: F, timeout: Duration, context: &str) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = operation();
    let elapsed = start.elapsed();
    
    assert!(
        elapsed < timeout,
        "{}: Operation took {:?}, exceeded timeout of {:?} - possible deadlock or performance bug",
        context,
        elapsed,
        timeout
    );
    
    result
}

/// ============================================================================
/// CONCURRENT TEST HELPERS
/// ============================================================================

/// Run concurrent allocation test with multiple threads
///
/// **Bug this finds:** Race conditions, thread safety issues
pub fn run_concurrent_allocations<F>(
    fixture: &GcFixture,
    thread_count: usize,
    allocations_per_thread: usize,
    allocation_size: usize,
    validator: F,
) where
    F: Fn(Vec<usize>) + Send + 'static,
{
    let mut handles = Vec::with_capacity(thread_count);

    for thread_id in 0..thread_count {
        let gc: Arc<fgc::GarbageCollector> = Arc::clone(&fixture.gc);

        let handle = thread::spawn(move || {
            let mut addresses = Vec::with_capacity(allocations_per_thread);
            
            for _ in 0..allocations_per_thread {
                let addr = gc.heap()
                    .allocate_tlab_memory(allocation_size)
                    .unwrap_or_else(|e| {
                        panic!("Thread {} allocation failed: {:?}", thread_id, e);
                    });
                addresses.push(addr);
            }
            
            addresses
        });
        
        handles.push(handle);
    }
    
    // Collect all addresses
    let mut all_addresses = Vec::new();
    for handle in handles {
        let addresses = handle.join()
            .unwrap_or_else(|e| {
                panic!("Thread panicked: {:?}", e);
            });
        all_addresses.extend(addresses);
    }
    
    // Run validator
    validator(all_addresses);
}

/// ============================================================================
/// POINTER VALIDATION
/// ============================================================================

/// Validate colored pointer properties
/// 
/// **Bug this finds:** Color bit corruption, pointer encoding bugs
pub struct ColoredPointerValidator {
    pub address: usize,
    pub raw: usize,
}

impl ColoredPointerValidator {
    pub fn new(address: usize, raw: usize) -> Self {
        Self { address, raw }
    }
    
    /// Assert that address bits are preserved correctly
    /// 
    /// **Bug this finds:** Address truncation, bit manipulation bugs
    #[track_caller]
    pub fn assert_address_preserved(&self, expected_address: usize) {
        assert_eq!(
            self.address,
            expected_address,
            "Address mismatch: expected {:#x}, got {:#x} - address bits corrupted",
            expected_address,
            self.address
        );
    }
    
    /// Assert that color bits are in correct position (bits 44-47)
    /// 
    /// **Bug this finds:** Wrong bit positions, mask bugs
    #[track_caller]
    pub fn assert_color_bits_position(&self) {
        // Color bits should be in bits 44-47
        let color_bits = (self.raw >> 44) & 0xF;
        let address_bits = self.raw & ((1u64 << 44) - 1) as usize;
        
        assert_eq!(
            address_bits,
            self.address,
            "Address extraction failed: extracted {:#x}, expected {:#x}",
            address_bits,
            self.address
        );
        
        // Verify no overlap between color and address bits
        assert_eq!(
            color_bits << 44 | address_bits,
            self.raw,
            "Bit layout corrupted: color={:#x}, address={:#x}, raw={:#x}",
            color_bits,
            address_bits,
            self.raw
        );
    }
}

/// ============================================================================
/// MEMORY SAFETY CHECKS
/// ============================================================================

/// Check for potential memory safety issues
/// 
/// **Bug this finds:** Double-free, use-after-free, memory leaks
pub struct MemorySafetyChecker {
    allocated: HashSet<usize>,
    freed: HashSet<usize>,
}

impl MemorySafetyChecker {
    pub fn new() -> Self {
        Self {
            allocated: HashSet::new(),
            freed: HashSet::new(),
        }
    }
    
    /// Record allocation
    pub fn record_allocation(&mut self, address: usize) {
        if self.freed.contains(&address) {
            panic!(
                "USE-AFTER-FREE detected: Address {:#x} was freed but allocated again",
                address
            );
        }
        
        if self.allocated.contains(&address) {
            panic!(
                "DOUBLE-ALLOC detected: Address {:#x} already allocated",
                address
            );
        }
        
        self.allocated.insert(address);
    }
    
    /// Record deallocation
    pub fn record_free(&mut self, address: usize) {
        if !self.allocated.contains(&address) {
            panic!(
                "FREE-OF-UNALLOCATED detected: Address {:#x} was never allocated",
                address
            );
        }
        
        if self.freed.contains(&address) {
            panic!(
                "DOUBLE-FREE detected: Address {:#x} already freed",
                address
            );
        }
        
        self.freed.insert(address);
    }
    
    /// Check for memory leaks (allocated but not freed)
    #[track_caller]
    pub fn assert_no_leaks(&self, context: &str) {
        let leaked: Vec<_> = self.allocated.difference(&self.freed).collect();
        
        assert!(
            leaked.is_empty(),
            "{}: Memory leak detected - {} addresses allocated but not freed: {:?}",
            context,
            leaked.len(),
            leaked.iter().take(10).collect::<Vec<_>>()
        );
    }
}

impl Default for MemorySafetyChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// STATISTICS VALIDATION
/// ============================================================================

/// Validate GC statistics
/// 
/// **Bug this finds:** Stats tracking bugs, incorrect metrics
#[track_caller]
pub fn assert_gc_stats_valid(before_used: usize, after_used: usize, collected: bool) {
    if collected {
        // After successful GC, used memory should decrease or stay same
        // (stay same if nothing was garbage)
        assert!(
            after_used <= before_used,
            "GC stats bug: used memory increased after collection \
             (before={} bytes, after={} bytes)",
            before_used,
            after_used
        );
    }
}
