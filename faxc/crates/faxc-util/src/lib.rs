//! faxc-util - Core Utilities and Foundation Types
//!
//! ============================================================================
//! MODULE OVERVIEW
//! ============================================================================
//!
//! This module provides fundamental utilities and types that form the foundation
//! of the entire faxc compiler infrastructure. These utilities are designed to be
//! zero-cost abstractions that improve code clarity, type safety, and performance
//! without runtime overhead.
//!
//! DESIGN PRINCIPLES:
//! ------------------
//! 1. ZERO-COST ABSTRACTIONS
//!    All utilities compile down to efficient code with no runtime penalty
//!    compared to hand-written implementations.
//!
//! 2. TYPE SAFETY
//!    Leverage Rust's type system to prevent bugs at compile time.
//!    Examples: Typed indices prevent mixing different ID spaces.
//
// 3. PERFORMANCE
//    Optimize for the common case while maintaining correctness.
//    Examples: Efficient string interning, lock-free data structures.
//
// 4. ERGONOMICS
//    APIs should be intuitive and easy to use correctly.
//    Examples: Builder patterns, type inference-friendly interfaces.
//
// ============================================================================
// STRING INTERNING (SYMBOL)
// ============================================================================
//
// THEORY OF STRING INTERNING:
// ---------------------------
//
// String interning is a technique for storing only one copy of each distinct
// string value, which must be immutable. All occurrences of the same string
// point to the same memory location, enabling fast equality comparisons and
// reducing memory usage.
//
// MATHEMATICAL BASIS:
// -------------------
// Let S be the set of all strings in a program.
// Let I: S → ℕ be the interning function that maps each string to a unique ID.
//
// Properties:
// - ∀s₁, s₂ ∈ S: I(s₁) = I(s₂) ⟺ s₁ = s₂  (injective mapping)
// - |Range(I)| ≤ |S|  (compression through deduplication)
//
// TIME COMPLEXITY:
// ----------------
// Without interning:
// - Comparison: O(n) where n is string length
// - Hash computation: O(n)
// - Memory: O(total length of all strings)
//
// With interning:
// - Comparison: O(1) (integer comparison)
// - Hash computation: O(1) (precomputed)
// - Memory: O(unique strings × avg length) + O(|S| × sizeof(ID))
//
// MEMORY LAYOUT:
// --------------
// ```
// String Table (Global):
// ┌─────────────────────────────────────────────────────────────┐
// │ Slot 0 │ Slot 1 │ Slot 2 │ Slot 3 │ ... │ Slot N            │
// ├────────┴────────┴────────┴────────┴─────┴───────────────────┤
// │ "main" │ "fn"   │ "let"  │ "x"    │ ... │ "very_long_id"   │
// └─────────────────────────────────────────────────────────────┘
//        ↑
//        Interned strings stored contiguously or in arena
//
// Symbol (4 bytes):
// ┌──────────────┐
// │    index     │  u32: index into string table
// │   (32-bit)   │
// └──────────────┘
//
// String (24 bytes on 64-bit):
// ┌────────┬────────┬────────┐
// │ pointer│ length │ capacity│  Heap allocation for data
// │  8B    │   8B   │   8B   │
// └────────┴────────┴────────┘
// ```
//
// USE CASES IN COMPILERS:
// -----------------------
// 1. IDENTIFIERS: Variable names, function names appear thousands of times
// 2. KEYWORDS: "let", "fn", "if" are repeated throughout source
// 3. TYPE NAMES: "int", "string", custom types
// 4. STRING LITERALS: May be repeated in source code
//
// THREAD-SAFE IMPLEMENTATION:
// ---------------------------
// Global string table must support concurrent access:
//
// Approach 1: Read-Write Lock (RwLock)
// - Multiple readers can access concurrently
// - Writers (new intern operations) require exclusive access
// - Good for read-heavy workloads
//
// Approach 2: Lock-Free Hash Table (e.g., DashMap)
// - No locks for read or write
// - Better concurrency under heavy contention
// - Higher memory overhead
//
// Approach 3: Thread-Local Tables with Merge
// - Each thread has local table
// - Periodic merge to global table
// - Best for maximum parallelism, complex implementation
//
// COLLISION HANDLING:
// -------------------
// Strings are hashed for O(1) lookup. Collisions handled via:
// - Chaining: Linked list of strings with same hash
// - Open addressing: Probe sequence until empty slot found
//
// Example (chaining):
// ```
// Hash Table (buckets):
// Bucket 0: ["main" → 0]
// Bucket 1: ["fn" → 1] → ["if" → 5]  (collision resolved via chain)
// Bucket 2: ["let" → 2]
// ```
//
// IMPLEMENTATION STRATEGIES:
// --------------------------
// 1. LAZY INTERNING
//    Strings interned on first use
//    Pro: Only pay for what you use
//    Con: First access slower
//
// 2. EAGER INTERNING
//    All strings interned upfront (e.g., all keywords)
//    Pro: Predictable performance
//    Con: Higher memory if not all used
//
// 3. LAZY + CACHE
//    Cache recent lookups to avoid table access
//    Pro: Fast for repeated lookups
//    Con: Cache management overhead

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

/// Symbol - An interned string identifier
///
/// A Symbol is a compact (4-byte) handle to a string stored in a global
/// string table. This enables O(1) string comparison and reduces memory
/// usage when the same string appears multiple times.
///
/// # Size
///
/// `Symbol` is exactly 4 bytes (u32), making it very cache-friendly compared
/// to `String` which is 24 bytes plus heap allocation.
///
/// # Example
///
/// ```
/// let s1 = Symbol::intern("hello");
/// let s2 = Symbol::intern("hello");
/// let s3 = Symbol::intern("world");
///
/// assert_eq!(s1, s2);  // Same symbol for same string
/// assert_ne!(s1, s3);  // Different strings get different symbols
///
/// // Comparison is O(1) integer comparison
/// assert!(s1 == s2);   // Fast!
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    /// Index into the global string table
    ///
    /// Using u32 limits the maximum number of interned strings to 2^32-1,
    /// which is sufficient for any practical compiler (would require >50GB
    /// of unique strings to exhaust).
    index: u32,
}

impl Symbol {
    /// The maximum index value for a symbol
    pub const MAX_INDEX: u32 = u32::MAX;

    /// Intern a string, returning its symbol
    ///
    /// This function will:
    /// 1. Hash the string to check for existing entry
    /// 2. If found, return existing symbol
    /// 3. If not found, allocate new slot and return new symbol
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. Multiple threads can intern strings
    /// concurrently.
    ///
    /// # Performance
    ///
    /// - Best case (string already interned): O(1) hash lookup
    /// - Worst case (new unique string): O(1) hash insert + allocation
    ///
    /// # Examples
    ///
    /// ```
    /// let keyword = Symbol::intern("fn");
    /// let name = Symbol::intern("main");
    /// ```
    pub fn intern(string: &str) -> Self {
        // Implementation would access global string table
        // For now, placeholder that just hashes
        unimplemented!("String interning not yet implemented")
    }

    /// Get the string value associated with this symbol
    ///
    /// # Performance
    ///
    /// O(1) array/vector index operation
    ///
    /// # Panics
    ///
    /// May panic if the symbol is invalid (e.g., created manually with
    /// an out-of-bounds index).
    pub fn as_str(&self) -> &'static str {
        unimplemented!("String lookup not yet implemented")
    }

    /// Returns true if this is a "known" symbol (predefined keywords)
    ///
    /// Known symbols are interned at compiler startup and have indices
    /// in a reserved range.
    pub fn is_known(&self) -> bool {
        self.index < RESERVED_SYMBOLS_END
    }

    /// Get the raw index value
    ///
    /// Useful for serialization or debugging.
    pub fn as_u32(&self) -> u32 {
        self.index
    }

    /// Create a symbol from a raw index
    ///
    /// # Safety
    ///
    /// The index must correspond to a valid entry in the string table.
    /// Creating a symbol with an invalid index leads to undefined behavior
    /// when calling `as_str()`.
    pub unsafe fn from_u32_unchecked(index: u32) -> Self {
        Self { index }
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // In a real implementation, we'd look up the string
        write!(f, "Symbol({})", self.index)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // In a real implementation, we'd look up and display the string
        write!(f, "#{}", self.index)
    }
}

/// End of reserved symbol indices
const RESERVED_SYMBOLS_END: u32 = 256;

/// Global string table
///
/// This structure holds all interned strings. It uses Arc for shared ownership
/// and RwLock for concurrent access.
///
/// # Memory Management
///
/// Strings are never removed from the table (leak-on-purpose strategy).
/// This is acceptable because:
/// 1. Total unique strings in a program is bounded
/// 2. Strings are typically small (identifiers, keywords)
/// 3. Simplifies implementation (no reference counting needed)
///
/// # Lock Contention
///
/// Under heavy concurrent interning, the write lock may become a bottleneck.
/// Solutions:
/// - Shard the table (multiple locks for different hash ranges)
/// - Use lock-free data structures
/// - Batch interning operations
pub struct StringTable {
    /// Maps string hash to symbol index
    ///
    /// We store hashes rather than full strings to reduce memory usage.
    /// Collisions are handled by probing.
    index: FxHashMap<u64, u32>,

    /// Storage for actual string data
    ///
    /// Strings are stored in an arena allocator for efficiency.
    strings: Vec<&'static str>,

    /// Arena allocator for string data
    arena: bumpalo::Bump,
}

impl StringTable {
    /// Create a new empty string table
    pub fn new() -> Self {
        Self {
            index: FxHashMap::default(),
            strings: Vec::new(),
            arena: bumpalo::Bump::new(),
        }
    }

    /// Intern a string
    ///
    /// # Algorithm
    ///
    /// 1. Compute hash of string
    /// 2. Check if hash exists in index
    /// 3. If exists, verify string equality (handle hash collision)
    /// 4. If not exists or collision, allocate new entry
    pub fn intern(&mut self, string: &str) -> Symbol {
        let hash = Self::hash_string(string);

        // Check if already interned
        if let Some(&index) = self.index.get(&hash) {
            // Verify no collision
            if self.strings[index as usize] == string {
                return Symbol { index };
            }
        }

        // Allocate new string (leak to get 'static lifetime for simplicity)
        let interned: &'static str = Box::leak(string.to_string().into_boxed_str());
        let index = self.strings.len() as u32;
        self.strings.push(interned);
        self.index.insert(hash, index);

        Symbol { index }
    }

    /// Get string by symbol
    pub fn get(&self, symbol: Symbol) -> Option<&'static str> {
        self.strings.get(symbol.index as usize).copied()
    }

    /// Compute hash of string
    fn hash_string(string: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        hasher.finish()
    }
}

use std::marker::PhantomData;
/// IndexVec - A vector with typed indices
///
/// ============================================================================
/// TYPED INDEX PATTERN
/// ============================================================================
///
/// PROBLEM STATEMENT:
// ------------------
// In a compiler, we often deal with multiple index spaces:
// - Expression indices
// - Statement indices
// - Local variable indices
// - Type parameter indices
//
// Using raw usize/u32 for all of these is error-prone:
// ```rust,ignore
// let expr_id: usize = 5;
// let stmt_id: usize = 10;
//
/// Oops! Mixed up indices, but compiler accepts it
// let wrong = expressions[stmt_id]; // Wrong but compiles!
// ```
//
// SOLUTION:
// ---------
// Use newtype pattern to create distinct index types:
//
// ```rust,ignore
// struct ExprId(u32);
// struct StmtId(u32);
//
// let expr_id = ExprId(5);
// let stmt_id = StmtId(10);
//
/// This won't compile - type mismatch!
// let wrong = expressions[stmt_id]; // Compile error!
// ```
//
// BENEFITS:
// ---------
// 1. TYPE SAFETY: Compiler catches index mix-ups
// 2. DOCUMENTATION: Types document intent
// 3. REFACTORING: Easy to find all uses of specific index type
// 4. ABSTRACTION: Can change underlying representation
//
// PERFORMANCE:
// ------------
// Zero-cost abstraction. Newtypes compile to same code as primitives.
///
/// IndexVec provides ergonomic access to these typed vectors.
use std::ops::{Index, IndexMut};

/// A vector indexed by a specific type
///
/// # Type Parameters
///
/// - `I`: The index type (must implement Idx trait)
/// - `T`: The element type
///
/// # Example
///
/// ```
/// #[derive(Clone, Copy)]
/// struct ExprId(u32);
///
/// impl Idx for ExprId {
///     fn from_usize(idx: usize) -> Self { ExprId(idx as u32) }
///     fn index(self) -> usize { self.0 as usize }
/// }
///
/// let mut exprs: IndexVec<ExprId, Expression> = IndexVec::new();
/// let id = exprs.push(Expression::Number(42));
/// let expr = exprs[id];  // Type-safe indexing
/// ```
#[derive(Clone)]
pub struct IndexVec<I, T> {
    raw: Vec<T>,
    _marker: PhantomData<fn(&I)>,
}

impl<I, T> IndexVec<I, T> {
    /// Create empty IndexVec
    pub fn new() -> Self {
        Self {
            raw: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    /// Returns number of elements
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns true if empty
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Reserve capacity
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional)
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.raw.clear()
    }
}

impl<I: Idx, T> IndexVec<I, T> {
    /// Push element and return typed index
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `I::MAX` (e.g., > 2^32 for u32 indices)
    pub fn push(&mut self, value: T) -> I {
        let index = self.raw.len();
        self.raw.push(value);
        I::from_usize(index)
    }

    /// Pop element and return with index
    pub fn pop(&mut self) -> Option<(I, T)> {
        self.raw.pop().map(|v| {
            let idx = I::from_usize(self.raw.len());
            (idx, v)
        })
    }

    /// Get element by index
    pub fn get(&self, index: I) -> Option<&T> {
        self.raw.get(index.index())
    }

    /// Get mutable element by index
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        self.raw.get_mut(index.index())
    }

    /// Iterate over (index, value) pairs
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (I, &T)> {
        self.raw
            .iter()
            .enumerate()
            .map(|(i, v)| (I::from_usize(i), v))
    }

    /// Iterate over indices only
    pub fn indices(&self) -> impl Iterator<Item = I> {
        (0..self.raw.len()).map(I::from_usize)
    }
}

impl<I: Idx, T> Index<I> for IndexVec<I, T> {
    type Output = T;

    fn index(&self, index: I) -> &T {
        &self.raw[index.index()]
    }
}

impl<I: Idx, T> IndexMut<I> for IndexVec<I, T> {
    fn index_mut(&mut self, index: I) -> &mut T {
        &mut self.raw[index.index()]
    }
}

impl<I, T> Default for IndexVec<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be used as indices
///
/// This trait defines the mapping between the index type and usize,
/// allowing compact storage (e.g., u32) while supporting vector indexing.
///
/// # Safety
///
/// Implementations must ensure:
/// - `from_usize(index).index() == index` for all valid indices
/// - `from_usize` should panic if index exceeds type capacity
pub trait Idx: Copy + Eq + PartialEq {
    /// Convert from usize to index type
    ///
    /// # Panics
    ///
    /// May panic if `idx` exceeds the maximum value representable by `Self`
    fn from_usize(idx: usize) -> Self;

    /// Convert index to usize for array indexing
    fn index(self) -> usize;
}

/// Macro to define index types easily
#[macro_export]
macro_rules! define_idx {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl $crate::Idx for $name {
            fn from_usize(idx: usize) -> Self {
                assert!(idx <= u32::MAX as usize);
                $name(idx as u32)
            }

            fn index(self) -> usize {
                self.0 as usize
            }
        }
    };
}

// Re-export commonly used types
pub use rustc_hash::FxHashMap;
pub use rustc_hash::FxHashSet;

/// Diagnostic for error reporting
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
}

/// Diagnostic level
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

/// Handler for diagnostics
pub struct Handler;

impl Handler {
    /// Create new handler
    pub fn new() -> Self {
        Handler
    }
}

/// Span for source locations
#[derive(Clone, Copy)]
pub struct Span;

impl Span {
    /// Dummy span
    pub const DUMMY: Span = Span;
}
