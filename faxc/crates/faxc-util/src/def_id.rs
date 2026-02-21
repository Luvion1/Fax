use std::sync::atomic::{AtomicU32, Ordering};
use crate::Idx;

/// Global unique identifier for definitions (functions, variables, types, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefId(pub u32);

impl Idx for DefId {
    fn from_usize(idx: usize) -> Self {
        DefId(idx as u32)
    }
    fn index(self) -> usize {
        self.0 as usize
    }
}

impl DefId {
    /// Reserved DefId for local/temporary usage or errors
    pub const DUMMY: DefId = DefId(u32::MAX);
    
    pub fn is_dummy(self) -> bool {
        self == Self::DUMMY
    }
}

/// Generator for unique DefIds
pub struct DefIdGenerator {
    counter: AtomicU32,
}

impl DefIdGenerator {
    /// Create a new generator starting from 0
    pub fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }

    /// Generate a new unique DefId
    pub fn next(&self) -> DefId {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        if id == u32::MAX {
            panic!("DefId overflow! Compiler reached maximum number of definitions.");
        }
        DefId(id)
    }
}

impl Default for DefIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
