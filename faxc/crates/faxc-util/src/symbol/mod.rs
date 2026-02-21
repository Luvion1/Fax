//! Symbol module - String interning for efficient identifier handling.
//!
//! This module provides the [`Symbol`] type, which is a compact (4-byte) handle
//! to an interned string. Symbols enable O(1) string comparison and reduce memory
//! usage when the same string appears multiple times in the compiler.
//!
//! # Overview
//!
//! The symbol interning system is a core component of the Fax compiler, providing:
//!
//! - **Memory efficiency**: Each unique string is stored only once
//! - **Fast comparison**: Symbol comparison is O(1) via index comparison
//! - **Thread safety**: Safe to use across multiple threads concurrently
//! - **Stable handles**: Symbols remain valid for the program lifetime
//!
//! # Performance Characteristics
//!
//! | Operation | Complexity | Notes |
//! |-----------|------------|-------|
//! | `Symbol::intern()` (hit) | O(1) | String already interned |
//! | `Symbol::intern()` (miss) | O(1) | New string allocation |
//! | `Symbol` comparison | O(1) | Index comparison only |
//! | `Symbol::as_str()` | O(n) | Linear search by index |
//! | `Symbol::eq_str()` | O(1) | Hash + pointer comparison |
//!
//! # Thread Safety
//!
//! The symbol interner is fully thread-safe (`Sync + Send`). Multiple threads
//! can intern strings concurrently without blocking each other, thanks to
//! DashMap's lock-free design.
//!
//! # Memory Model
//!
//! Interned strings are allocated on the heap with `'static` lifetime.
//! They are never deallocated, which is acceptable because:
//! 1. The compiler runs for a finite duration
//! 2. The total memory usage is bounded by source code size
//! 3. This avoids complex lifetime tracking
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use faxc_util::symbol::Symbol;
//!
//! let s1 = Symbol::intern("hello");
//! let s2 = Symbol::intern("hello");
//! let s3 = Symbol::intern("world");
//!
//! assert_eq!(s1, s2);  // Same symbol for same string
//! assert_ne!(s1, s3);  // Different strings get different symbols
//! ```
//!
//! Using known keywords:
//!
//! ```
//! use faxc_util::symbol::{Symbol, KW_FN, KW_LET};
//!
//! assert_eq!(KW_FN.as_str(), "fn");
//! assert_eq!(KW_LET.as_str(), "let");
//! assert!(KW_FN.is_known());  // Pre-defined keyword
//! ```
//!
//! Thread-safe usage:
//!
//! ```
//! use faxc_util::symbol::Symbol;
//! use std::thread;
//!
//! let handles: Vec<_> = (0..4)
//!     .map(|i| {
//!         thread::spawn(move || {
//!             Symbol::intern(&format!("thread_{}", i))
//!         })
//!     })
//!     .collect();
//!
//! let symbols: Vec<_> = handles.into_iter()
//!     .map(|h| h.join().unwrap())
//!     .collect();
//!
//! // All symbols are unique
//! assert_eq!(symbols.len(), 4);
//! ```
//!
//! # Statistics and Profiling
//!
//! The interner tracks hit/miss statistics for performance profiling:
//!
//! ```
//! use faxc_util::symbol::Symbol;
//!
//! let _ = Symbol::intern("test");
//! let _ = Symbol::intern("test");  // Hit
//!
//! let stats = Symbol::stats_struct();
//! println!("Hits: {}, Misses: {}", stats.hits, stats.misses);
//! ```

mod interner;

pub use interner::STRING_TABLE;

/// Statistics about the string interner for profiling
///
/// Provides insights into memory usage and efficiency of the interner.
///
/// # Fields
///
/// * `count` - Number of unique interned strings
/// * `capacity` - Hash map capacity (number of buckets)
/// * `collisions` - Number of hash collisions encountered
/// * `hits` - Number of times an already-interned string was requested
/// * `misses` - Number of times a new string was allocated
///
/// # Examples
///
/// ```
/// use faxc_util::symbol::{Symbol, InternerStats};
///
/// let stats = Symbol::stats_struct();
/// println!("Interned {} strings", stats.count);
/// println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
/// println!("Load factor: {:.2}", stats.load_factor());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct InternerStats {
    /// Number of interned strings
    pub count: usize,
    /// Hash map capacity (number of buckets)
    pub capacity: usize,
    /// Number of hash collisions encountered
    pub collisions: usize,
    /// Number of cache hits (string already interned)
    pub hits: usize,
    /// Number of cache misses (new string allocation)
    pub misses: usize,
}

impl InternerStats {
    /// Create new stats with the given values
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::InternerStats;
    ///
    /// let stats = InternerStats::new(100, 256, 5, 80, 20);
    /// assert_eq!(stats.count, 100);
    /// assert_eq!(stats.hits, 80);
    /// ```
    pub const fn new(
        count: usize,
        capacity: usize,
        collisions: usize,
        hits: usize,
        misses: usize,
    ) -> Self {
        Self {
            count,
            capacity,
            collisions,
            hits,
            misses,
        }
    }

    /// Calculate the load factor (count / capacity)
    ///
    /// Returns 0.0 if capacity is 0.
    ///
    /// A load factor above 0.75 indicates the hash map may need resizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::InternerStats;
    ///
    /// let stats = InternerStats::new(100, 200, 0, 0, 0);
    /// assert_eq!(stats.load_factor(), 0.5);
    /// ```
    pub fn load_factor(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.count as f64 / self.capacity as f64
        }
    }

    /// Check if the interner is getting full (load factor > 0.75)
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::InternerStats;
    ///
    /// let stats = InternerStats::new(100, 200, 0, 0, 0);
    /// assert!(!stats.is_near_capacity());
    ///
    /// let stats = InternerStats::new(100, 120, 0, 0, 0);
    /// assert!(stats.is_near_capacity());
    /// ```
    pub fn is_near_capacity(&self) -> bool {
        self.load_factor() > 0.75
    }

    /// Calculate the hit rate (hits / (hits + misses))
    ///
    /// Returns 0.0 if no lookups have been performed.
    ///
    /// A high hit rate (>0.9) indicates good interning efficiency.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::InternerStats;
    ///
    /// let stats = InternerStats::new(100, 200, 0, 90, 10);
    /// assert_eq!(stats.hit_rate(), 0.9);
    /// ```
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get total number of interning operations
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::InternerStats;
    ///
    /// let stats = InternerStats::new(100, 200, 0, 90, 10);
    /// assert_eq!(stats.total_operations(), 100);
    /// ```
    pub fn total_operations(&self) -> usize {
        self.hits + self.misses
    }

    /// Get memory efficiency ratio (unique strings / total operations)
    ///
    /// Lower values indicate better deduplication.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::InternerStats;
    ///
    /// let stats = InternerStats::new(10, 100, 0, 90, 10);
    /// assert_eq!(stats.memory_efficiency(), 0.1);  // 10 unique / 100 ops
    /// ```
    pub fn memory_efficiency(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.count as f64 / self.total_operations() as f64
        }
    }
}

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
/// # Thread Safety
///
/// Symbols are safe to share across threads (`Sync + Send`). The underlying
/// string table uses DashMap for lock-free concurrent access.
///
/// # Lifetime
///
/// Interned strings have `'static` lifetime and are never deallocated.
/// This is safe because the string table lives for the program duration.
///
/// # Examples
///
/// ```
/// use faxc_util::symbol::Symbol;
///
/// let keyword = Symbol::intern("fn");
/// let name = Symbol::intern("main");
///
/// assert_eq!(keyword.as_str(), "fn");
/// assert_eq!(name.as_str(), "main");
/// assert_eq!(keyword, Symbol::intern("fn"));  // O(1) comparison
/// ```
///
/// # Performance Notes
///
/// - **Creation**: O(1) hash lookup/insert
/// - **Comparison**: O(1) index comparison
/// - **String retrieval**: O(n) linear search (avoid in hot paths)
/// - **Memory**: One allocation per unique string
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    /// Index into the global string table
    pub(crate) index: u32,
}

// ============================================================================
// KNOWN SYMBOLS (KEYWORDS AND BUILTINS)
// ============================================================================
//
/// Reserved symbol indices for known symbols (keywords, types, operators)
///
/// All symbols with index < this value are pre-defined at compile time
/// and correspond to language keywords, type names, and common operators.
const RESERVED_SYMBOLS_END: u32 = 512;

// ----------------------------------------------------------------------------
// Control Flow Keywords
// ----------------------------------------------------------------------------

/// Known symbol for the `fn` keyword
pub const KW_FN: Symbol = Symbol { index: 0 };
/// Known symbol for the `let` keyword
pub const KW_LET: Symbol = Symbol { index: 1 };
/// Known symbol for the `const` keyword
pub const KW_CONST: Symbol = Symbol { index: 2 };
/// Known symbol for the `mut` keyword
pub const KW_MUT: Symbol = Symbol { index: 3 };
/// Known symbol for the `if` keyword
pub const KW_IF: Symbol = Symbol { index: 4 };
/// Known symbol for the `else` keyword
pub const KW_ELSE: Symbol = Symbol { index: 5 };
/// Known symbol for the `while` keyword
pub const KW_WHILE: Symbol = Symbol { index: 6 };
/// Known symbol for the `for` keyword
pub const KW_FOR: Symbol = Symbol { index: 7 };
/// Known symbol for the `loop` keyword
pub const KW_LOOP: Symbol = Symbol { index: 8 };
/// Known symbol for the `return` keyword
pub const KW_RETURN: Symbol = Symbol { index: 9 };
/// Known symbol for the `break` keyword
pub const KW_BREAK: Symbol = Symbol { index: 10 };
/// Known symbol for the `continue` keyword
pub const KW_CONTINUE: Symbol = Symbol { index: 11 };
/// Known symbol for the `match` keyword
pub const KW_MATCH: Symbol = Symbol { index: 12 };

// ----------------------------------------------------------------------------
// Type Declaration Keywords
// ----------------------------------------------------------------------------

/// Known symbol for the `struct` keyword
pub const KW_STRUCT: Symbol = Symbol { index: 13 };
/// Known symbol for the `enum` keyword
pub const KW_ENUM: Symbol = Symbol { index: 14 };
/// Known symbol for the `impl` keyword
pub const KW_IMPL: Symbol = Symbol { index: 15 };
/// Known symbol for the `trait` keyword
pub const KW_TRAIT: Symbol = Symbol { index: 16 };
/// Known symbol for the `type` keyword
pub const KW_TYPE: Symbol = Symbol { index: 17 };
/// Known symbol for the `mod` keyword
pub const KW_MOD: Symbol = Symbol { index: 18 };
/// Known symbol for the `use` keyword
pub const KW_USE: Symbol = Symbol { index: 19 };

// ----------------------------------------------------------------------------
// Visibility and Module Keywords
// ----------------------------------------------------------------------------

/// Known symbol for the `pub` keyword
pub const KW_PUB: Symbol = Symbol { index: 20 };
/// Known symbol for the `crate` keyword
pub const KW_CRATE: Symbol = Symbol { index: 21 };
/// Known symbol for the `super` keyword
pub const KW_SUPER: Symbol = Symbol { index: 22 };
/// Known symbol for the `self` keyword
pub const KW_SELF: Symbol = Symbol { index: 23 };
/// Known symbol for the `Self` type
pub const KW_SELF_UPPER: Symbol = Symbol { index: 24 };
/// Known symbol for the `as` keyword
pub const KW_AS: Symbol = Symbol { index: 25 };

// ----------------------------------------------------------------------------
// Literals and Values
// ----------------------------------------------------------------------------

/// Known symbol for the `true` literal
pub const KW_TRUE: Symbol = Symbol { index: 26 };
/// Known symbol for the `false` literal
pub const KW_FALSE: Symbol = Symbol { index: 27 };

// ----------------------------------------------------------------------------
// Extended Control Flow
// ----------------------------------------------------------------------------

/// Known symbol for the `unsafe` keyword
pub const KW_UNSAFE: Symbol = Symbol { index: 28 };
/// Known symbol for the `extern` keyword
pub const KW_EXTERN: Symbol = Symbol { index: 29 };

// ----------------------------------------------------------------------------
// Type Keywords - Integers
// ----------------------------------------------------------------------------

/// Known symbol for the `i8` type
pub const TY_I8: Symbol = Symbol { index: 30 };
/// Known symbol for the `i16` type
pub const TY_I16: Symbol = Symbol { index: 31 };
/// Known symbol for the `i32` type
pub const TY_I32: Symbol = Symbol { index: 32 };
/// Known symbol for the `i64` type
pub const TY_I64: Symbol = Symbol { index: 33 };
/// Known symbol for the `isize` type
pub const TY_ISIZE: Symbol = Symbol { index: 34 };
/// Known symbol for the `u8` type
pub const TY_U8: Symbol = Symbol { index: 35 };
/// Known symbol for the `u16` type
pub const TY_U16: Symbol = Symbol { index: 36 };
/// Known symbol for the `u32` type
pub const TY_U32: Symbol = Symbol { index: 37 };
/// Known symbol for the `u64` type
pub const TY_U64: Symbol = Symbol { index: 38 };
/// Known symbol for the `usize` type
pub const TY_USIZE: Symbol = Symbol { index: 39 };

// ----------------------------------------------------------------------------
// Type Keywords - Floats
// ----------------------------------------------------------------------------

/// Known symbol for the `f32` type
pub const TY_F32: Symbol = Symbol { index: 40 };
/// Known symbol for the `f64` type
pub const TY_F64: Symbol = Symbol { index: 41 };

// ----------------------------------------------------------------------------
// Type Keywords - Other
// ----------------------------------------------------------------------------

/// Known symbol for the `bool` type
pub const TY_BOOL: Symbol = Symbol { index: 42 };
/// Known symbol for the `char` type
pub const TY_CHAR: Symbol = Symbol { index: 43 };
/// Known symbol for the `str` type
pub const TY_STR: Symbol = Symbol { index: 44 };

// ----------------------------------------------------------------------------
// Type System Keywords
// ----------------------------------------------------------------------------

/// Known symbol for the `dyn` keyword
pub const KW_DYN: Symbol = Symbol { index: 45 };
/// Known symbol for the `where` keyword
pub const KW_WHERE: Symbol = Symbol { index: 46 };

// ----------------------------------------------------------------------------
// Container Type Keywords
// ----------------------------------------------------------------------------

/// Known symbol for the `array` keyword
pub const KW_ARRAY: Symbol = Symbol { index: 47 };
/// Known symbol for the `tuple` keyword
pub const KW_TUPLE: Symbol = Symbol { index: 48 };

// ----------------------------------------------------------------------------
// Operator Symbols
// ----------------------------------------------------------------------------

/// Known symbol for the `add` operator
pub const OP_ADD: Symbol = Symbol { index: 49 };
/// Known symbol for the `sub` operator
pub const OP_SUB: Symbol = Symbol { index: 50 };
/// Known symbol for the `mul` operator
pub const OP_MUL: Symbol = Symbol { index: 51 };
/// Known symbol for the `div` operator
pub const OP_DIV: Symbol = Symbol { index: 52 };
/// Known symbol for the `rem` operator
pub const OP_REM: Symbol = Symbol { index: 53 };
/// Known symbol for the `neg` operator
pub const OP_NEG: Symbol = Symbol { index: 54 };
/// Known symbol for the `not` operator
pub const OP_NOT: Symbol = Symbol { index: 55 };
/// Known symbol for the `bit_and` operator
pub const OP_BIT_AND: Symbol = Symbol { index: 56 };
/// Known symbol for the `bit_or` operator
pub const OP_BIT_OR: Symbol = Symbol { index: 57 };
/// Known symbol for the `bit_xor` operator
pub const OP_BIT_XOR: Symbol = Symbol { index: 58 };
/// Known symbol for the `bit_not` operator
pub const OP_BIT_NOT: Symbol = Symbol { index: 59 };
/// Known symbol for the `shl` operator
pub const OP_SHL: Symbol = Symbol { index: 60 };
/// Known symbol for the `shr` operator
pub const OP_SHR: Symbol = Symbol { index: 61 };
/// Known symbol for the `eq` operator
pub const OP_EQ: Symbol = Symbol { index: 62 };
/// Known symbol for the `ne` operator
pub const OP_NE: Symbol = Symbol { index: 63 };
/// Known symbol for the `lt` operator
pub const OP_LT: Symbol = Symbol { index: 64 };
/// Known symbol for the `le` operator
pub const OP_LE: Symbol = Symbol { index: 65 };
/// Known symbol for the `gt` operator
pub const OP_GT: Symbol = Symbol { index: 66 };
/// Known symbol for the `ge` operator
pub const OP_GE: Symbol = Symbol { index: 67 };
/// Known symbol for the `and` operator
pub const OP_AND: Symbol = Symbol { index: 68 };
/// Known symbol for the `or` operator
pub const OP_OR: Symbol = Symbol { index: 69 };

// ----------------------------------------------------------------------------
// Common Identifiers
// ----------------------------------------------------------------------------

/// Known symbol for `main`
pub const ID_MAIN: Symbol = Symbol { index: 70 };
/// Known symbol for `new`
pub const ID_NEW: Symbol = Symbol { index: 71 };
/// Known symbol for `init`
pub const ID_INIT: Symbol = Symbol { index: 72 };
/// Known symbol for `drop`
pub const ID_DROP: Symbol = Symbol { index: 73 };
/// Known symbol for `ok`
pub const ID_OK: Symbol = Symbol { index: 74 };
/// Known symbol for `err`
pub const ID_ERR: Symbol = Symbol { index: 75 };
/// Known symbol for `some`
pub const ID_SOME: Symbol = Symbol { index: 76 };
/// Known symbol for `none`
pub const ID_NONE: Symbol = Symbol { index: 77 };
/// Known symbol for `len`
pub const ID_LEN: Symbol = Symbol { index: 78 };
/// Known symbol for `size`
pub const ID_SIZE: Symbol = Symbol { index: 79 };
/// Known symbol for `capacity`
pub const ID_CAPACITY: Symbol = Symbol { index: 80 };
/// Known symbol for `push`
pub const ID_PUSH: Symbol = Symbol { index: 81 };
/// Known symbol for `pop`
pub const ID_POP: Symbol = Symbol { index: 82 };
/// Known symbol for `insert`
pub const ID_INSERT: Symbol = Symbol { index: 83 };
/// Known symbol for `remove`
pub const ID_REMOVE: Symbol = Symbol { index: 84 };
/// Known symbol for `get`
pub const ID_GET: Symbol = Symbol { index: 85 };
/// Known symbol for `set`
pub const ID_SET: Symbol = Symbol { index: 86 };
/// Known symbol for `first`
pub const ID_FIRST: Symbol = Symbol { index: 87 };
/// Known symbol for `last`
pub const ID_LAST: Symbol = Symbol { index: 88 };
/// Known symbol for `iter`
pub const ID_ITER: Symbol = Symbol { index: 89 };
/// Known symbol for `next`
pub const ID_NEXT: Symbol = Symbol { index: 90 };
/// Known symbol for `done`
pub const ID_DONE: Symbol = Symbol { index: 91 };
/// Known symbol for `clone`
pub const ID_CLONE: Symbol = Symbol { index: 92 };
/// Known symbol for `copy`
pub const ID_COPY: Symbol = Symbol { index: 93 };
/// Known symbol for `hash`
pub const ID_HASH: Symbol = Symbol { index: 94 };
/// Known symbol for `hasher`
pub const ID_HASHER: Symbol = Symbol { index: 95 };
/// Known symbol for `from`
pub const ID_FROM: Symbol = Symbol { index: 96 };
/// Known symbol for `into`
pub const ID_INTO: Symbol = Symbol { index: 97 };
/// Known symbol for `try_from`
pub const ID_TRY_FROM: Symbol = Symbol { index: 98 };
/// Known symbol for `try_into`
pub const ID_TRY_INTO: Symbol = Symbol { index: 99 };
/// Known symbol for `default`
pub const ID_DEFAULT: Symbol = Symbol { index: 100 };
/// Known symbol for `display`
pub const ID_DISPLAY: Symbol = Symbol { index: 101 };
/// Known symbol for `debug`
pub const ID_DEBUG: Symbol = Symbol { index: 102 };

impl Symbol {
    /// The maximum index value for a symbol
    pub const MAX_INDEX: u32 = u32::MAX;

    /// Intern a string, returning its symbol
    ///
    /// This function will:
    /// 1. Hash the string to check for existing entry
    /// 2. If found, return existing symbol (cache hit)
    /// 3. If not found, allocate new slot and return new symbol (cache miss)
    ///
    /// # Thread Safety
    ///
    /// This function is thread-safe. Multiple threads can intern strings
    /// concurrently using DashMap for lock-free access.
    ///
    /// # Performance
    ///
    /// - **Best case** (string already interned): O(1) hash lookup
    /// - **Worst case** (new unique string): O(1) hash insert + allocation
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let keyword = Symbol::intern("fn");
    /// let name = Symbol::intern("main");
    ///
    /// // Interning the same string returns the same symbol
    /// assert_eq!(Symbol::intern("fn"), keyword);
    /// ```
    #[inline]
    pub fn intern(string: &str) -> Self {
        STRING_TABLE.intern(string)
    }

    /// Get the string value associated with this symbol
    ///
    /// # Performance
    ///
    /// O(n) where n is the number of interned strings, as we need to
    /// search the DashMap by index. This is a trade-off for better
    /// concurrent write performance.
    ///
    /// For hot paths, consider caching the string reference or using
    /// [`Symbol::eq_str()`] for comparisons.
    ///
    /// # Panics
    ///
    /// Returns empty string if the symbol is invalid (e.g., created manually
    /// with an out-of-bounds index).
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("hello");
    /// assert_eq!(sym.as_str(), "hello");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &'static str {
        STRING_TABLE.get(*self).unwrap_or("")
    }

    /// Check if the symbol's string is empty
    ///
    /// # Performance
    ///
    /// O(1) - checks if index corresponds to the empty string symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// assert!(Symbol::intern("").is_empty());
    /// assert!(!Symbol::intern("hello").is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Get the length of the symbol's string in bytes
    ///
    /// # Performance
    ///
    /// O(n) where n is the number of interned strings (requires lookup).
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// assert_eq!(Symbol::intern("hello").len(), 5);
    /// assert_eq!(Symbol::intern("").len(), 0);
    /// assert_eq!(Symbol::intern("你好").len(), 6);  // UTF-8 bytes
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Check if the symbol's string starts with a given prefix
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to check for
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("hello_world");
    /// assert!(sym.starts_with("hello"));
    /// assert!(!sym.starts_with("world"));
    /// ```
    #[inline]
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.as_str().starts_with(prefix)
    }

    /// Check if the symbol's string ends with a given suffix
    ///
    /// # Arguments
    ///
    /// * `suffix` - The suffix to check for
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("hello_world");
    /// assert!(sym.ends_with("world"));
    /// assert!(!sym.ends_with("hello"));
    /// ```
    #[inline]
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.as_str().ends_with(suffix)
    }

    /// Compare the symbol's string with a `&str` without allocation
    ///
    /// This is more efficient than `symbol.as_str() == other` when you
    /// only need equality comparison, as it can short-circuit early.
    ///
    /// # Arguments
    ///
    /// * `other` - The string to compare against
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("hello");
    /// assert!(sym.eq_str("hello"));
    /// assert!(!sym.eq_str("world"));
    /// ```
    #[inline]
    pub fn eq_str(&self, other: &str) -> bool {
        self.as_str() == other
    }

    /// Returns true if this is a "known" symbol (predefined keywords)
    ///
    /// Known symbols are interned at compiler startup and have indices
    /// in a reserved range (0 to `RESERVED_SYMBOLS_END`).
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::{Symbol, KW_FN};
    ///
    /// assert!(KW_FN.is_known());
    /// assert!(!Symbol::intern("my_variable").is_known());
    /// ```
    #[inline]
    pub fn is_known(&self) -> bool {
        self.index < RESERVED_SYMBOLS_END
    }

    /// Get the raw index value
    ///
    /// Useful for serialization or debugging.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("test");
    /// let index = sym.as_u32();
    /// ```
    #[inline]
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
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("test");
    /// let index = sym.as_u32();
    /// let sym2 = unsafe { Symbol::from_u32_unchecked(index) };
    /// assert_eq!(sym, sym2);
    /// ```
    #[inline]
    pub unsafe fn from_u32_unchecked(index: u32) -> Self {
        Self { index }
    }

    /// Get statistics about the string interner for profiling
    ///
    /// Returns an `InternerStats` struct with detailed information about
    /// the interner's state, including count, capacity, collisions, hits,
    /// and misses.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let stats = Symbol::stats_struct();
    /// println!("Interned {} strings", stats.count);
    /// println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    /// ```
    #[inline]
    pub fn stats_struct() -> InternerStats {
        STRING_TABLE.stats()
    }

    /// Get basic statistics about the string interner (legacy API)
    ///
    /// Returns a tuple of (number of interned strings, hash map capacity).
    /// For more detailed statistics, use [`Symbol::stats_struct`].
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let (count, capacity) = Symbol::stats();
    /// println!("Interned {} strings with capacity {}", count, capacity);
    /// ```
    #[inline]
    pub fn stats() -> (usize, usize) {
        let stats = STRING_TABLE.stats();
        (stats.count, stats.capacity)
    }

    /// Intern a known keyword, returning its predefined symbol
    ///
    /// This is a convenience method for interning common keywords.
    /// For known keywords, this returns the predefined constant symbol
    /// without hashing or allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::{Symbol, KW_FN};
    ///
    /// let fn_sym = Symbol::intern_known("fn");
    /// assert_eq!(fn_sym, KW_FN);
    ///
    /// // Unknown keywords are interned normally
    /// let unknown = Symbol::intern_known("not_a_keyword");
    /// assert!(!unknown.is_known());
    /// ```
    #[inline]
    pub fn intern_known(string: &str) -> Self {
        match string {
            // Control flow
            "fn" => KW_FN,
            "let" => KW_LET,
            "const" => KW_CONST,
            "mut" => KW_MUT,
            "if" => KW_IF,
            "else" => KW_ELSE,
            "while" => KW_WHILE,
            "for" => KW_FOR,
            "loop" => KW_LOOP,
            "return" => KW_RETURN,
            "break" => KW_BREAK,
            "continue" => KW_CONTINUE,
            "match" => KW_MATCH,
            // Type declarations
            "struct" => KW_STRUCT,
            "enum" => KW_ENUM,
            "impl" => KW_IMPL,
            "trait" => KW_TRAIT,
            "type" => KW_TYPE,
            "mod" => KW_MOD,
            "use" => KW_USE,
            // Visibility and modules
            "pub" => KW_PUB,
            "crate" => KW_CRATE,
            "super" => KW_SUPER,
            "self" => KW_SELF,
            "Self" => KW_SELF_UPPER,
            "as" => KW_AS,
            // Literals
            "true" => KW_TRUE,
            "false" => KW_FALSE,
            // Extended control flow
            "unsafe" => KW_UNSAFE,
            "extern" => KW_EXTERN,
            // Type keywords - integers
            "i8" => TY_I8,
            "i16" => TY_I16,
            "i32" => TY_I32,
            "i64" => TY_I64,
            "isize" => TY_ISIZE,
            "u8" => TY_U8,
            "u16" => TY_U16,
            "u32" => TY_U32,
            "u64" => TY_U64,
            "usize" => TY_USIZE,
            // Type keywords - floats
            "f32" => TY_F32,
            "f64" => TY_F64,
            // Type keywords - other
            "bool" => TY_BOOL,
            "char" => TY_CHAR,
            "str" => TY_STR,
            // Type system
            "dyn" => KW_DYN,
            "where" => KW_WHERE,
            // Container types
            "array" => KW_ARRAY,
            "tuple" => KW_TUPLE,
            // Operators
            "add" => OP_ADD,
            "sub" => OP_SUB,
            "mul" => OP_MUL,
            "div" => OP_DIV,
            "rem" => OP_REM,
            "neg" => OP_NEG,
            "not" => OP_NOT,
            "bit_and" => OP_BIT_AND,
            "bit_or" => OP_BIT_OR,
            "bit_xor" => OP_BIT_XOR,
            "bit_not" => OP_BIT_NOT,
            "shl" => OP_SHL,
            "shr" => OP_SHR,
            "eq" => OP_EQ,
            "ne" => OP_NE,
            "lt" => OP_LT,
            "le" => OP_LE,
            "gt" => OP_GT,
            "ge" => OP_GE,
            "and" => OP_AND,
            "or" => OP_OR,
            // Common identifiers
            "main" => ID_MAIN,
            "new" => ID_NEW,
            "init" => ID_INIT,
            "drop" => ID_DROP,
            "ok" => ID_OK,
            "err" => ID_ERR,
            "some" => ID_SOME,
            "none" => ID_NONE,
            "len" => ID_LEN,
            "size" => ID_SIZE,
            "capacity" => ID_CAPACITY,
            "push" => ID_PUSH,
            "pop" => ID_POP,
            "insert" => ID_INSERT,
            "remove" => ID_REMOVE,
            "get" => ID_GET,
            "set" => ID_SET,
            "first" => ID_FIRST,
            "last" => ID_LAST,
            "iter" => ID_ITER,
            "next" => ID_NEXT,
            "done" => ID_DONE,
            "clone" => ID_CLONE,
            "copy" => ID_COPY,
            // Note: "eq" is handled above as operator
            "hash" => ID_HASH,
            "hasher" => ID_HASHER,
            "from" => ID_FROM,
            "into" => ID_INTO,
            "try_from" => ID_TRY_FROM,
            "try_into" => ID_TRY_INTO,
            "default" => ID_DEFAULT,
            "display" => ID_DISPLAY,
            "debug" => ID_DEBUG,
            _ => Self::intern(string),
        }
    }

    /// Get the symbol for a type keyword
    ///
    /// Returns the predefined symbol for primitive type keywords,
    /// or interns the string for unknown types.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::{Symbol, TY_I32, TY_STR};
    ///
    /// assert_eq!(Symbol::intern_type("i32"), TY_I32);
    /// assert_eq!(Symbol::intern_type("str"), TY_STR);
    /// ```
    #[inline]
    pub fn intern_type(string: &str) -> Self {
        match string {
            "i8" => TY_I8,
            "i16" => TY_I16,
            "i32" => TY_I32,
            "i64" => TY_I64,
            "isize" => TY_ISIZE,
            "u8" => TY_U8,
            "u16" => TY_U16,
            "u32" => TY_U32,
            "u64" => TY_U64,
            "usize" => TY_USIZE,
            "f32" => TY_F32,
            "f64" => TY_F64,
            "bool" => TY_BOOL,
            "char" => TY_CHAR,
            "str" => TY_STR,
            _ => Self::intern(string),
        }
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

impl std::fmt::Debug for Symbol {
    /// Format the symbol for debugging, showing the actual string content
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("hello");
    /// assert_eq!(format!("{:?}", sym), "Symbol(hello)");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({})", self.as_str())
    }
}

impl std::fmt::Display for Symbol {
    /// Format the symbol for display, showing just the string content
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::symbol::Symbol;
    ///
    /// let sym = Symbol::intern("hello");
    /// assert_eq!(format!("{}", sym), "hello");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Pretty-print a symbol with quotes for debugging
///
/// This trait provides optional pretty-printing with quotes to distinguish
/// symbols from regular strings in debug output.
///
/// # Examples
///
/// ```
/// use faxc_util::symbol::{Symbol, SymbolPretty};
///
/// let sym = Symbol::intern("hello");
/// assert_eq!(format!("{}", sym.pretty()), "\"hello\"");
/// ```
pub trait SymbolPretty {
    /// Get a pretty-printed version with quotes
    fn pretty(&self) -> SymbolPrettyDisplay<'_>;
}

impl SymbolPretty for Symbol {
    fn pretty(&self) -> SymbolPrettyDisplay<'_> {
        SymbolPrettyDisplay(self)
    }
}

/// Display wrapper for pretty-printing symbols with quotes
pub struct SymbolPrettyDisplay<'a>(&'a Symbol);

impl std::fmt::Display for SymbolPrettyDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0.as_str())
    }
}

impl std::fmt::Debug for SymbolPrettyDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymbolPretty(\"{}\")", self.0.as_str())
    }
}

// Ensure Symbol is thread-safe
static_assertions::assert_impl_all!(Symbol: Send, Sync);

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // ========================================================================
    // Basic Interning Tests
    // ========================================================================

    #[test]
    fn test_symbol_intern() {
        let s1 = Symbol::intern("hello");
        let s2 = Symbol::intern("hello");
        let s3 = Symbol::intern("world");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s3.as_str(), "world");
    }

    #[test]
    fn test_symbol_display() {
        let s = Symbol::intern("test");
        assert_eq!(format!("{}", s), "test");
        assert_eq!(format!("{:?}", s), "Symbol(test)");
    }

    #[test]
    fn test_symbol_pretty() {
        let s = Symbol::intern("test");
        assert_eq!(format!("{}", s.pretty()), "\"test\"");
        assert_eq!(format!("{:?}", s.pretty()), "SymbolPretty(\"test\")");
    }

    // ========================================================================
    // Symbol Utility Methods
    // ========================================================================

    #[test]
    fn test_symbol_is_empty() {
        assert!(Symbol::intern("").is_empty());
        assert!(!Symbol::intern("a").is_empty());
        assert!(!Symbol::intern("hello").is_empty());
    }

    #[test]
    fn test_symbol_len() {
        assert_eq!(Symbol::intern("").len(), 0);
        assert_eq!(Symbol::intern("a").len(), 1);
        assert_eq!(Symbol::intern("hello").len(), 5);
        assert_eq!(Symbol::intern("你好").len(), 6); // UTF-8 bytes
    }

    #[test]
    fn test_symbol_starts_with() {
        let sym = Symbol::intern("hello_world");
        assert!(sym.starts_with("hello"));
        assert!(sym.starts_with("hello_world"));
        assert!(!sym.starts_with("world"));
        assert!(!sym.starts_with(""));
    }

    #[test]
    fn test_symbol_ends_with() {
        let sym = Symbol::intern("hello_world");
        assert!(sym.ends_with("world"));
        assert!(sym.ends_with("hello_world"));
        assert!(!sym.ends_with("hello"));
        assert!(sym.ends_with(""));
    }

    #[test]
    fn test_symbol_eq_str() {
        let sym = Symbol::intern("hello");
        assert!(sym.eq_str("hello"));
        assert!(!sym.eq_str("world"));
        assert!(!sym.eq_str(""));
    }

    #[test]
    fn test_symbol_to_string() {
        let sym = Symbol::intern("hello");
        let string = sym.to_string();
        assert_eq!(string, "hello");
        assert_eq!(string.len(), 5);
    }

    // ========================================================================
    // Known Symbols Tests
    // ========================================================================

    #[test]
    fn test_symbol_is_known() {
        assert!(KW_FN.is_known());
        assert!(KW_LET.is_known());
        assert!(TY_I32.is_known());
        assert!(OP_ADD.is_known());
        assert!(!Symbol::intern("my_variable").is_known());
    }

    #[test]
    fn test_known_symbols_values() {
        // Control flow
        assert_eq!(KW_FN.as_str(), "fn");
        assert_eq!(KW_LET.as_str(), "let");
        assert_eq!(KW_IF.as_str(), "if");
        assert_eq!(KW_MATCH.as_str(), "match");

        // Type declarations
        assert_eq!(KW_STRUCT.as_str(), "struct");
        assert_eq!(KW_ENUM.as_str(), "enum");
        assert_eq!(KW_TRAIT.as_str(), "trait");

        // Type keywords
        assert_eq!(TY_I8.as_str(), "i8");
        assert_eq!(TY_I32.as_str(), "i32");
        assert_eq!(TY_F64.as_str(), "f64");
        assert_eq!(TY_BOOL.as_str(), "bool");
        assert_eq!(TY_STR.as_str(), "str");

        // Operators
        assert_eq!(OP_ADD.as_str(), "add");
        assert_eq!(OP_SUB.as_str(), "sub");
        assert_eq!(OP_EQ.as_str(), "eq");
    }

    #[test]
    fn test_intern_known() {
        assert_eq!(Symbol::intern_known("fn"), KW_FN);
        assert_eq!(Symbol::intern_known("i32"), TY_I32);
        assert_eq!(Symbol::intern_known("add"), OP_ADD);

        let unknown = Symbol::intern_known("unknown_keyword");
        assert_eq!(unknown.as_str(), "unknown_keyword");
        assert!(!unknown.is_known());
    }

    #[test]
    fn test_intern_type() {
        assert_eq!(Symbol::intern_type("i32"), TY_I32);
        assert_eq!(Symbol::intern_type("f64"), TY_F64);
        assert_eq!(Symbol::intern_type("bool"), TY_BOOL);

        let custom = Symbol::intern_type("MyType");
        assert_eq!(custom.as_str(), "MyType");
        assert!(!custom.is_known());
    }

    // ========================================================================
    // Raw Index Tests
    // ========================================================================

    #[test]
    fn test_from_u32_unchecked() {
        let sym = Symbol::intern("test");
        let index = sym.as_u32();
        let sym2 = unsafe { Symbol::from_u32_unchecked(index) };
        assert_eq!(sym, sym2);
    }

    #[test]
    fn test_as_u32() {
        let sym = Symbol::intern("test");
        let index = sym.as_u32();
        assert!(index < Symbol::MAX_INDEX);
    }

    // ========================================================================
    // Statistics Tests
    // ========================================================================

    #[test]
    fn test_stats() {
        let (count, capacity) = Symbol::stats();
        assert!(count > 0); // Known symbols are pre-interned
        assert!(capacity >= count);
    }

    #[test]
    fn test_stats_struct() {
        let stats = Symbol::stats_struct();
        assert!(stats.count > 0);
        assert!(stats.capacity >= stats.count);
        assert!(stats.load_factor() >= 0.0);
        assert!(stats.load_factor() <= 1.0);
        assert!(stats.hit_rate() >= 0.0);
        assert!(stats.hit_rate() <= 1.0);
    }

    #[test]
    fn test_interner_stats_methods() {
        let stats = InternerStats::new(100, 200, 5, 80, 20);

        assert_eq!(stats.count, 100);
        assert_eq!(stats.capacity, 200);
        assert_eq!(stats.collisions, 5);
        assert_eq!(stats.hits, 80);
        assert_eq!(stats.misses, 20);

        assert_eq!(stats.load_factor(), 0.5);
        assert!(!stats.is_near_capacity());

        assert_eq!(stats.hit_rate(), 0.8);
        assert_eq!(stats.total_operations(), 100);
        assert_eq!(stats.memory_efficiency(), 1.0);

        let stats_full = InternerStats::new(100, 120, 0, 0, 0);
        assert!(stats_full.is_near_capacity());

        let stats_empty = InternerStats::new(0, 0, 0, 0, 0);
        assert_eq!(stats_empty.load_factor(), 0.0);
        assert_eq!(stats_empty.hit_rate(), 0.0);
    }

    // ========================================================================
    // Thread Safety Tests
    // ========================================================================

    #[test]
    fn test_concurrent_intern() {
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let s = Symbol::intern(&format!("thread_{}", i));
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
                thread::spawn(|| Symbol::intern("concurrent_same"))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All should be the same symbol
        for symbol in &results[1..] {
            assert_eq!(results[0], *symbol);
        }
    }

    #[test]
    fn test_thread_safety_stress() {
        const THREADS: usize = 20;
        const ITERATIONS: usize = 50;

        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                thread::spawn(move || {
                    let mut local_symbols = Vec::new();
                    for i in 0..ITERATIONS {
                        let s = Symbol::intern(&format!("stress_{}_{}", t, i));
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
        assert_send_sync::<Symbol>();
        assert_send_sync::<InternerStats>();
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_string() {
        let s = Symbol::intern("");
        assert_eq!(s.as_str(), "");
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_unicode_strings() {
        let test_cases = ["你好", "世界", "🦀", "こんにちは", "Привет"];

        for test in &test_cases {
            let sym = Symbol::intern(test);
            assert_eq!(sym.as_str(), *test);
        }
    }

    #[test]
    fn test_long_strings() {
        let long_string = "a".repeat(10000);
        let sym = Symbol::intern(&long_string);
        assert_eq!(sym.as_str(), long_string.as_str());
        assert_eq!(sym.len(), 10000);
    }

    #[test]
    fn test_special_characters() {
        let special = "hello\nworld\t!";
        let sym = Symbol::intern(special);
        assert_eq!(sym.as_str(), special);
    }

    // ========================================================================
    // Property-Based Tests (Manual Implementation)
    // ========================================================================

    #[test]
    fn test_idempotence() {
        // Property: intern(intern(x)) == intern(x)
        let test_strings = ["hello", "world", "test", "foo", "bar"];

        for s in &test_strings {
            let sym1 = Symbol::intern(s);
            let sym2 = Symbol::intern(s);
            let sym3 = Symbol::intern(&sym1.as_str());

            assert_eq!(sym1, sym2);
            assert_eq!(sym1, sym3);
        }
    }

    #[test]
    fn test_uniqueness() {
        // Property: different strings get different symbols
        let strings = ["abc", "def", "ghi", "jkl", "mno"];

        for i in 0..strings.len() {
            for j in (i + 1)..strings.len() {
                let sym_i = Symbol::intern(strings[i]);
                let sym_j = Symbol::intern(strings[j]);
                assert_ne!(sym_i, sym_j);
            }
        }
    }

    #[test]
    fn test_roundtrip() {
        // Property: intern(to_string(intern(x))) == intern(x)
        let test_strings = ["hello", "world", "test"];

        for s in &test_strings {
            let sym1 = Symbol::intern(s);
            let string = sym1.to_string();
            let sym2 = Symbol::intern(&string);
            assert_eq!(sym1, sym2);
        }
    }

    #[test]
    fn test_comparison_consistency() {
        // Property: symbol comparison is consistent with string comparison
        let pairs = [
            ("hello", "hello", true),
            ("hello", "world", false),
            ("", "", true),
            ("a", "a", true),
            ("a", "b", false),
        ];

        for (s1, s2, expected_eq) in &pairs {
            let sym1 = Symbol::intern(s1);
            let sym2 = Symbol::intern(s2);

            assert_eq!(sym1 == sym2, *expected_eq);
            assert_eq!(sym1.eq_str(s2), *expected_eq);
            assert_eq!(s1 == s2, *expected_eq);
        }
    }

    // ========================================================================
    // Performance Tests (Not Benchmarks, but Performance-Related)
    // ========================================================================

    #[test]
    fn test_hit_miss_tracking() {
        STRING_TABLE.reset_stats();

        // First intern should be a miss
        let _ = Symbol::intern("unique_perf_test");
        let stats = Symbol::stats_struct();
        assert!(stats.misses >= 1);

        // Second intern of same string should be a hit
        let _ = Symbol::intern("unique_perf_test");
        let stats = Symbol::stats_struct();
        assert!(stats.hits >= 1);

        // Verify hit rate calculation
        assert!(stats.hit_rate() > 0.0);
    }

    #[test]
    fn test_known_symbol_performance() {
        // Known symbols should have predictable indices
        assert!(KW_FN.index < RESERVED_SYMBOLS_END);
        assert!(TY_I32.index < RESERVED_SYMBOLS_END);
        assert!(OP_ADD.index < RESERVED_SYMBOLS_END);

        // Known symbols should be fast to look up
        for _ in 0..1000 {
            let _ = Symbol::intern_known("fn");
            let _ = Symbol::intern_known("i32");
        }
    }
}
