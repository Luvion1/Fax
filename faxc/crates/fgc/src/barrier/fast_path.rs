//! Fast Path - Inline Load Barrier for Performance
//!
//! Fast path is an inline barrier embedded directly in code generation.
//! Slow path (function call) is only invoked if fast path fails.
//!
//! Performance Considerations:
//! - Function marked `#[inline(always)]` to ensure inlining
//! - Minimal branches in fast path
//! - Atomic operations with relaxed ordering for performance
//! - Branch prediction friendly code layout
//!
//! Fast Path Logic:
//! ```
//! if (pointer == null) return true;           // Null check
//! if (mark_word & MARK_MASK != 0) return true; // Already marked
//! return false;                                // Need slow path
//! ```
//!
//! Slow Path:
//! - Function call to load_barrier_slow_path
//! - Enqueue object to mark queue
//! - Handle pointer healing if needed
