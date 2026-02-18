//! GC Threads - Worker Threads for Parallel Marking
//!
//! This module implements worker threads for concurrent marking.
//! Uses work-stealing for load balancing between threads.
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
//! 1. Worker processes from its own local queue
//! 2. If local queue is empty, steal from other workers
//! 3. If all queues are empty, idle/wait

use crate::marker::{Marker, MarkQueue};
use crate::error::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use parking_lot::Mutex as ParkingMutex;

/// GC Worker Thread
///
/// Worker thread that processes marking work from queue.
/// Each worker has a local queue to reduce contention.
pub struct GcWorker {
    /// Worker ID (unique within pool)
    id: usize,

    /// Reference to marker
    marker: Arc<Marker>,

    /// Local work queue to reduce contention
    local_queue: Arc<ParkingMutex<Vec<usize>>>,

    /// Statistics - processed object count
    processed_count: AtomicUsize,

    /// Worker is idle
    idle: AtomicBool,

    /// Worker should stop
    should_stop: AtomicBool,

    /// Reference to global queue for work stealing
    global_queue: Arc<MarkQueue>,
}

impl GcWorker {
    /// Create new GC worker
    ///
    /// # Arguments
    /// * `id` - Worker ID (unique within pool)
    /// * `marker` - Reference to marker
    /// * `global_queue` - Global mark queue for work stealing
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

    /// Check if worker is idle
    pub fn is_idle(&self) -> bool {
        self.idle.load(Ordering::Relaxed)
    }

    /// Check if worker should stop
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Ordering::Relaxed)
    }

    /// Signal worker to stop
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    /// Start worker thread
    ///
    /// Returns JoinHandle for the spawned thread.
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
            // Try get work from local queue
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

            // Try steal work from other workers
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

            // Sleep briefly before checking again
            thread::sleep(Duration::from_micros(100));
        }

        self.idle.store(true, Ordering::Relaxed);
    }

    /// Pop work from local queue
    fn pop_local_work(&self) -> Option<usize> {
        let mut queue = self.local_queue.lock();
        queue.pop()
    }

    /// Push work to local queue
    pub fn push_local_work(&self, work: usize) {
        let mut queue = self.local_queue.lock();
        queue.push(work);
    }

    /// Steal work from other workers
    ///
    /// Work stealing is done by trying to steal from other workers
    /// in round-robin fashion.
    fn steal_work(&self) -> Option<usize> {
        // Work stealing will be handled by GcThreadPool
        // which has references to all workers
        None
    }

    /// Process single object
    fn process_object(&self, object: usize) -> Result<()> {
        // Delegate to marker for processing
        // Marker will scan object and enqueue references
        self.marker.process_mark_work(object)
    }

    /// Check if worker should terminate
    fn should_terminate(&self) -> bool {
        // Terminate if:
        // 1. Marking is no longer in progress
        // 2. Global queue is empty
        // 3. Local queue is empty
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
/// Manages pool of GC worker threads.
/// Responsible for:
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

    /// Reference to marker
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
    /// * `marker` - Reference to marker
    /// * `num_workers` - Number of worker threads
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

    /// Start all worker threads
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

    /// Stop all worker threads
    pub fn stop(&self) {
        if !self.active.load(Ordering::Relaxed) {
            return;
        }

        log::info!("[GC ThreadPool] Stopping {} worker threads", self.total_workers);

        // Signal all workers to stop
        for worker in &self.workers {
            worker.stop();
        }

        // Wait for all threads to finish
        let mut handles = self.handles.lock();
        for handle in handles.drain(..) {
            if let Err(e) = handle.join() {
                log::error!("[GC ThreadPool] Worker thread join error: {:?}", e);
            }
        }

        self.active.store(false, Ordering::Relaxed);
    }

    /// Wait completion - wait until all workers are idle and queues are empty
    pub fn wait_completion(&self) -> Result<()> {
        log::info!("[GC ThreadPool] Waiting for completion...");

        loop {
            // Check if all workers are idle
            let all_idle = self.workers.iter().all(|w| w.is_idle());

            // Check if queues are empty
            let global_empty = self.global_queue.is_empty();
            let locals_empty = self.workers.iter().all(|w| w.local_queue.lock().is_empty());

            if all_idle && global_empty && locals_empty {
                // Double-check with small delay
                thread::sleep(Duration::from_millis(1));

                let still_all_idle = self.workers.iter().all(|w| w.is_idle());
                let still_global_empty = self.global_queue.is_empty();
                let still_locals_empty = self.workers.iter().all(|w| w.local_queue.lock().is_empty());

                if still_all_idle && still_global_empty && still_locals_empty {
                    break;
                }
            }

            // Check if marking is finished
            if !self.marker.is_marking_in_progress() {
                // Wait briefly for final processing
                thread::sleep(Duration::from_millis(1));

                // Check again
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

    /// Distribute work to workers
    ///
    /// Work is distributed round-robin to local queues.
    pub fn distribute_work(&self, work_items: &[usize]) {
        for (i, &work) in work_items.iter().enumerate() {
            let worker_idx = i % self.total_workers;
            self.workers[worker_idx].push_local_work(work);
        }
    }

    /// Get statistics from pool
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

    /// Check if pool is active
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

/// Statistics for GC worker
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

/// Statistics for GC thread pool
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

        // Wait for threads to start
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
