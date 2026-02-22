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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_def_id_new_and_index() {
        let id = DefId::from_usize(42);
        assert_eq!(id.index(), 42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_def_id_dummy() {
        assert_eq!(DefId::DUMMY.0, u32::MAX);
        assert!(DefId::DUMMY.is_dummy());
        assert!(!DefId(0).is_dummy());
    }

    #[test]
    fn test_def_id_comparison() {
        let id1 = DefId(1);
        let id2 = DefId(2);
        let id3 = DefId(1);

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_def_id_generator_new() {
        let gen = DefIdGenerator::new();
        assert_eq!(gen.counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_def_id_generator_default() {
        let gen = DefIdGenerator::default();
        assert_eq!(gen.counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_def_id_generator_next() {
        let gen = DefIdGenerator::new();
        
        let id1 = gen.next();
        assert_eq!(id1, DefId(0));
        
        let id2 = gen.next();
        assert_eq!(id2, DefId(1));
        
        let id3 = gen.next();
        assert_eq!(id3, DefId(2));
        
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_def_id_generator_multiple_generators() {
        let gen1 = DefIdGenerator::new();
        let gen2 = DefIdGenerator::new();
        
        assert_eq!(gen1.next(), DefId(0));
        assert_eq!(gen2.next(), DefId(0));
        assert_eq!(gen1.next(), DefId(1));
        assert_eq!(gen2.next(), DefId(1));
    }

    #[test]
    fn test_def_id_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        let id1 = DefId(1);
        let id2 = DefId(2);
        let id3 = DefId(1);
        
        set.insert(id1);
        set.insert(id2);
        set.insert(id3);
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }
}
