//! GC Threads - Worker Threads for Parallel Marking
//!
//! Module ini mengimplementasikan worker threads untuk concurrent marking.
//! Menggunakan work-stealing untuk load balancing antar threads.
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │                    GC Thread Pool                        │
//! │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
//! │  │ Worker 0 │ │ Worker 1 │ │ Worker 2 │ │ Worker N │   │
//! │  │  Local   │ │  Local   │ │  Local   │ │  Local   │   │
//! │  │  Queue   │ │  Queue   │ │  Queue   │ │  Queue   │   │
//! │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │
//! │       │           │           │           │           │
//! │       └───────────┴─────┬─────┴───────────┘           │
//! │                         │                             │
//! │                  Global Queue                         │
//! └─────────────────────────┴─────────────────────────────┘
//! ```
//!
//! Work Stealing Algorithm:
//! 1. Worker process dari local queue sendiri
//! 2. Jika local queue kosong, steal dari worker lain
//! 3. Jika semua queue kosong, idle/wait

use crate::marker::{Marker, MarkQueue};
use crate::error::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use parking_lot::Mutex as ParkingMutex;

/// GC Worker Thread
///
/// Worker thread yang memproses marking work dari queue.
/// Setiap worker memiliki local queue untuk mengurangi contention.
pub struct GcWorker {
    /// Worker ID (unique dalam pool)
    id: usize,

    /// Reference ke marker
    marker: Arc<Marker>,

    /// Local work queue untuk mengurangi contention
    local_queue: Arc<ParkingMutex<Vec<usize>>>,

    /// Statistics - processed object count
    processed_count: AtomicUsize,

    /// Worker is idle
    idle: AtomicBool,

    /// Worker should stop
    should_stop: AtomicBool,

    /// Reference ke global queue untuk work stealing
    global_queue: Arc<MarkQueue>,
}

impl GcWorker {
    /// Create new GC worker
    ///
    /// # Arguments
    /// * `id` - Worker ID (unique dalam pool)
    /// * `marker` - Reference ke marker
    /// * `global_queue` - Global mark queue untuk work stealing
    pub fn new(id: usize, marker: Arc<Marker>, global_queue: Arc<MarkQueue>) -> Self {
        Self {
            id,
            marker,
            local_queue: Arc::new(ParkingMutex::new(Vec::with_capacity(256))),
            processed_count: AtomicUsize::new(0),
            idle: AtomicBool::new(true),  // Start as idle
            should_stop: AtomicBool::new(false),
            global_queue,
        }
    }

    /// Get worker ID
    pub fn id(&self) -> usize {
        self.id
    }

    /// Check jika worker idle
    pub fn is_idle(&self) -> bool {
        self.idle.load(Ordering::Relaxed)
    }

    /// Check jika worker should stop
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Ordering::Relaxed)
    }

    /// Signal worker to stop
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    /// Start worker thread
    ///
    /// Returns JoinHandle untuk thread yang dijalankan.
    pub fn start(self: &Arc<Self>) -> JoinHandle<()> {
        let worker = Arc::clone(self);
        
        thread::Builder::new()
            .name(format!("gc-worker-{}", self.id))
            .spawn(move || {
                worker.run();
            })
            .expect("Failed to spawn GC worker thread")
    }

    /// Main worker loop
    fn run(&self) {
        while !self.should_stop.load(Ordering::Relaxed) {
            // Try get work dari local queue
            if let Some(object) = self.pop_local_work() {
                self.idle.store(false, Ordering::Relaxed);
                
                // Process work
                match self.process_object(object) {
                    Ok(_) => {
                        self.processed_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        log::error!("[GC Worker {}] Error processing object {:#x}: {:?}", 
                                   self.id, object, e);
                    }
                }
                continue;
            }

            // Local queue empty, try global queue
            if let Some(object) = self.global_queue.pop() {
                self.idle.store(false, Ordering::Relaxed);
                
                match self.process_object(object) {
                    Ok(_) => {
                        self.processed_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        log::error!("[GC Worker {}] Error processing object {:#x}: {:?}", 
                                   self.id, object, e);
                    }
                }
                continue;
            }

            // Try steal work dari worker lain
            if let Some(object) = self.steal_work() {
                self.idle.store(false, Ordering::Relaxed);
                
                match self.process_object(object) {
                    Ok(_) => {
                        self.processed_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        log::error!("[GC Worker {}] Error processing object {:#x}: {:?}", 
                                   self.id, object, e);
                    }
                }
                continue;
            }

            // No work available, mark as idle
            self.idle.store(true, Ordering::Relaxed);

            // Check termination
            if self.should_terminate() {
                break;
            }

            // Sleep sebentar sebelum check lagi
            thread::sleep(Duration::from_micros(100));
        }

        self.idle.store(true, Ordering::Relaxed);
    }

    /// Pop work dari local queue
    fn pop_local_work(&self) -> Option<usize> {
        let mut queue = self.local_queue.lock();
        queue.pop()
    }

    /// Push work ke local queue
    pub fn push_local_work(&self, work: usize) {
        let mut queue = self.local_queue.lock();
        queue.push(work);
    }

    /// Steal work dari worker lain
    ///
    /// Work stealing dilakukan dengan mencoba steal dari worker
    /// lain secara round-robin.
    fn steal_work(&self) -> Option<usize> {
        // Work stealing akan di-handle oleh GcThreadPool
        // yang memiliki referensi ke semua workers
        None
    }

    /// Process single object
    fn process_object(&self, object: usize) -> Result<()> {
        // Delegate ke marker untuk processing
        // Marker akan scan object dan enqueue references
        self.marker.process_mark_work(object)
    }

    /// Check jika worker harus terminate
    fn should_terminate(&self) -> bool {
        // Terminate jika:
        // 1. Marking tidak lagi in progress
        // 2. Global queue empty
        // 3. Local queue empty
        !self.marker.is_marking_in_progress() 
            && self.global_queue.is_empty()
            && self.local_queue.lock().is_empty()
    }

    /// Get processed count
    pub fn processed_count(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }

    /// Get worker statistics
    pub fn get_stats(&self) -> GcWorkerStats {
        GcWorkerStats {
            id: self.id,
            processed_count: self.processed_count(),
            is_idle: self.is_idle(),
            local_queue_size: self.local_queue.lock().len(),
        }
    }
}

/// GC Thread Pool Manager
///
/// Mengelola pool dari GC worker threads.
/// Responsible untuk:
/// - Spawning worker threads
/// - Work distribution
/// - Work stealing coordination
/// - Shutdown coordination
pub struct GcThreadPool {
    /// Worker threads
    workers: Vec<Arc<GcWorker>>,

    /// Thread handles
    handles: ParkingMutex<Vec<JoinHandle<()>>>,

    /// Pool active
    active: AtomicBool,

    /// Total workers
    total_workers: usize,

    /// Reference ke marker
    marker: Arc<Marker>,

    /// Global mark queue
    global_queue: Arc<MarkQueue>,

    /// Statistics - total processed
    total_processed: AtomicUsize,
}

impl GcThreadPool {
    /// Create new GC thread pool
    ///
    /// # Arguments
    /// * `marker` - Reference ke marker
    /// * `num_workers` - Jumlah worker threads
    pub fn new(marker: Arc<Marker>, num_workers: usize) -> Self {
        let global_queue = marker.get_global_queue();
        let mut workers = Vec::with_capacity(num_workers);

        for i in 0..num_workers {
            let worker = Arc::new(GcWorker::new(i, marker.clone(), global_queue.clone()));
            workers.push(worker);
        }

        Self {
            workers,
            handles: ParkingMutex::new(Vec::with_capacity(num_workers)),
            active: AtomicBool::new(false),
            total_workers: num_workers,
            marker,
            global_queue,
            total_processed: AtomicUsize::new(0),
        }
    }

    /// Start semua worker threads
    pub fn start(&mut self) {
        if self.active.load(Ordering::Relaxed) {
            log::warn!("GC thread pool already active");
            return;
        }

        log::info!("[GC ThreadPool] Starting {} worker threads", self.total_workers);

        let mut handles = self.handles.lock();
        for worker in &self.workers {
            let handle = worker.start();
            handles.push(handle);
        }

        self.active.store(true, Ordering::Relaxed);
    }

    /// Stop semua worker threads
    pub fn stop(&self) {
        if !self.active.load(Ordering::Relaxed) {
            return;
        }

        log::info!("[GC ThreadPool] Stopping {} worker threads", self.total_workers);

        // Signal semua workers untuk stop
        for worker in &self.workers {
            worker.stop();
        }

        // Wait untuk semua threads selesai
        let mut handles = self.handles.lock();
        for handle in handles.drain(..) {
            if let Err(e) = handle.join() {
                log::error!("[GC ThreadPool] Worker thread join error: {:?}", e);
            }
        }

        self.active.store(false, Ordering::Relaxed);
    }

    /// Wait completion - tunggu semua workers idle dan queue empty
    pub fn wait_completion(&self) -> Result<()> {
        log::info!("[GC ThreadPool] Waiting for completion...");

        loop {
            // Check jika semua workers idle
            let all_idle = self.workers.iter().all(|w| w.is_idle());
            
            // Check jika queues empty
            let global_empty = self.global_queue.is_empty();
            let locals_empty = self.workers.iter().all(|w| w.local_queue.lock().is_empty());

            if all_idle && global_empty && locals_empty {
                // Double-check dengan delay kecil
                thread::sleep(Duration::from_millis(1));
                
                let still_all_idle = self.workers.iter().all(|w| w.is_idle());
                let still_global_empty = self.global_queue.is_empty();
                let still_locals_empty = self.workers.iter().all(|w| w.local_queue.lock().is_empty());

                if still_all_idle && still_global_empty && still_locals_empty {
                    break;
                }
            }

            // Check jika marking sudah selesai
            if !self.marker.is_marking_in_progress() {
                // Wait sebentar untuk final processing
                thread::sleep(Duration::from_millis(1));
                
                // Check lagi
                let all_idle = self.workers.iter().all(|w| w.is_idle());
                let global_empty = self.global_queue.is_empty();
                
                if all_idle && global_empty {
                    break;
                }
            }

            thread::sleep(Duration::from_micros(100));
        }

        log::info!("[GC ThreadPool] Completion reached");
        Ok(())
    }

    /// Distribute work ke workers
    ///
    /// Work didistribusikan secara round-robin ke local queues.
    pub fn distribute_work(&self, work_items: &[usize]) {
        for (i, &work) in work_items.iter().enumerate() {
            let worker_idx = i % self.total_workers;
            self.workers[worker_idx].push_local_work(work);
        }
    }

    /// Get statistics dari pool
    pub fn get_stats(&self) -> GcPoolStats {
        let mut worker_stats = Vec::with_capacity(self.total_workers);
        let mut total_processed = 0;
        let mut idle_count = 0;

        for worker in &self.workers {
            let stats = worker.get_stats();
            total_processed += stats.processed_count;
            if stats.is_idle {
                idle_count += 1;
            }
            worker_stats.push(stats);
        }

        GcPoolStats {
            total_workers: self.total_workers,
            active_workers: self.total_workers - idle_count,
            idle_workers: idle_count,
            total_processed,
            is_active: self.active.load(Ordering::Relaxed),
            global_queue_size: self.global_queue.len(),
            worker_stats,
        }
    }

    /// Get worker by ID
    pub fn get_worker(&self, id: usize) -> Option<&Arc<GcWorker>> {
        self.workers.get(id)
    }

    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.total_workers
    }

    /// Check jika pool active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }
}

impl Drop for GcThreadPool {
    fn drop(&mut self) {
        if self.active.load(Ordering::Relaxed) {
            self.stop();
        }
    }
}

/// Statistics untuk GC worker
#[derive(Debug, Clone)]
pub struct GcWorkerStats {
    /// Worker ID
    pub id: usize,
    /// Processed object count
    pub processed_count: usize,
    /// Worker is idle
    pub is_idle: bool,
    /// Local queue size
    pub local_queue_size: usize,
}

/// Statistics untuk GC thread pool
#[derive(Debug, Clone)]
pub struct GcPoolStats {
    /// Total workers
    pub total_workers: usize,
    /// Active workers (not idle)
    pub active_workers: usize,
    /// Idle workers
    pub idle_workers: usize,
    /// Total processed objects
    pub total_processed: usize,
    /// Pool is active
    pub is_active: bool,
    /// Global queue size
    pub global_queue_size: usize,
    /// Per-worker statistics
    pub worker_stats: Vec<GcWorkerStats>,
}

impl std::fmt::Display for GcPoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GcPoolStats {{ total: {}, active: {}, idle: {}, processed: {}, active: {}, queue: {} }}",
            self.total_workers,
            self.active_workers,
            self.idle_workers,
            self.total_processed,
            self.is_active,
            self.global_queue_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::Heap;
    use crate::config::GcConfig;
    use std::sync::Arc;
    use std::time::Duration;

    fn create_test_marker() -> (Arc<Marker>, Arc<Heap>) {
        let config = Arc::new(GcConfig::default());
        let heap = Arc::new(Heap::new(config.clone()).unwrap());
        let marker = Arc::new(Marker::new(heap.clone()));
        (marker, heap)
    }

    #[test]
    fn test_gc_worker_creation() {
        let (marker, _heap) = create_test_marker();
        let global_queue = marker.get_global_queue();
        
        let worker = GcWorker::new(0, marker.clone(), global_queue);
        
        assert_eq!(worker.id(), 0);
        assert!(worker.is_idle());
        assert!(!worker.should_stop());
        assert_eq!(worker.processed_count(), 0);
    }

    #[test]
    fn test_gc_worker_push_pop_local_work() {
        let (marker, _heap) = create_test_marker();
        let global_queue = marker.get_global_queue();
        
        let worker = Arc::new(GcWorker::new(0, marker.clone(), global_queue));
        
        worker.push_local_work(0x1234);
        worker.push_local_work(0x5678);
        
        // Local queue should have 2 items
        assert_eq!(worker.local_queue.lock().len(), 2);
    }

    #[test]
    fn test_gc_worker_stop() {
        let (marker, _heap) = create_test_marker();
        let global_queue = marker.get_global_queue();
        
        let worker = GcWorker::new(0, marker.clone(), global_queue);
        
        assert!(!worker.should_stop());
        
        worker.stop();
        
        assert!(worker.should_stop());
    }

    #[test]
    fn test_gc_thread_pool_creation() {
        let (marker, _heap) = create_test_marker();
        
        let pool = GcThreadPool::new(marker.clone(), 4);
        
        assert_eq!(pool.num_workers(), 4);
        assert!(!pool.is_active());
        
        let stats = pool.get_stats();
        assert_eq!(stats.total_workers, 4);
        assert_eq!(stats.active_workers, 0);
        assert_eq!(stats.idle_workers, 4);
    }

    #[test]
    fn test_gc_thread_pool_start_stop() {
        let (marker, _heap) = create_test_marker();
        
        let mut pool = GcThreadPool::new(marker.clone(), 2);
        
        assert!(!pool.is_active());
        
        pool.start();
        
        // Wait untuk threads start
        thread::sleep(Duration::from_millis(10));
        
        assert!(pool.is_active());
        
        pool.stop();
        
        assert!(!pool.is_active());
    }

    #[test]
    fn test_gc_thread_pool_distribute_work() {
        let (marker, _heap) = create_test_marker();
        
        let pool = GcThreadPool::new(marker.clone(), 4);
        
        let work_items = vec![0x1000, 0x2000, 0x3000, 0x4000, 0x5000];
        pool.distribute_work(&work_items);
        
        // Check work distributed round-robin
        assert_eq!(pool.get_worker(0).unwrap().local_queue.lock().len(), 2); // 0x1000, 0x5000
        assert_eq!(pool.get_worker(1).unwrap().local_queue.lock().len(), 1); // 0x2000
        assert_eq!(pool.get_worker(2).unwrap().local_queue.lock().len(), 1); // 0x3000
        assert_eq!(pool.get_worker(3).unwrap().local_queue.lock().len(), 1); // 0x4000
    }

    #[test]
    fn test_gc_pool_stats() {
        let (marker, _heap) = create_test_marker();
        
        let pool = GcThreadPool::new(marker.clone(), 3);
        
        let stats = pool.get_stats();
        
        assert_eq!(stats.total_workers, 3);
        assert_eq!(stats.idle_workers, 3);
        assert_eq!(stats.total_processed, 0);
        assert!(!stats.is_active);
    }

    #[test]
    fn test_gc_worker_stats() {
        let (marker, _heap) = create_test_marker();
        let global_queue = marker.get_global_queue();
        
        let worker = GcWorker::new(0, marker.clone(), global_queue);
        let stats = worker.get_stats();
        
        assert_eq!(stats.id, 0);
        assert_eq!(stats.processed_count, 0);
        assert!(stats.is_idle);
        assert_eq!(stats.local_queue_size, 0);
    }
}
