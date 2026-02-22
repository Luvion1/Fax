//! Mark Queue - Concurrent Work-Stealing Queue for Marking
//!
//! Implementation using crossbeam-deque for high-performance load balancing.

use crossbeam_deque::{Injector, Stealer, Worker};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// MarkQueue - Orchestrator for work-stealing marking tasks
pub struct MarkQueue {
    /// Global injector for work from mutators or initial roots
    injector: Injector<usize>,

    /// Stealers from all workers
    stealers: Arc<parking_lot::RwLock<Vec<Stealer<usize>>>>,

    /// Counter for statistics
    enqueued_count: AtomicUsize,
    processed_count: AtomicUsize,

    /// Queue is closed
    closed: AtomicBool,
}

impl MarkQueue {
    /// Create a new work-stealing mark queue
    pub fn new() -> Self {
        Self {
            injector: Injector::new(),
            stealers: Arc::new(parking_lot::RwLock::new(Vec::new())),
            enqueued_count: AtomicUsize::new(0),
            processed_count: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
        }
    }

    /// Push work into global injector
    pub fn push(&self, object: usize) {
        if self.closed.load(Ordering::Relaxed) {
            return;
        }
        self.injector.push(object);
        self.enqueued_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Create a new worker for a GC thread
    pub fn create_worker(&self) -> MarkingWorker<'_> {
        let worker = Worker::new_fifo();
        let stealer = worker.stealer();

        self.stealers.write().push(stealer);

        MarkingWorker {
            worker,
            injector: &self.injector,
            all_stealers: self.stealers.clone(),
            processed_local: 0,
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    pub fn is_empty(&self) -> bool {
        self.injector.is_empty()
    }

    pub fn len(&self) -> usize {
        self.injector.len()
    }

    pub fn clear(&self) {
        while self.injector.steal().is_success() {}
    }

    pub fn stats(&self) -> MarkQueueStats {
        MarkQueueStats {
            enqueued: self.enqueued_count.load(Ordering::Relaxed),
            processed: self.processed_count.load(Ordering::Relaxed),
            pending: 0, // Pending is hard to calculate exactly in lock-free
        }
    }
}

/// MarkingWorker - Local queue for a single GC thread
pub struct MarkingWorker<'a> {
    worker: Worker<usize>,
    injector: &'a Injector<usize>,
    all_stealers: Arc<parking_lot::RwLock<Vec<Stealer<usize>>>>,
    processed_local: usize,
}

impl<'a> MarkingWorker<'a> {
    /// Push work to local queue
    pub fn push(&mut self, object: usize) {
        self.worker.push(object);
    }

    /// Pop work (local first, then steal)
    pub fn pop(&mut self) -> Option<usize> {
        // 1. Local pop
        if let Some(obj) = self.worker.pop() {
            self.processed_local += 1;
            return Some(obj);
        }

        // 2. Global injector pop
        loop {
            match self.injector.steal_batch_and_pop(&self.worker) {
                crossbeam_deque::Steal::Success(obj) => return Some(obj),
                crossbeam_deque::Steal::Empty => break,
                crossbeam_deque::Steal::Retry => continue,
            }
        }

        // 3. Steal from others
        let stealers = self.all_stealers.read();
        if stealers.is_empty() {
            return None;
        }

        // Try to steal from a random worker (simplified for MVP)
        for stealer in stealers.iter() {
            loop {
                match stealer.steal_batch_and_pop(&self.worker) {
                    crossbeam_deque::Steal::Success(obj) => return Some(obj),
                    crossbeam_deque::Steal::Empty => break,
                    crossbeam_deque::Steal::Retry => continue,
                }
            }
        }

        None
    }
}

pub struct MarkQueueStats {
    pub enqueued: usize,
    pub processed: usize,
    pub pending: usize,
}

impl Default for MarkQueue {
    fn default() -> Self {
        Self::new()
    }
}
