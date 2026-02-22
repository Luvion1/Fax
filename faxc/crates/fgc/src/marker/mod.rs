//! Marker Module - Concurrent Marking System
//!
//! This module implements concurrent marking to identify
//! which objects are still "alive" (referenced) and which are garbage.
//!
//! Marking Algorithm:
//! - Tri-color marking (White, Grey, Black)
//! - Concurrent with application running
//! - Load barriers mark objects when accessed
//!
//! Marking Phases:
//! 1. Pause Mark Start (STW < 1ms) - Setup, scan roots
//! 2. Concurrent Mark - Mark objects via load barriers
//! 3. Pause Mark End (STW < 1ms) - Finalize marking
//!
//! Root Types:
//! - Stack roots (local variables, registers)
//! - Global roots (static variables)
//! - Class roots (loaded classes)
//! - VM internal roots

pub mod bitmap;
pub mod gc_threads;
pub mod mark_queue;
pub mod object_scanner;
pub mod roots;
pub mod stack_scan;

pub use bitmap::MarkBitmap;
pub use gc_threads::{GcPoolStats, GcThreadPool, GcWorker};
pub use mark_queue::MarkQueue;
pub use object_scanner::{
    scan_object, scan_object_conservative, scan_object_precise, ObjectScanStats,
};
pub use roots::{RootDescriptor, RootHandle, RootScanner, RootStats, RootType};

use crate::error::Result;
use crate::heap::Heap;
use indexmap::IndexMap;
use parking_lot::Mutex as ParkingMutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Marker - orchestrator for concurrent marking
///
/// Marker manages the entire marking process:
/// - Root scanning
/// - Mark queue management
/// - GC thread coordination
/// - Termination detection
pub struct Marker {
    /// Reference to heap
    heap: Arc<Heap>,

    /// Global mark queue
    global_queue: Arc<MarkQueue>,

    /// Mark bitmaps per region
    bitmaps: std::sync::Mutex<IndexMap<usize, MarkBitmap>>,

    /// Root scanner
    root_scanner: RootScanner,

    /// Current mark bit (false = Marked0, true = Marked1)
    current_mark_bit: AtomicBool,

    /// Marking in progress
    marking_in_progress: AtomicBool,

    /// Marked object count
    marked_count: AtomicU64,

    /// GC threads for marking
    thread_count: usize,

    /// GC thread pool (optional, created during marking)
    gc_thread_pool: ParkingMutex<Option<Arc<GcThreadPool>>>,
}

impl Marker {
    /// Create new marker
    ///
    /// # Arguments
    /// * `heap` - Heap reference
    pub fn new(heap: Arc<Heap>) -> Self {
        Self {
            heap,
            global_queue: Arc::new(MarkQueue::new()),
            bitmaps: std::sync::Mutex::new(IndexMap::new()),
            root_scanner: RootScanner::new(),
            current_mark_bit: AtomicBool::new(false),
            marking_in_progress: AtomicBool::new(false),
            marked_count: AtomicU64::new(0),
            thread_count: 4, // Default
            gc_thread_pool: ParkingMutex::new(None),
        }
    }

    /// Start concurrent marking phase
    ///
    /// Called when GC cycle enters marking phase.
    /// Setup state and spawn GC threads.
    pub fn start_marking(&self) -> Result<()> {
        self.marking_in_progress.store(true, Ordering::SeqCst);
        self.marked_count.store(0, Ordering::Relaxed);

        // Clear mark queues
        self.global_queue.clear();

        // Scan roots and enqueue
        self.scan_roots()?;

        Ok(())
    }

    /// Scan all roots
    ///
    /// Called at the start of marking for initial work.
    /// Finds all live references from roots and enqueues them for marking.
    pub fn scan_roots(&self) -> Result<()> {
        // Scan all roots using the new root scanner
        let mut root_count = 0;

        self.root_scanner.scan_roots(|ref_value| {
            // Enqueue each live reference
            self.global_queue.push(ref_value);
            root_count += 1;
        });

        // Also scan current thread's stack for additional roots
        let stack_roots = self.scan_current_thread_stack();
        root_count += stack_roots;

        if self.root_scanner.get_stats().live_roots > 0 || stack_roots > 0 {
            self.marked_count
                .fetch_add(root_count as u64, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Scan current thread's stack for heap pointers
    ///
    /// This is called during root scanning to find references
    /// that are stored in the current thread's stack.
    fn scan_current_thread_stack(&self) -> usize {
        use crate::marker::stack_scan::StackScanner;

        let scanner = StackScanner::new();
        let heap = self.heap.clone();

        // Get heap range for validation
        let heap_base = heap.base_address();
        let heap_size = heap.max_size();
        let heap_range = (heap_base, heap_base + heap_size);

        // Scan current thread stack conservatively
        let pointers = scanner.scan_current_stack_conservative(heap_range);

        let mut count = 0;
        for ptr in pointers {
            if ptr != 0 {
                self.global_queue.push(ptr);
                count += 1;
            }
        }

        count
    }

    /// Concurrent marking loop
    ///
    /// Called by GC worker threads.
    /// Process mark queue until empty.
    pub fn concurrent_mark(
        &self,
        mut worker: crate::marker::mark_queue::MarkingWorker,
    ) -> Result<()> {
        while self.marking_in_progress.load(Ordering::Relaxed) {
            // Get work from local worker (includes stealing)
            if let Some(object) = worker.pop() {
                // Process object
                self.process_object(object, &mut worker)?;
            } else {
                // No work, check termination
                if self.should_terminate() {
                    break;
                }

                // Sleep briefly
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
        }

        Ok(())
    }

    /// Process single object
    ///
    /// Scan object fields and enqueue reachable references.
    fn process_object(
        &self,
        object: usize,
        worker: &mut crate::marker::mark_queue::MarkingWorker,
    ) -> Result<()> {
        // Check if already marked
        if self.is_marked(object) {
            return Ok(());
        }

        // Mark object
        self.mark_object(object)?;

        // Scan object fields and enqueue references using object scanner
        let _ref_count = scan_object(object, |ref_addr| {
            unsafe {
                let ref_value = crate::memory::read_pointer(ref_addr);
                if ref_value != 0 {
                    // Push to local worker queue for efficiency
                    worker.push(ref_value);
                }
            }
        });

        Ok(())
    }

    /// Mark object in bitmap
    fn mark_object(&self, object: usize) -> Result<()> {
        // Find region for object
        let regions = self.heap.get_active_regions();

        for region in regions {
            if object >= region.start() && object < region.end() {
                region.mark_object(object, 64); // Dummy size
                self.marked_count.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        }

        Ok(())
    }

    /// Check if object is already marked
    pub fn is_marked(&self, object: usize) -> bool {
        let regions = self.heap.get_active_regions();

        for region in regions {
            if object >= region.start() && object < region.end() {
                return region.is_marked(object);
            }
        }

        false
    }

    /// Wait until marking is complete
    pub fn wait_marking_complete(&self) -> Result<()> {
        // Wait until injector is empty and all threads are idle
        while !self.global_queue.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        Ok(())
    }

    /// Finalize marking
    pub fn finalize_marking(&self) -> Result<()> {
        self.marking_in_progress.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Shutdown marker
    pub fn shutdown(&self) -> Result<()> {
        self.marking_in_progress.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Check if marking should terminate
    fn should_terminate(&self) -> bool {
        // Note: Real termination detection would need a distributed quiescence algorithm
        // Simplified: injector is empty and no marking in progress
        self.global_queue.is_empty() && !self.marking_in_progress.load(Ordering::Relaxed)
    }

    /// Get marked count
    pub fn marked_count(&self) -> u64 {
        self.marked_count.load(Ordering::Relaxed)
    }

    /// Set thread count
    pub fn set_thread_count(&mut self, count: usize) {
        self.thread_count = count;
    }

    /// Get global queue reference
    pub fn get_global_queue(&self) -> Arc<MarkQueue> {
        self.global_queue.clone()
    }

    /// Check if marking is in progress
    pub fn is_marking_in_progress(&self) -> bool {
        self.marking_in_progress.load(Ordering::Relaxed)
    }

    /// Check if there is work in the mark queue
    pub fn has_mark_work(&self) -> bool {
        !self.global_queue.is_empty()
    }

    /// Process mark work (called by GC workers)
    ///
    /// This method is called by worker threads to process marking work.
    /// It scans the object and enqueues any references found.
    pub fn process_mark_work(&self, object: usize) -> Result<()> {
        // Check if already marked
        if self.is_marked(object) {
            return Ok(());
        }

        // Mark object
        self.mark_object(object)?;

        // Scan object fields and enqueue references using object scanner
        let ref_count = scan_object(object, |ref_addr| unsafe {
            let ref_value = crate::memory::read_pointer(ref_addr);
            if ref_value != 0 {
                self.global_queue.push(ref_value);
            }
        });

        // Update statistics
        self.marked_count
            .fetch_add(ref_count as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Start concurrent marking with worker threads
    ///
    /// This method starts the concurrent marking phase by spawning worker threads.
    ///
    /// # Arguments
    /// * `num_threads` - Number of worker threads for marking
    ///
    /// # Returns
    /// Result with OK if successfully started marking
    pub fn start_concurrent_marking(&self, num_threads: usize) -> Result<()> {
        // Start marking phase
        self.start_marking()?;

        // Create and start GC thread pool
        let marker_arc = Arc::new(self.clone_for_pool());
        let pool = GcThreadPool::new(num_threads, marker_arc, Arc::clone(&self.global_queue));

        // Start the pool before storing
        pool.start();

        // Store in mutex
        let mut pool_guard = self.gc_thread_pool.lock();
        *pool_guard = Some(Arc::new(pool));

        Ok(())
    }

    /// Wait for all worker threads to complete
    ///
    /// This method waits for all worker threads to finish marking.
    pub fn wait_completion(&self) -> Result<()> {
        // Wait for GC thread pool completion
        let pool_guard = self.gc_thread_pool.lock();
        if let Some(ref pool) = *pool_guard {
            pool.wait_completion()?;
        } else {
            // Fallback to single-threaded wait
            self.wait_marking_complete()?;
        }

        Ok(())
    }

    /// Get GC pool statistics
    pub fn get_pool_stats(&self) -> Option<GcPoolStats> {
        let pool_guard = self.gc_thread_pool.lock();
        pool_guard.as_ref().map(|pool| pool.stats())
    }

    /// Clone marker for thread pool (creates Arc-compatible version)
    fn clone_for_pool(&self) -> Marker {
        let bitmaps_clone = self
            .bitmaps
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| IndexMap::new());

        Marker {
            heap: self.heap.clone(),
            global_queue: self.global_queue.clone(),
            bitmaps: std::sync::Mutex::new(bitmaps_clone),
            root_scanner: self.root_scanner.clone(),
            current_mark_bit: AtomicBool::new(self.current_mark_bit.load(Ordering::Relaxed)),
            marking_in_progress: AtomicBool::new(self.marking_in_progress.load(Ordering::Relaxed)),
            marked_count: AtomicU64::new(self.marked_count.load(Ordering::Relaxed)),
            thread_count: self.thread_count,
            gc_thread_pool: ParkingMutex::new(None),
        }
    }

    /// Get reference to root scanner
    pub fn root_scanner(&self) -> &RootScanner {
        &self.root_scanner
    }
}

impl Clone for Marker {
    fn clone(&self) -> Self {
        let bitmaps_clone = self
            .bitmaps
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| IndexMap::new());

        Self {
            heap: self.heap.clone(),
            global_queue: self.global_queue.clone(),
            bitmaps: std::sync::Mutex::new(bitmaps_clone),
            root_scanner: self.root_scanner.clone(),
            current_mark_bit: AtomicBool::new(self.current_mark_bit.load(Ordering::Relaxed)),
            marking_in_progress: AtomicBool::new(self.marking_in_progress.load(Ordering::Relaxed)),
            marked_count: AtomicU64::new(self.marked_count.load(Ordering::Relaxed)),
            thread_count: self.thread_count,
            gc_thread_pool: ParkingMutex::new(None),
        }
    }
}
