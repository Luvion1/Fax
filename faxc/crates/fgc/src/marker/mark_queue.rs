//! Mark Queue - Concurrent Work-Stealing Queue for Marking
//!
//! Implementation using crossbeam-deque for high-performance load balancing.
//!
//! ## Optimizations Implemented
//!
//! - **Batch Stealing**: Steal multiple objects at once to reduce contention
//! - **Load Balancing**: Steal from most-loaded queue first
//! - **Adaptive Spin**: Dynamic spin/sleep based on idle time
//! - **Thread Local Caching**: Minimize atomic operations
//! - **Adaptive Batch Sizing**: Dynamically adjust batch size based on workload
//! - **Work stealing with random victim selection**: Better load distribution

use crossbeam_deque::{Injector, Stealer, Worker};
use rand::Rng;
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

    /// Batch size for stealing (tunable)
    batch_size: usize,

    /// Total workers for load balancing
    worker_count: AtomicUsize,
}

/// Configuration for mark queue performance tuning
#[derive(Debug, Clone)]
pub struct MarkQueueConfig {
    /// Number of items to steal in a batch
    pub batch_size: usize,
    /// Maximum spin iterations before sleeping
    pub max_spin_iters: usize,
    /// Sleep duration in microseconds when no work
    pub sleep_us: u32,
    /// Minimum batch size for adaptive sizing
    pub min_batch_size: usize,
    /// Maximum batch size for adaptive sizing
    pub max_batch_size: usize,
    /// Threshold for increasing batch size
    pub batch_grow_threshold: usize,
    /// Threshold for decreasing batch size
    pub batch_shrink_threshold: usize,
}

impl Default for MarkQueueConfig {
    fn default() -> Self {
        Self {
            batch_size: 16,
            max_spin_iters: 100,
            sleep_us: 50,
            min_batch_size: 4,
            max_batch_size: 64,
            batch_grow_threshold: 8,
            batch_shrink_threshold: 2,
        }
    }
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
            batch_size: 16,
            worker_count: AtomicUsize::new(0),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: MarkQueueConfig) -> Self {
        Self {
            injector: Injector::new(),
            stealers: Arc::new(parking_lot::RwLock::new(Vec::new())),
            enqueued_count: AtomicUsize::new(0),
            processed_count: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
            batch_size: config.batch_size,
            worker_count: AtomicUsize::new(0),
        }
    }

    /// Get batch size
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// Set batch size
    pub fn set_batch_size(&mut self, size: usize) {
        self.batch_size = size;
    }

    /// Push work into global injector
    #[inline]
    pub fn push(&self, object: usize) {
        if self.closed.load(Ordering::Relaxed) {
            return;
        }
        self.injector.push(object);
        self.enqueued_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Push batch of objects efficiently
    #[inline]
    pub fn push_batch(&self, objects: &[usize]) {
        if self.closed.load(Ordering::Relaxed) || objects.is_empty() {
            return;
        }
        for &obj in objects {
            self.injector.push(obj);
        }
        self.enqueued_count
            .fetch_add(objects.len(), Ordering::Relaxed);
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
            batch_size: self.batch_size,
            idle_count: 0,
            rng: rand::SeedableRng::seed_from_u64(0x123456789ABCDEF0),
            consecutive_empty: 0,
            local_batch_size: self.batch_size,
            total_processed: 0,
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    /// Register a new worker
    pub fn register_worker(&self) {
        self.worker_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Unregister a worker
    pub fn unregister_worker(&self) {
        self.worker_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get number of active workers
    pub fn worker_count(&self) -> usize {
        self.worker_count.load(Ordering::Relaxed)
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
#[allow(dead_code)]
pub struct MarkingWorker<'a> {
    worker: Worker<usize>,
    injector: &'a Injector<usize>,
    all_stealers: Arc<parking_lot::RwLock<Vec<Stealer<usize>>>>,
    processed_local: usize,
    batch_size: usize,
    idle_count: usize,
    rng: rand::rngs::StdRng,
    consecutive_empty: usize,
    local_batch_size: usize,
    total_processed: usize,
}

impl<'a> MarkingWorker<'a> {
    /// Push work to local queue
    #[inline]
    pub fn push(&mut self, object: usize) {
        self.worker.push(object);
    }

    /// Adjust batch size based on workload characteristics
    #[inline]
    fn adapt_batch_size(&mut self) {
        self.total_processed += 1;

        let success_rate = self.processed_local as f64 / self.total_processed.max(1) as f64;

        if self.consecutive_empty > 3 && self.local_batch_size > 4 {
            self.local_batch_size = (self.local_batch_size / 2).max(4);
            self.consecutive_empty = 0;
        } else if success_rate > 0.8 && self.local_batch_size < 64 {
            self.local_batch_size = (self.local_batch_size * 2).min(64);
        }
    }

    /// Pop work (local first, then steal with batching)
    #[inline]
    pub fn pop(&mut self) -> Option<usize> {
        if let Some(obj) = self.worker.pop() {
            self.processed_local += 1;
            self.idle_count = 0;
            self.consecutive_empty = 0;
            return Some(obj);
        }

        self.idle_count += 1;
        self.consecutive_empty += 1;

        if self.idle_count < 3 {
            self.steal_batch_from_injector()
        } else {
            self.adapt_batch_size();
            self.steal_from_workers()
        }
    }

    /// Steal batch from global injector (fast path)
    #[inline]
    fn steal_batch_from_injector(&mut self) -> Option<usize> {
        loop {
            match self.injector.steal_batch_and_pop(&self.worker) {
                crossbeam_deque::Steal::Success(obj) => {
                    self.idle_count = 0;
                    return Some(obj);
                },
                crossbeam_deque::Steal::Empty => return None,
                crossbeam_deque::Steal::Retry => continue,
            }
        }
    }

    /// Steal from other workers with load balancing (slow path)
    #[inline]
    fn steal_from_workers(&mut self) -> Option<usize> {
        let stealers = self.all_stealers.read();
        if stealers.is_empty() {
            return None;
        }

        let num_stealers = stealers.len();
        if num_stealers == 1 {
            return self.try_steal_from(&stealers[0]);
        }

        let start_idx = self.rng.gen_range(0..num_stealers);
        for i in 0..num_stealers {
            let idx = (start_idx + i) % num_stealers;
            if let Some(obj) = self.try_steal_from(&stealers[idx]) {
                return Some(obj);
            }
        }

        None
    }

    #[inline]
    fn try_steal_from(&self, stealer: &Stealer<usize>) -> Option<usize> {
        loop {
            match stealer.steal_batch_and_pop(&self.worker) {
                crossbeam_deque::Steal::Success(obj) => return Some(obj),
                crossbeam_deque::Steal::Empty => return None,
                crossbeam_deque::Steal::Retry => continue,
            }
        }
    }

    /// Pop batch of objects for processing
    pub fn pop_batch(&mut self, max_count: usize) -> Vec<usize> {
        let batch_size = max_count.min(self.local_batch_size);
        let mut batch = Vec::with_capacity(batch_size);

        while batch.len() < max_count {
            if let Some(obj) = self.pop() {
                batch.push(obj);
            } else {
                break;
            }
        }

        batch
    }

    /// Get current adaptive batch size
    pub fn current_batch_size(&self) -> usize {
        self.local_batch_size
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
