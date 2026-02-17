//! Marker Module - Concurrent Marking System
//!
//! Module ini mengimplementasikan concurrent marking untuk mengidentifikasi
//! object mana yang masih "hidup" (direference) dan mana yang garbage.
//!
//! Marking Algorithm:
//! - Tri-color marking (White, Grey, Black)
//! - Concurrent dengan aplikasi berjalan
//! - Load barriers mark objects saat diakses
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

pub mod mark_queue;
pub mod bitmap;
pub mod roots;
pub mod stack_scan;
pub mod object_scanner;
pub mod gc_threads;

pub use mark_queue::MarkQueue;
pub use bitmap::MarkBitmap;
pub use roots::{RootScanner, RootType, RootDescriptor, RootHandle, RootStats};
pub use object_scanner::{ObjectScanStats, scan_object, scan_object_precise, scan_object_conservative};
pub use gc_threads::{GcThreadPool, GcPoolStats, GcWorker, GcWorkerStats};

use crate::error::Result;
use crate::heap::Heap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::Mutex as ParkingMutex;

/// Marker - orchestrator untuk concurrent marking
///
/// Marker mengelola seluruh proses marking:
/// - Root scanning
/// - Mark queue management
/// - GC thread coordination
/// - Termination detection
pub struct Marker {
    /// Reference ke heap
    heap: Arc<Heap>,

    /// Global mark queue
    global_queue: Arc<MarkQueue>,

    /// Mark bitmaps per region
    bitmaps: std::sync::Mutex<std::collections::HashMap<usize, MarkBitmap>>,

    /// Root scanner
    root_scanner: RootScanner,

    /// Current mark bit (false = Marked0, true = Marked1)
    current_mark_bit: AtomicBool,

    /// Marking in progress
    marking_in_progress: AtomicBool,

    /// Marked object count
    marked_count: AtomicU64,

    /// GC threads untuk marking
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
            bitmaps: std::sync::Mutex::new(std::collections::HashMap::new()),
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
    /// Dipanggil saat GC cycle memasuki marking phase.
    /// Setup state dan spawn GC threads.
    pub fn start_marking(&self) -> Result<()> {
        self.marking_in_progress.store(true, Ordering::SeqCst);
        self.marked_count.store(0, Ordering::Relaxed);

        // Clear mark queues
        self.global_queue.clear();

        // Scan roots dan enqueue
        self.scan_roots()?;

        Ok(())
    }

    /// Scan semua roots
    ///
    /// Dipanggil di awal marking untuk initial work.
    /// Menemukan semua live references dari roots dan enqueue untuk marking.
    pub fn scan_roots(&self) -> Result<()> {
        // Scan all roots using the new root scanner
        let mut root_count = 0;

        self.root_scanner.scan_roots(|ref_value| {
            // Enqueue each live reference
            self.global_queue.push(ref_value);
            root_count += 1;
        });

        if self.root_scanner.get_stats().live_roots > 0 {
            self.marked_count.fetch_add(root_count as u64, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Concurrent marking loop
    ///
    /// Dipanggil oleh GC worker threads.
    /// Process mark queue sampai empty.
    pub fn concurrent_mark(&self) -> Result<()> {
        while self.marking_in_progress.load(Ordering::Relaxed) {
            // Get work dari queue
            if let Some(object) = self.global_queue.pop() {
                // Process object
                self.process_object(object)?;
            } else {
                // No work, check termination
                if self.should_terminate() {
                    break;
                }

                // Sleep sebentar
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
        }

        Ok(())
    }

    /// Process single object
    ///
    /// Scan fields object dan enqueue reachable references.
    fn process_object(&self, object: usize) -> Result<()> {
        // Check jika sudah marked
        if self.is_marked(object) {
            return Ok(());
        }

        // Mark object
        self.mark_object(object)?;

        // Scan object fields dan enqueue references menggunakan object scanner
        let ref_count = scan_object(object, |ref_addr| {
            unsafe {
                let ref_value = crate::memory::read_pointer(ref_addr);
                if ref_value != 0 {
                    self.global_queue.push(ref_value);
                }
            }
        });

        // Update statistics
        self.marked_count.fetch_add(ref_count as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Mark object di bitmap
    fn mark_object(&self, object: usize) -> Result<()> {
        // Find region untuk object
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

    /// Check jika object sudah marked
    pub fn is_marked(&self, object: usize) -> bool {
        let regions = self.heap.get_active_regions();

        for region in regions {
            if object >= region.start() && object < region.end() {
                return region.is_marked(object);
            }
        }

        false
    }

    /// Wait sampai marking complete
    pub fn wait_marking_complete(&self) -> Result<()> {
        // Wait sampai queue empty dan semua threads idle
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

    /// Check jika marking harus terminate
    fn should_terminate(&self) -> bool {
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

    /// Check jika marking in progress
    pub fn is_marking_in_progress(&self) -> bool {
        self.marking_in_progress.load(Ordering::Relaxed)
    }

    /// Process mark work (called by GC workers)
    ///
    /// This method is called by worker threads to process marking work.
    /// It scans the object and enqueues any references found.
    pub fn process_mark_work(&self, object: usize) -> Result<()> {
        // Check jika sudah marked
        if self.is_marked(object) {
            return Ok(());
        }

        // Mark object
        self.mark_object(object)?;

        // Scan object fields dan enqueue references menggunakan object scanner
        let ref_count = scan_object(object, |ref_addr| {
            unsafe {
                let ref_value = crate::memory::read_pointer(ref_addr);
                if ref_value != 0 {
                    self.global_queue.push(ref_value);
                }
            }
        });

        // Update statistics
        self.marked_count.fetch_add(ref_count as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Start concurrent marking dengan worker threads
    ///
    /// Method ini memulai concurrent marking phase dengan spawn worker threads.
    ///
    /// # Arguments
    /// * `num_threads` - Jumlah worker threads untuk marking
    ///
    /// # Returns
    /// Result dengan OK jika berhasil start marking
    pub fn start_concurrent_marking(&self, num_threads: usize) -> Result<()> {
        // Start marking phase
        self.start_marking()?;

        // Create dan start GC thread pool
        let marker_arc = Arc::new(self.clone_for_pool());
        let mut pool = GcThreadPool::new(marker_arc, num_threads);
        
        // Start the pool before storing
        pool.start();
        
        // Store in mutex
        let mut pool_guard = self.gc_thread_pool.lock();
        *pool_guard = Some(Arc::new(pool));

        Ok(())
    }

    /// Wait untuk semua worker threads selesai
    ///
    /// Method ini menunggu semua worker threads menyelesaikan marking.
    pub fn wait_completion(&self) -> Result<()> {
        // Wait untuk GC thread pool completion
        let pool_guard = self.gc_thread_pool.lock();
        if let Some(ref pool) = *pool_guard {
            pool.wait_completion()?;
        } else {
            // Fallback ke single-threaded wait
            self.wait_marking_complete()?;
        }

        Ok(())
    }

    /// Get GC pool statistics
    pub fn get_pool_stats(&self) -> Option<GcPoolStats> {
        let pool_guard = self.gc_thread_pool.lock();
        pool_guard.as_ref().map(|pool| pool.get_stats())
    }

    /// Clone marker untuk thread pool (creates Arc-compatible version)
    fn clone_for_pool(&self) -> Marker {
        Marker {
            heap: self.heap.clone(),
            global_queue: self.global_queue.clone(),
            bitmaps: std::sync::Mutex::new(self.bitmaps.lock().unwrap().clone()),
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
        Self {
            heap: self.heap.clone(),
            global_queue: self.global_queue.clone(),
            bitmaps: std::sync::Mutex::new(self.bitmaps.lock().unwrap().clone()),
            root_scanner: self.root_scanner.clone(),
            current_mark_bit: AtomicBool::new(self.current_mark_bit.load(Ordering::Relaxed)),
            marking_in_progress: AtomicBool::new(self.marking_in_progress.load(Ordering::Relaxed)),
            marked_count: AtomicU64::new(self.marked_count.load(Ordering::Relaxed)),
            thread_count: self.thread_count,
            gc_thread_pool: ParkingMutex::new(None),
        }
    }
}
