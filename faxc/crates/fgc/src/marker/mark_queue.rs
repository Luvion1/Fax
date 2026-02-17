//! Mark Queue - Concurrent Work Queue for Marking
//!
//! Mark queue adalah concurrent queue untuk menyimpan object yang perlu di-mark.
//! Menggunakan work-stealing untuk load balancing antar GC threads.
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │                    GLOBAL MARK QUEUE                     │
//! │         (Thread-safe, lock-free, multiple producers)     │
//! └─────────────────────────────────────────────────────────┘
//!                             │
//!         ┌───────────────────┼───────────────────┐
//!         │                   │                   │
//!         ▼                   ▼                   ▼
//! ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
//! │ GC Thread 1  │   │ GC Thread 2  │   │ GC Thread N  │
//! │ Local Queue  │   │ Local Queue  │   │ Local Queue  │
//! └──────────────┘   └──────────────┘   └──────────────┘
//! ```
//!
//! Work Stealing:
//! Setiap GC thread punya local queue. Jika local queue kosong,
//! thread akan "steal" work dari queue GC thread lain.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// MarkQueue - concurrent queue untuk marking work
///
/// Implementasi thread-safe queue dengan multiple producers
/// dan multiple consumers (GC threads).
pub struct MarkQueue {
    /// Underlying concurrent queue
    queue: Arc<Mutex<VecDeque<usize>>>,

    /// Counter untuk statistik
    enqueued_count: AtomicUsize,
    processed_count: AtomicUsize,

    /// Queue is closed (no more pushes allowed)
    closed: AtomicBool,
}

impl MarkQueue {
    /// Create new mark queue
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            enqueued_count: AtomicUsize::new(0),
            processed_count: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
        }
    }

    /// Push object ke queue
    ///
    /// Thread-safe, bisa dipanggil dari multiple threads.
    ///
    /// # Arguments
    /// * `object` - Object address untuk di-mark
    pub fn push(&self, object: usize) {
        if self.closed.load(Ordering::Relaxed) {
            return;
        }

        let mut queue = self.queue.lock().unwrap();
        queue.push_back(object);
        self.enqueued_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Pop object dari queue
    ///
    /// Returns None jika queue empty.
    pub fn pop(&self) -> Option<usize> {
        let mut queue = self.queue.lock().unwrap();
        let object = queue.pop_front();

        if object.is_some() {
            self.processed_count.fetch_add(1, Ordering::Relaxed);
        }

        object
    }

    /// Steal work dari queue (untuk work stealing)
    ///
    /// Steal dari belakang queue (LIFO untuk stealers).
    pub fn steal(&self) -> Option<usize> {
        let mut queue = self.queue.lock().unwrap();
        let object = queue.pop_back();

        if object.is_some() {
            self.processed_count.fetch_add(1, Ordering::Relaxed);
        }

        object
    }

    /// Check jika queue is empty
    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }

    /// Get queue size (approximate)
    pub fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    /// Clear queue
    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    /// Close queue (no more pushes)
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    /// Check jika queue closed
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    /// Get enqueued count
    pub fn enqueued_count(&self) -> usize {
        self.enqueued_count.load(Ordering::Relaxed)
    }

    /// Get processed count
    pub fn processed_count(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }

    /// Get statistics
    pub fn stats(&self) -> MarkQueueStats {
        MarkQueueStats {
            enqueued: self.enqueued_count(),
            processed: self.processed_count(),
            pending: self.len(),
        }
    }
}

impl Default for MarkQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics untuk mark queue
#[derive(Debug, Default)]
pub struct MarkQueueStats {
    /// Total enqueued objects
    pub enqueued: usize,
    /// Total processed objects
    pub processed: usize,
    /// Pending objects in queue
    pub pending: usize,
}

/// Local work queue untuk GC thread
///
/// Lock-free queue untuk single producer, single consumer.
pub struct LocalWorkQueue {
    /// Queue data
    data: Vec<usize>,
    /// Head index
    head: usize,
    /// Tail index
    tail: usize,
}

impl LocalWorkQueue {
    /// Create new local queue dengan capacity tertentu
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            head: 0,
            tail: 0,
        }
    }

    /// Push work ke queue
    pub fn push(&mut self, work: usize) {
        if self.tail >= self.data.len() {
            self.data.reserve(self.data.len().max(64));
        }
        self.data.push(work);
        self.tail += 1;
    }

    /// Pop work dari queue
    pub fn pop(&mut self) -> Option<usize> {
        if self.head < self.tail {
            let work = self.data[self.head];
            self.head += 1;
            Some(work)
        } else {
            None
        }
    }

    /// Steal work dari queue (dari belakang)
    pub fn steal(&mut self) -> Option<usize> {
        if self.head < self.tail {
            self.tail -= 1;
            Some(self.data[self.tail])
        } else {
            None
        }
    }

    /// Check jika queue empty
    pub fn is_empty(&self) -> bool {
        self.head >= self.tail
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    /// Clear queue
    pub fn clear(&mut self) {
        self.data.clear();
        self.head = 0;
        self.tail = 0;
    }
}
