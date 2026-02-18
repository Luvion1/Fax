//! String interner implementation using DashMap for concurrent access.
//!
//! This module provides a thread-safe string interner optimized for compiler use cases:
//! - Lock-free concurrent access via DashMap
//! - Fast hashing with AHasher
//! - Statistics tracking for profiling
//! - Pre-allocated buffer for common symbols
//!
//! # Performance Characteristics
//!
//! - **Interning (hit)**: O(1) - hash lookup only
//! - **Interning (miss)**: O(1) - hash insert + allocation
//! - **Symbol comparison**: O(1) - pointer/index comparison
//! - **String retrieval**: O(n) - linear search by index (rare operation)
//!
//! # Thread Safety
//!
//! The interner is fully thread-safe (`Sync + Send`). Multiple threads can intern
//! strings concurrently without blocking each other, thanks to DashMap's
//! lock-free design.

use ahash::AHasher;
use dashmap::DashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::LazyLock;

use super::{InternerStats, Symbol};

/// Global string table instance using DashMap for concurrent access
///
/// Initialized on first use via `LazyLock`. All known keywords are pre-interned
/// during initialization to ensure they have stable, predictable indices.
pub static STRING_TABLE: LazyLock<StringTable> = LazyLock::new(|| {
    let table = StringTable::new();
    table.initialize_known_symbols();
    table
});

/// Thread-safe string table using DashMap
///
/// This structure holds all interned strings. It uses DashMap for
/// lock-free concurrent access, which provides better performance
/// than `RwLock<HashMap>` for read-heavy workloads.
///
/// # Memory Layout
///
/// Strings are allocated on the heap and leaked (intentionally) to obtain
/// `'static` lifetime references. This is safe because:
/// 1. The string table lives for the entire program duration
/// 2. Interned strings are never removed
/// 3. This avoids lifetime tracking overhead
pub struct StringTable {
    /// Maps string hash to (string, symbol index) for fast lookup
    ///
    /// Using `u64` hash as key enables O(1) lookup without string comparison
    /// in the common case (no hash collision).
    map: DashMap<u64, (&'static str, u32)>,

    /// Counter for next index (atomic for lock-free increment)
    ///
    /// Starts at `RESERVED_SYMBOLS_END` to leave room for pre-defined keywords.
    next_index: AtomicU32,

    /// Number of hash collisions encountered
    ///
    /// Useful for profiling hash function quality.
    collisions: AtomicUsize,

    /// Number of cache hits (string already interned)
    hits: AtomicUsize,

    /// Number of cache misses (new string allocation)
    misses: AtomicUsize,
}

/// Number of symbols reserved for known keywords
///
/// This constant defines the boundary between pre-defined keyword symbols
/// and dynamically interned symbols. All symbols with index < this value
/// are known at compile time.
const RESERVED_SYMBOLS_END: u32 = 512;

impl StringTable {
    /// Create a new empty string table
    ///
    /// The table starts with default capacity and grows as needed.
    /// Known symbols are initialized separately via `initialize_known_symbols()`.
    #[inline]
    fn new() -> Self {
        Self {
            map: DashMap::with_capacity(256),
            next_index: AtomicU32::new(RESERVED_SYMBOLS_END),
            collisions: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Initialize all known keyword symbols
    ///
    /// Pre-interns all language keywords and common symbols to ensure:
    /// 1. They have stable, predictable indices
    /// 2. They're available without allocation during compilation
    /// 3. Fast path lookup for common keywords
    ///
    /// This must be called exactly once during initialization.
    fn initialize_known_symbols(&self) {
        // List of all known symbols to pre-intern
        // These must match the order in mod.rs known symbol constants
        let known_symbols = [
            // Control flow keywords
            "fn", "let", "const", "mut", "if", "else", "while", "for", "loop",
            "return", "break", "continue", "match",
            // Type declarations
            "struct", "enum", "impl", "trait", "type", "mod", "use",
            // Visibility and modules
            "pub", "crate", "super", "self", "Self", "as",
            // Literals and values
            "true", "false",
            // Control flow extended
            "unsafe", "extern",
            // Type keywords - integers
            "i8", "i16", "i32", "i64", "isize",
            "u8", "u16", "u32", "u64", "usize",
            // Type keywords - floats
            "f32", "f64",
            // Type keywords - other
            "bool", "char", "str",
            // Type system
            "dyn", "where",
            // Container types
            "array", "tuple",
            // Operators as symbols
            "add", "sub", "mul", "div", "rem", "neg", "not",
            "bit_and", "bit_or", "bit_xor", "bit_not",
            "shl", "shr",
            "eq", "ne", "lt", "le", "gt", "ge",
            "and", "or",
            // Common identifiers
            "main", "new", "init", "drop",
            "ok", "err", "some", "none",
            "len", "size", "capacity",
            "push", "pop", "insert", "remove",
            "get", "set", "first", "last",
            "iter", "next", "done",
            "clone", "copy", "eq", "ord",
            "hash", "hasher",
            "from", "into", "try_from", "try_into",
            "default", "display", "debug",
        ];

        for (idx, symbol) in known_symbols.iter().enumerate() {
            let actual_idx = idx as u32;
            if actual_idx < RESERVED_SYMBOLS_END {
                let interned: &'static str = Box::leak(symbol.to_string().into_boxed_str());
                let hash = Self::hash_string(symbol);
                self.map.insert(hash, (interned, actual_idx));
            }
        }
    }

    /// Intern a string, returning its symbol
    ///
    /// This function is thread-safe and uses DashMap for concurrent access.
    /// If the string is already interned, returns the existing symbol.
    /// Otherwise, allocates a new entry and returns a new symbol.
    ///
    /// # Performance
    ///
    /// - **Best case** (string already interned): O(1) hash lookup
    /// - **Worst case** (new unique string): O(1) hash insert + allocation
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. Multiple threads can intern strings
    /// concurrently without blocking each other.
    ///
    /// # Statistics
    ///
    /// Each call updates hit/miss counters for profiling. Use
    /// [`StringTable::stats()`] to retrieve statistics.
    pub fn intern(&self, string: &str) -> Symbol {
        let hash = Self::hash_string(string);

        // Fast path: check if string is already interned
        if let Some(entry) = self.map.get(&hash) {
            if entry.value().0 == string {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Symbol { index: entry.value().1 };
            }
        }

        // Slow path: need to insert
        self.misses.fetch_add(1, Ordering::Relaxed);

        match self.map.entry(hash) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                // Hash collision - check if string matches
                if entry.get().0 == string {
                    return Symbol { index: entry.get().1 };
                }
                // Different string with same hash - handle collision
                self.handle_collision(string, hash)
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                // Allocate new string with 'static lifetime
                let interned: &'static str = Box::leak(string.to_string().into_boxed_str());
                let idx = self.next_index.fetch_add(1, Ordering::Relaxed);
                entry.insert((interned, idx));
                Symbol { index: idx }
            }
        }
    }

    /// Handle hash collisions by probing with modified hashes
    ///
    /// Uses linear probing with a prime-based offset to find an empty slot
    /// or the existing string entry.
    ///
    /// # Arguments
    ///
    /// * `string` - The string to intern
    /// * `original_hash` - The original hash that collided
    ///
    /// # Returns
    ///
    /// The symbol for the string (either existing or newly created)
    fn handle_collision(&self, string: &str, original_hash: u64) -> Symbol {
        // Linear probing with prime-based offset for better distribution
        const MAX_PROBES: u64 = 32;
        const PROBE_PRIME: u64 = 0x9e3779b97f4a7c15; // Golden ratio constant

        for i in 1u64..=MAX_PROBES {
            let probe_hash = original_hash.wrapping_add(i.wrapping_mul(PROBE_PRIME));

            if let Some(entry) = self.map.get(&probe_hash) {
                if entry.value().0 == string {
                    return Symbol { index: entry.value().1 };
                }
            } else {
                // Found empty slot
                self.collisions.fetch_add(1, Ordering::Relaxed);
                let interned: &'static str = Box::leak(string.to_string().into_boxed_str());
                let idx = self.next_index.fetch_add(1, Ordering::Relaxed);
                self.map.insert(probe_hash, (interned, idx));
                return Symbol { index: idx };
            }
        }

        // Fallback: extremely unlikely to reach here
        // Use original hash and accept potential overwrite
        self.collisions.fetch_add(1, Ordering::Relaxed);
        let interned: &'static str = Box::leak(string.to_string().into_boxed_str());
        let idx = self.next_index.fetch_add(1, Ordering::Relaxed);
        self.map.insert(original_hash, (interned, idx));
        Symbol { index: idx }
    }

    /// Get string by symbol
    ///
    /// # Performance
    ///
    /// O(n) where n is the number of interned strings, as DashMap doesn't
    /// support efficient index-based lookup. This is a deliberate trade-off
    /// for better concurrent write performance.
    ///
    /// # Panics
    ///
    /// Returns `None` if the symbol is invalid (e.g., created manually with
    /// an out-of-bounds index that doesn't exist in the table).
    pub fn get(&self, symbol: Symbol) -> Option<&'static str> {
        // Linear search by index - O(n) but should be rare
        self.map
            .iter()
            .find(|entry| entry.value().1 == symbol.index)
            .map(|entry| entry.value().0)
    }

    /// Compute hash of string using AHasher
    ///
    /// AHasher is a non-cryptographic hash function optimized for speed
    /// while maintaining good distribution properties.
    #[inline]
    fn hash_string(string: &str) -> u64 {
        let mut hasher = AHasher::default();
        string.hash(&mut hasher);
        hasher.finish()
    }

    /// Get statistics about the string table for profiling
    ///
    /// Returns an `InternerStats` struct with detailed information about
    /// memory usage, efficiency, and performance characteristics.
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe and can be called concurrently.
    pub fn stats(&self) -> InternerStats {
        let count = self.map.len();
        let capacity = self.map.capacity();
        let collisions = self.collisions.load(Ordering::Relaxed);
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);

        InternerStats {
            count,
            capacity,
            collisions,
            hits,
            misses,
        }
    }

    /// Reset statistics counters (useful for benchmarking)
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe but may race with concurrent
    /// interning operations. Use only in single-threaded benchmarks.
    #[cfg(test)]
    pub(crate) fn reset_stats(&self) {
        self.collisions.store(0, Ordering::Relaxed);
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_intern_same_string() {
        let s1 = STRING_TABLE.intern("hello");
        let s2 = STRING_TABLE.intern("hello");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_intern_different_strings() {
        let s1 = STRING_TABLE.intern("hello");
        let s2 = STRING_TABLE.intern("world");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_get_string() {
        let symbol = STRING_TABLE.intern("test_string");
        let string = STRING_TABLE.get(symbol);
        assert_eq!(string, Some("test_string"));
    }

    #[test]
    fn test_concurrent_intern() {
        let handles: Vec<_> = (0..20)
            .map(|i| {
                thread::spawn(move || {
                    let s = STRING_TABLE.intern(&format!("thread_{}", i));
                    (i, s)
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All symbols should be unique
        let symbols: Vec<_> = results.iter().map(|(_, s)| *s).collect();
        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                assert_ne!(symbols[i], symbols[j]);
            }
        }
    }

    #[test]
    fn test_concurrent_same_string() {
        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let s = STRING_TABLE.intern("concurrent_same");
                    s
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All should be the same symbol
        for symbol in &results[1..] {
            assert_eq!(results[0], *symbol);
        }
    }

    #[test]
    fn test_stats_tracking() {
        STRING_TABLE.reset_stats();

        // First intern should be a miss
        let _ = STRING_TABLE.intern("new_unique_string_12345");
        let stats = STRING_TABLE.stats();
        assert!(stats.misses >= 1);

        // Second intern of same string should be a hit
        let _ = STRING_TABLE.intern("new_unique_string_12345");
        let stats = STRING_TABLE.stats();
        assert!(stats.hits >= 1);
    }

    #[test]
    fn test_stats_struct() {
        let stats = STRING_TABLE.stats();
        assert!(stats.count > 0); // Known symbols are pre-interned
        assert!(stats.capacity >= stats.count);
        assert!(stats.load_factor() >= 0.0);
        assert!(stats.load_factor() <= 1.0);
    }

    #[test]
    fn test_hash_collision_handling() {
        // Intern many strings to increase collision probability
        let mut symbols = Vec::new();
        for i in 0..1000 {
            let s = STRING_TABLE.intern(&format!("collision_test_{}", i));
            symbols.push(s);
        }

        // All should be unique
        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                assert_ne!(symbols[i], symbols[j]);
            }
        }

        // Verify we can retrieve all strings
        for (i, sym) in symbols.iter().enumerate() {
            let expected = format!("collision_test_{}", i);
            assert_eq!(STRING_TABLE.get(*sym), Some(expected.as_str()));
        }
    }

    #[test]
    fn test_empty_string() {
        let s = STRING_TABLE.intern("");
        assert_eq!(STRING_TABLE.get(s), Some(""));
    }

    #[test]
    fn test_unicode_strings() {
        let test_cases = ["你好", "世界", "🦀", "こんにちは", "Привет"];

        for test in &test_cases {
            let sym = STRING_TABLE.intern(test);
            assert_eq!(STRING_TABLE.get(sym), Some(*test));
        }
    }

    #[test]
    fn test_long_strings() {
        let long_string = "a".repeat(10000);
        let sym = STRING_TABLE.intern(&long_string);
        assert_eq!(STRING_TABLE.get(sym), Some(long_string.as_str()));
    }

    #[test]
    fn test_thread_safety_stress() {
        const THREADS: usize = 50;
        const ITERATIONS: usize = 100;

        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                thread::spawn(move || {
                    let mut local_symbols = Vec::new();
                    for i in 0..ITERATIONS {
                        let s = STRING_TABLE.intern(&format!("stress_{}_{}", t, i));
                        local_symbols.push(s);
                    }
                    local_symbols
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Verify all symbols within each thread are unique
        for symbols in &results {
            for i in 0..symbols.len() {
                for j in (i + 1)..symbols.len() {
                    assert_ne!(symbols[i], symbols[j]);
                }
            }
        }
    }

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StringTable>();
    }
}
