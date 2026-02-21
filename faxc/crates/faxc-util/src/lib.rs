//! faxc-util - Core Utilities and Foundation Types
//!
//! This crate provides fundamental utilities and types that form the foundation
//! of the entire faxc compiler infrastructure.
//!
//! # Modules
//!
//! - [`symbol`] - String interning for efficient identifier handling
//! - [`index_vec`] - Type-safe indexed vectors
//! - [`diagnostic`] - Error and warning reporting infrastructure
//! - [`span`] - Source location tracking
//!
//! # Example
//!
//! ```
//! use faxc_util::symbol::{Symbol, InternerStats};
//! use faxc_util::index_vec::{IndexVec, Idx};
//! use faxc_util::diagnostic::{Handler, DiagnosticBuilder, Span, DiagnosticCode};
//!
//! // Symbol interning
//! let sym = Symbol::intern("hello");
//! assert_eq!(sym.as_str(), "hello");
//!
//! // Symbol stats
//! let stats = Symbol::stats_struct();
//! println!("Interned {} strings", stats.count);
//!
//! // Typed index vector
//! #[derive(Clone, Copy, PartialEq, Eq)]
//! struct MyId(u32);
//! impl Idx for MyId {
//!     fn from_usize(i: usize) -> Self { MyId(i as u32) }
//!     fn index(self) -> usize { self.0 as usize }
//! }
//! let mut vec: IndexVec<MyId, i32> = IndexVec::new();
//! let idx = vec.push(42);
//! assert_eq!(vec[idx], 42);
//!
//! // Diagnostics with builder
//! let handler = Handler::new();
//! handler.build_error(Span::DUMMY, "error message")
//!     .code(DiagnosticCode::E0001)
//!     .emit(&handler);
//! ```

pub mod symbol;
pub mod index_vec;
pub mod diagnostic;
pub mod span;
pub mod def_id;

// Re-export commonly used types at crate root for convenience
pub use symbol::{Symbol, InternerStats, KW_FN, KW_LET, KW_CONST, KW_MUT, KW_IF, KW_ELSE, KW_WHILE, KW_FOR, KW_LOOP, KW_RETURN, KW_BREAK, KW_CONTINUE, KW_STRUCT, KW_ENUM, KW_IMPL, KW_TRAIT, KW_TYPE, KW_MOD, KW_USE, KW_PUB, KW_TRUE, KW_FALSE, KW_SELF, KW_SELF_UPPER, KW_AS, KW_MATCH, KW_UNSAFE, KW_EXTERN, KW_CRATE, KW_SUPER};
pub use index_vec::{Idx, IndexVec};
pub use def_id::{DefId, DefIdGenerator};
pub use diagnostic::{
    Handler, Diagnostic, Level, DiagnosticCode, DiagnosticBuilder, SourceSnippet,
    // Predefined diagnostic codes
    E0001, E0002, E0003, E0004, E0005,
    E_LEXER_UNEXPECTED_CHAR, E_LEXER_UNTERMINATED_STRING, E_LEXER_INVALID_NUMBER, E_LEXER_UNKNOWN_TOKEN,
    E_PARSER_UNEXPECTED_TOKEN, E_PARSER_EXPECTED_TOKEN, E_PARSER_UNEXPECTED_EOF, E_PARSER_DUPLICATE_DEF,
    E_SEMANTIC_TYPE_MISMATCH, E_SEMANTIC_UNDEFINED_VAR, E_SEMANTIC_UNDEFINED_FN, E_SEMANTIC_MUT_REQUIRED,
    W0001, W0002, W0003,
    W_UNUSED_VARIABLE, W_UNUSED_FUNCTION, W_DEAD_CODE,
};
pub use span::{FileId, Span, SourceFile, SourceMap};

// Re-export from external crates
pub use rustc_hash::{FxHashMap, FxHashSet};

// Note: define_idx macro is available at crate root due to #[macro_export]
