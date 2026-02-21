//! GC Threads - Worker Threads for Parallel Marking
//!
//! This module implements worker threads for concurrent marking.
//! Uses work-stealing for load balancing between threads.

use crate::marker::Marker;
use crate::marker::mark_queue::MarkQueue;
use crate::error::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// GC Worker Thread
pub struct GcWorker {
    /// Worker ID
    id: usize,
    /// Reference to marker
    marker: Arc<Marker>,
    /// Statistics
    processed_count: AtomicUsize,
    /// Worker is idle
    idle: AtomicBool,
    /// Worker should stop
    should_stop: AtomicBool,
    /// Reference to global queue
    global_queue: Arc<MarkQueue>,
}

impl GcWorker {
    pub fn new(id: usize, marker: Arc<Marker>, global_queue: Arc<MarkQueue>) -> Self {
        Self {
            id,
            marker,
            processed_count: AtomicUsize::new(0),
            idle: AtomicBool::new(true),
            should_stop: AtomicBool::new(false),
            global_queue,
        }
    }

    pub fn start(self: &Arc<Self>) -> JoinHandle<()> {
        let worker = Arc::clone(self);
        let global_queue = Arc::clone(&self.global_queue);

        thread::Builder::new()
            .name(format!("gc-worker-{}", self.id))
            .spawn(move || {
                // Create local work-stealing worker
                let mut marking_worker = global_queue.create_worker();
                worker.run(&mut marking_worker);
            })
            .expect("Failed to spawn GC worker thread")
    }

    fn run(&self, marking_worker: &mut crate::marker::mark_queue::MarkingWorker) {
        while !self.should_stop.load(Ordering::Relaxed) {
            // MarkingWorker::pop handles local pop, global injector pop, and stealing
            if let Some(object) = marking_worker.pop() {
                self.idle.store(false, Ordering::Relaxed);

                // marker.concurrent_mark logic integrated here
                match self.marker.process_object(object, marking_worker) {
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

            // No work found
            self.idle.store(true, Ordering::Relaxed);

            // Brief sleep to reduce CPU usage when idle
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    pub fn is_idle(&self) -> bool {
        self.idle.load(Ordering::Relaxed)
    }

    pub fn processed_count(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }
}

/// GC Thread Pool
pub struct GcThreadPool {
    workers: Vec<Arc<GcWorker>>,
    handles: std::sync::Mutex<Vec<JoinHandle<()>>>,
    /// Whether the pool is currently active (threads running)
    active: AtomicBool,
}

impl GcThreadPool {
    pub fn new(count: usize, marker: Arc<Marker>, global_queue: Arc<MarkQueue>) -> Self {
        let mut workers = Vec::with_capacity(count);
        for i in 0..count {
            workers.push(Arc::new(GcWorker::new(i, Arc::clone(&marker), Arc::clone(&global_queue))));
        }

        Self {
            workers,
            handles: std::sync::Mutex::new(Vec::new()),
            active: AtomicBool::new(false),
        }
    }

    /// Get number of workers in the pool
    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }

    /// Check if the pool is active (threads are running)
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn start(&self) {
        let mut handles = self.handles.lock().unwrap();
        for worker in &self.workers {
            handles.push(worker.start());
        }
        self.active.store(true, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        for worker in &self.workers {
            worker.stop();
        }
        self.active.store(false, Ordering::Relaxed);
    }

    pub fn join(&self) {
        let mut handles = self.handles.lock().unwrap();
        for handle in handles.drain(..) {
            let _ = handle.join();
        }
        self.active.store(false, Ordering::Relaxed);
    }

    pub fn wait_completion(&self) -> Result<()> {
        self.join();
        Ok(())
    }

    pub fn is_all_idle(&self) -> bool {
        self.workers.iter().all(|w| worker_is_actually_idle(w))
    }

    /// Get pool statistics
    pub fn stats(&self) -> GcPoolStats {
        let total_processed = self.workers.iter().map(|w| w.processed_count()).sum();
        let idle_count = self.workers.iter().filter(|w| w.is_idle()).count();
        
        let mut worker_stats = Vec::new();
        for worker in &self.workers {
            worker_stats.push(WorkerStats {
                processed_count: worker.processed_count(),
                is_idle: worker.is_idle(),
                local_queue_size: 0, // Simplified - no local queue tracking
            });
        }

        GcPoolStats {
            total_workers: self.workers.len(),
            idle_workers: idle_count,
            total_processed,
            is_active: self.active.load(Ordering::Relaxed),
            worker_stats,
        }
    }

    /// Alias for stats() for API compatibility
    pub fn get_stats(&self) -> GcPoolStats {
        self.stats()
    }

    /// Distribute work items to workers (round-robin)
    /// 
    /// Note: This is a simplified implementation for testing.
    /// In a real implementation, work would be pushed to local queues.
    pub fn distribute_work(&self, work_items: &[usize]) {
        let global_queue = self.workers[0].global_queue.clone();
        for &item in work_items {
            global_queue.push(item);
        }
    }
}

fn worker_is_actually_idle(worker: &GcWorker) -> bool {
    worker.is_idle()
}

/// Statistics for individual worker
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub processed_count: usize,
    pub is_idle: bool,
    pub local_queue_size: usize,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct GcPoolStats {
    pub total_workers: usize,
    pub idle_workers: usize,
    pub total_processed: usize,
    pub is_active: bool,
    pub worker_stats: Vec<WorkerStats>,
}
