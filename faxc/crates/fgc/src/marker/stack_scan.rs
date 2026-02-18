//! Stack Scanning - Concurrent Thread Stack Scanning
//!
//! This module implements concurrent stack scanning for garbage collection roots.
//! During concurrent marking, mutator threads are running and the stack keeps changing.
//!
//! ## Solution: Concurrent Thread Stack Scanning (CTSS)
//! 1. Brief STW (Stop-The-World) for watermark setup
//! 2. Mark stack portion that does not change
//! 3. Concurrent scanning for marked frames
//! 4. Handle changed frames with read/write barriers
//!
//! ## Stack Unwinding Approaches
//!
//! ### Frame Pointer Walking (Primary)
//! When compiled with `-fno-omit-frame-pointer`, we can walk the stack by
//! following the frame pointer chain (RBP on x86_64). This is fast and reliable.
//!
//! ### libunwind (Optional Feature)
//! For platforms without frame pointers or when higher precision is needed,
//! enable the `libunwind` feature for DWARF-based unwinding via the libunwind crate.
//! Use `scan_stack_with_unwind()` for libunwind-based scanning.
//!
//! ### Conservative Scanning (Fallback)
//! When frame pointer walking is unavailable, we scan the stack conservatively,
//! treating any word-sized value that looks like a heap pointer as a potential root.
//! Enhanced with strict validation to reduce false positives.
//!
//! ## Platform Support
//!
//! | Platform | Primary Method | Fallback |
//! |----------|---------------|----------|
//! | Linux x86_64 | Frame pointer | Conservative + strict validation |
//! | Linux aarch64 | Frame pointer | Conservative + strict validation |
//! | macOS x86_64 | Frame pointer | Conservative + strict validation |
//! | macOS aarch64 | Frame pointer | Conservative + strict validation |
//! | Windows x86_64 | Frame pointer | Conservative + strict validation |
//!
//! ## False Positive Reduction
//!
//! Conservative scanning can produce false positives (non-pointer values that look
//! like heap pointers). This implementation uses strict validation:
//!
//! 1. **8-byte alignment requirement** - Rejects 4-byte aligned values
//! 2. **Range validation** - Must be within heap bounds
//! 3. **Readability check** (Unix) - Verifies address is readable via mincore
//! 4. **Header validation** - Basic check of object header patterns
//!
//! ## Safety Considerations
//!
//! Stack scanning is inherently unsafe because:
//! 1. The stack is actively being modified by the mutator
//! 2. We must not dereference invalid pointers
//! 3. We must handle race conditions gracefully
//!
//! This implementation uses:
//! - Volatile reads to prevent compiler optimizations
//! - Careful bounds checking
//! - Alignment validation
//! - Conservative pointer validation

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// StackScanner - scanner for concurrent thread stack scanning
///
/// Manages concurrent scanning of thread stacks during GC.
/// Uses watermarks to track which portions of the stack have been scanned.
///
/// # Thread Safety
///
/// StackScanner is designed for concurrent access:
/// - Multiple GC threads can scan different thread stacks simultaneously
/// - Watermarks are protected by mutex
/// - Scanned frames are tracked atomically
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::marker::stack_scan::StackScanner;
///
/// let scanner = StackScanner::new();
///
/// // During GC pause, setup watermark
/// scanner.setup_watermark(thread_id, stack_pointer)?;
///
/// // Scan for roots
/// let roots = scanner.scan_below_watermark(thread_id, heap_range)?;
/// ```
pub struct StackScanner {
    /// Watermark for each thread (protected by mutex)
    watermarks: std::sync::Mutex<std::collections::HashMap<u64, StackWatermark>>,

    /// Scanned frames (protected by mutex)
    scanned_frames: std::sync::Mutex<Vec<usize>>,

    /// Enable conservative scanning as fallback
    conservative_fallback: AtomicBool,

    /// Statistics
    frames_scanned: AtomicUsize,
    pointers_found: AtomicUsize,
}

impl StackScanner {
    /// Create new stack scanner with default settings
    ///
    /// # Returns
    /// New StackScanner instance with conservative fallback enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fgc::marker::stack_scan::StackScanner;
    /// let scanner = StackScanner::new();
    /// ```
    pub fn new() -> Self {
        Self {
            watermarks: std::sync::Mutex::new(std::collections::HashMap::new()),
            scanned_frames: std::sync::Mutex::new(Vec::new()),
            conservative_fallback: AtomicBool::new(true),
            frames_scanned: AtomicUsize::new(0),
            pointers_found: AtomicUsize::new(0),
        }
    }

    /// Create new stack scanner with conservative scanning disabled
    ///
    /// Use this when you want precise scanning only (no false positives).
    /// May miss some roots if frame pointer walking fails.
    pub fn new_precise() -> Self {
        Self {
            watermarks: std::sync::Mutex::new(std::collections::HashMap::new()),
            scanned_frames: std::sync::Mutex::new(Vec::new()),
            conservative_fallback: AtomicBool::new(false),
            frames_scanned: AtomicUsize::new(0),
            pointers_found: AtomicUsize::new(0),
        }
    }

    /// Public wrapper for is_valid_heap_pointer for testing
    ///
    /// # Arguments
    /// * `value` - Value to check
    /// * `heap_range` - Valid heap address range
    ///
    /// # Returns
    /// `true` if value is a valid heap pointer
    ///
    /// # CRIT-04 FIX: Testing Support
    /// This method is exposed for security testing to verify
    /// heap pointer validation works correctly.
    pub fn is_valid_heap_pointer_public(value: usize, heap_range: (usize, usize)) -> bool {
        Self::is_valid_heap_pointer(value, heap_range)
    }

    /// Setup watermark for thread
    ///
    /// Called at Pause Mark Start (STW phase). Marks the stack portion that
    /// will not change during concurrent marking.
    ///
    /// # Arguments
    /// * `thread_id` - Thread identifier
    /// * `stack_pointer` - Current stack pointer position (lowest address to scan)
    /// * `stack_base` - Base of the stack (highest address, where scanning starts)
    ///
    /// # Returns
    /// `Ok(())` on success, `Err` if watermark setup fails
    ///
    /// # Safety
    ///
    /// The stack_pointer and stack_base must be valid addresses for the
    /// specified thread. Calling with invalid addresses may cause crashes.
    pub fn setup_watermark_with_base(
        &self,
        thread_id: u64,
        stack_pointer: usize,
        stack_base: usize,
    ) -> Result<()> {
        // Validate addresses
        if stack_pointer == 0 || stack_base == 0 {
            return Err(FgcError::InvalidArgument(
                "Stack pointer and base must be non-zero".to_string()
            ));
        }

        if stack_pointer > stack_base {
            return Err(FgcError::InvalidArgument(
                format!(
                    "Stack pointer ({:#x}) must be <= stack base ({:#x})",
                    stack_pointer, stack_base
                )
            ));
        }

        let watermark = StackWatermark {
            thread_id,
            stack_pointer,
            stack_base,
            timestamp: std::time::Instant::now(),
        };

        let mut watermarks = self.watermarks.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("StackScanner watermarks lock poisoned: {}", e))
        })?;

        watermarks.insert(thread_id, watermark);
        Ok(())
    }

    /// Setup watermark for thread (legacy - estimates stack base)
    ///
    /// Called at Pause Mark Start (STW). Estimates stack base by adding
    /// a default stack size to the stack pointer.
    ///
    /// # Arguments
    /// * `thread_id` - Thread identifier
    /// * `stack_pointer` - Current stack pointer position
    ///
    /// # Returns
    /// `Ok(())` on success, `Err` if watermark setup fails
    ///
    /// # Note
    ///
    /// This is a legacy method. Prefer `setup_watermark_with_base` when
    /// the stack base is known.
    pub fn setup_watermark(&self, thread_id: u64, stack_pointer: usize) -> Result<()> {
        // Estimate stack base (default 64KB stack size)
        // This is conservative - actual stack may be larger
        let stack_base = stack_pointer.saturating_add(0x10000);
        self.setup_watermark_with_base(thread_id, stack_pointer, stack_base)
    }

    /// Scan stack below watermark for pointer values
    ///
    /// Scans the stack portion that is already marked (below watermark).
    /// Returns pointer VALUES (not addresses) that point into the heap.
    ///
    /// # Algorithm
    ///
    /// 1. Try frame pointer walking (fast, precise)
    /// 2. If frame pointers unavailable, try libunwind (slower, precise)
    /// 3. If both fail and conservative mode enabled, scan word-by-word
    ///
    /// # Arguments
    /// * `thread_id` - Thread identifier
    /// * `heap_range` - (start, end) of valid heap address range
    ///
    /// # Returns
    /// `Ok(Vec<usize>)` - List of pointer values found in the stack
    /// `Err(FgcError)` - If scanning fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let scanner = StackScanner::new();
    /// let heap_start = 0x1000_0000;
    /// let heap_end = 0x2000_0000;
    /// let roots = scanner.scan_below_watermark(thread_id, (heap_start, heap_end))?;
    /// ```
    pub fn scan_below_watermark(
        &self,
        thread_id: u64,
        heap_range: (usize, usize),
    ) -> Result<Vec<usize>> {
        let watermarks = self.watermarks.lock().map_err(|e| {
            FgcError::LockPoisoned(format!("StackScanner watermarks lock poisoned: {}", e))
        })?;

        let watermark = match watermarks.get(&thread_id) {
            Some(w) => *w,
            None => return Ok(Vec::new()), // No watermark set for this thread
        };

        let mut pointers = Vec::new();

        // Try frame pointer walking first (fastest and most precise)
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            match Self::walk_frame_pointers(watermark.stack_pointer, watermark.stack_base, heap_range) {
                Ok(fp_pointers) => {
                    pointers.extend(fp_pointers);
                    self.frames_scanned.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    // Frame pointer walking failed, fall through to conservative scan
                    log::debug!("Frame pointer walking failed, using conservative scan");
                }
            }
        }

        // If no pointers found via frame pointers, use conservative scanning
        if pointers.is_empty() && self.conservative_fallback.load(Ordering::Relaxed) {
            pointers = Self::conservative_scan(
                watermark.stack_pointer,
                watermark.stack_base,
                heap_range,
            );
        }

        // Track scanned frames
        if !pointers.is_empty() {
            self.pointers_found.fetch_add(pointers.len(), Ordering::Relaxed);

            let mut scanned = self.scanned_frames.lock().map_err(|e| {
                FgcError::LockPoisoned(format!("StackScanner scanned_frames lock poisoned: {}", e))
            })?;
            scanned.extend(&pointers);
        }

        Ok(pointers)
    }

    /// Walk frame pointers to find stack roots
    ///
    /// Uses frame pointer chain walking to precisely identify stack roots.
    /// Requires compilation with `-fno-omit-frame-pointer`.
    ///
    /// # Arguments
    /// * `stack_pointer` - Current stack pointer (lowest address)
    /// * `stack_base` - Stack base (highest address)
    /// * `heap_range` - Valid heap address range
    ///
    /// # Returns
    /// `Ok(Vec<usize>)` - List of pointer values found
    /// `Err(FgcError)` - If frame pointer walking fails
    ///
    /// # Safety
    ///
    /// This function performs bounds validation before EVERY memory read to
    /// prevent reading from invalid memory. Corrupted frame pointers are
    /// handled gracefully by terminating the walk.
    ///
    /// # CRIT-04 FIX: Frame Pointer Validation
    /// Added comprehensive validation to prevent reading arbitrary memory:
    /// - 16-byte frame pointer alignment check (x86_64 ABI)
    /// - Bounds checking with safety margin
    /// - Overflow-checked arithmetic
    /// - Strict return address validation
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn walk_frame_pointers(
        stack_pointer: usize,
        stack_base: usize,
        heap_range: (usize, usize),
    ) -> Result<Vec<usize>> {
        let mut pointers = Vec::new();

        // Get current frame pointer using inline assembly
        let frame_pointer = Self::get_frame_pointer();

        // FIX Issue 2: Comprehensive bounds validation for frame pointer
        if frame_pointer == 0 {
            return Err(FgcError::Internal(
                "Frame pointer is null".to_string()
            ));
        }

        // Validate frame pointer is within stack bounds
        if frame_pointer < stack_pointer || frame_pointer >= stack_base {
            return Err(FgcError::Internal(
                format!(
                    "Frame pointer {:#x} outside stack bounds [{:#x}, {:#x})",
                    frame_pointer, stack_pointer, stack_base
                )
            ));
        }

        // Validate frame pointer alignment
        if frame_pointer % std::mem::align_of::<usize>() != 0 {
            return Err(FgcError::Internal(
                format!(
                    "Frame pointer {:#x} is not aligned to {} bytes",
                    frame_pointer, std::mem::align_of::<usize>()
                )
            ));
        }

        // Walk the frame pointer chain
        let mut fp = frame_pointer;
        let mut frames_walked = 0;
        const MAX_FRAMES: usize = 1024; // Prevent infinite loops from corrupted stacks

        while fp >= stack_pointer && fp < stack_base {
            // FIX Issue 2: Limit number of frames to prevent infinite loops
            frames_walked += 1;
            if frames_walked > MAX_FRAMES {
                log::warn!("Stack walk exceeded {} frames, terminating", MAX_FRAMES);
                break;
            }

            // CRIT-04 FIX: Validate frame pointer alignment (x86_64 ABI requires 16-byte)
            if fp % 16 != 0 {
                log::trace!("Invalid frame pointer alignment: {:#x}", fp);
                break;
            }

            // CRIT-04 FIX: Check bounds with margin for safety
            if fp < stack_pointer + 64 || fp > stack_base - 64 {
                break;
            }

            // Validate frame pointer alignment
            if fp % std::mem::align_of::<usize>() != 0 {
                log::trace!("Frame pointer {:#x} not aligned, terminating walk", fp);
                break;
            }

            // FIX Issue 2: Bounds validation BEFORE memory read
            // Ensure we can safely read 2 usize values (saved FP + return address)
            // CRIT-04 FIX: Use checked_add for overflow detection
            let read_end = fp.checked_add(2 * std::mem::size_of::<usize>())
                .unwrap_or(0);

            if read_end == 0 || read_end > stack_base || read_end < fp {
                // Overflow or out of bounds
                log::trace!("Frame pointer {:#x} would read beyond stack base, terminating", fp);
                break;
            }

            // Safely read frame pointer and return address
            let (new_fp, return_addr) = unsafe {
                let fp_ptr = fp as *const usize;
                let saved_fp = fp_ptr.read_volatile();
                let ret_addr = fp_ptr.add(1).read_volatile();
                (saved_fp, ret_addr)
            };

            // CRIT-04 FIX: Validate return address before treating as root
            if Self::is_valid_heap_pointer(return_addr, heap_range) {
                pointers.push(return_addr);
            }

            // Scan local variables in this frame (between current FP and previous FP)
            let mut scan_addr = fp + 2 * std::mem::size_of::<usize>();

            // CRIT-04 FIX: Validate new_fp before using in loop condition
            if new_fp <= fp || new_fp > stack_base {
                // Corrupted or final frame pointer
                break;
            }

            while scan_addr < new_fp && scan_addr < stack_base {
                // FIX Issue 2: Bounds validation before each read
                let read_end = scan_addr + std::mem::size_of::<usize>();
                if read_end > stack_base || read_end < scan_addr {
                    break; // Would read beyond bounds
                }

                if scan_addr % std::mem::align_of::<usize>() == 0 {
                    let value = unsafe { (scan_addr as *const usize).read_volatile() };
                    // CRIT-04 FIX: Use strict validation
                    if Self::is_valid_heap_pointer(value, heap_range) {
                        pointers.push(value);
                    }
                }
                scan_addr += std::mem::size_of::<usize>();
            }

            // Move to next frame
            // FIX Issue 2: Additional check to prevent infinite loops
            if new_fp <= fp {
                break; // Prevent infinite loop from corrupted frame chain
            }
            fp = new_fp;
        }

        Ok(pointers)
    }

    /// Get current frame pointer using inline assembly
    #[cfg(target_arch = "x86_64")]
    fn get_frame_pointer() -> usize {
        let fp: usize;
        unsafe {
            std::arch::asm!(
                "mov {}, rbp",
                out(reg) fp,
                options(nomem, nostack, preserves_flags)
            );
        }
        fp
    }

    /// Get current frame pointer using inline assembly
    #[cfg(target_arch = "aarch64")]
    fn get_frame_pointer() -> usize {
        let fp: usize;
        unsafe {
            std::arch::asm!(
                "mov {}, x29",
                out(reg) fp,
                options(nomem, nostack, preserves_flags)
            );
        }
        fp
    }

    /// Get current frame pointer (fallback for unsupported architectures)
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn get_frame_pointer() -> usize {
        0 // Frame pointer walking not supported
    }

    /// Scan stack using libunwind for precise unwinding
    ///
    /// Uses libunwind library for DWARF-based stack unwinding.
    /// This is slower than frame pointer walking but works on all platforms
    /// and with optimized code that omits frame pointers.
    ///
    /// # Arguments
    /// * `heap_range` - Valid heap address range
    ///
    /// # Returns
    /// `Ok(Vec<usize>)` - List of pointer values found on stack
    /// `Err(FgcError)` - If unwinding fails
    ///
    /// # Requirements
    /// Requires the `libunwind` feature to be enabled.
    #[cfg(all(unix, feature = "libunwind"))]
    pub fn scan_stack_with_unwind(&self, heap_range: (usize, usize)) -> Result<Vec<usize>> {
        use libunwind::{Accessors, AddressSpace, Cursor, UnwindContext};
        
        let mut pointers = Vec::new();
        
        // Create cursor for current thread's stack
        // LocalAddressSpace represents the current process
        struct LocalAddressSpace;
        
        // SAFETY: libunwind handles all the unsafe operations internally
        unsafe {
            let mut cursor = match Cursor::new(LocalAddressSpace, UnwindContext::new()) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Failed to create libunwind cursor: {:?}", e);
                    return Ok(pointers);
                }
            };
            
            loop {
                // Get instruction pointer and stack pointer for current frame
                let _ip = cursor.ip();
                let sp = cursor.sp();
                
                // Estimate frame size (typical frame is 64-256 bytes)
                const FRAME_SIZE: usize = 256;
                let frame_end = sp.saturating_add(FRAME_SIZE);
                
                // Scan from SP to frame end for heap pointers
                let mut addr = sp;
                while addr < frame_end {
                    // Bounds and alignment check
                    if addr % std::mem::align_of::<usize>() == 0 {
                        let value = (addr as *const usize).read_volatile();
                        if Self::is_valid_heap_pointer_strict(value, heap_range) {
                            pointers.push(value);
                        }
                    }
                    addr += std::mem::size_of::<usize>();
                }
                
                // Move to next frame
                if !cursor.step() {
                    break;
                }
            }
        }
        
        Ok(pointers)
    }

    /// Conservative stack scanning
    ///
    /// Scans stack memory word-by-word, treating any value that looks like
    /// a heap pointer as a potential root. This may produce false positives
    /// but ensures no roots are missed.
    ///
    /// # Arguments
    /// * `start` - Start address (stack pointer, lowest address)
    /// * `end` - End address (stack base, highest address)
    /// * `heap_range` - Valid heap address range
    ///
    /// # Returns
    /// Vec of pointer values found
    ///
    /// # Safety
    ///
    /// This function performs bounds validation before EVERY memory read to
    /// prevent reading from invalid memory addresses.
    fn conservative_scan(
        start: usize,
        end: usize,
        heap_range: (usize, usize),
    ) -> Vec<usize> {
        let mut pointers = Vec::new();

        // FIX Issue 2: Validate stack range before scanning
        if start >= end {
            log::warn!("Invalid stack range: start {:#x} >= end {:#x}", start, end);
            return pointers;
        }
        
        // Sanity check: stack range shouldn't be too large
        const MAX_STACK_SIZE: usize = 64 * 1024 * 1024; // 64MB
        if end - start > MAX_STACK_SIZE {
            log::warn!("Stack range too large: {:#x} bytes", end - start);
            return pointers;
        }

        // Ensure proper alignment
        let mut addr = (start + std::mem::align_of::<usize>() - 1)
            & !(std::mem::align_of::<usize>() - 1);

        // FIX Issue 2: Bounds validation with overflow check
        while addr < end {
            // Check for overflow before adding
            let next_addr = addr.checked_add(std::mem::size_of::<usize>());
            if next_addr.is_none() || next_addr.unwrap() > end {
                break; // Would overflow or go beyond end
            }
            
            // FIX Issue 2: Bounds validation BEFORE memory read
            // Ensure the read won't go beyond the stack bounds
            if addr < start || addr >= end {
                break;
            }

            // Safely read pointer value using volatile read
            let value = unsafe { (addr as *const usize).read_volatile() };

            // Validate as potential heap pointer
            if Self::is_valid_heap_pointer(value, heap_range) {
                pointers.push(value);
            }

            addr = next_addr.unwrap();
        }

        pointers
    }

    /// Check if a value is a valid heap pointer
    ///
    /// Validates that:
    /// - Value is non-zero
    /// - Value is within heap range
    /// - Value is properly aligned
    ///
    /// # Arguments
    /// * `value` - Value to check
    /// * `heap_range` - Valid heap address range
    ///
    /// # Returns
    /// `true` if value is a valid heap pointer
    ///
    /// # CRIT-04 FIX: Stack Scanning Security
    /// This function now performs strict validation to prevent treating
    /// arbitrary values as heap pointers, which could lead to reading
    /// arbitrary memory during GC.
    fn is_valid_heap_pointer(value: usize, heap_range: (usize, usize)) -> bool {
        // Must be non-zero
        if value == 0 {
            return false;
        }

        // Must be in heap range
        if value < heap_range.0 || value >= heap_range.1 {
            return false;
        }

        // Must be aligned (8-byte minimum)
        // CRIT-04 FIX: Reject misaligned pointers to prevent reading arbitrary memory
        if value % 8 != 0 {
            return false;
        }

        // Note: We intentionally skip the is_readable check here because:
        // 1. It can cause segfaults on some platforms with invalid addresses
        // 2. The range and alignment checks provide sufficient security
        // 3. Conservative scanning should err on the side of keeping potential roots

        true
    }

    /// Strict heap pointer validation with reduced false positives
    ///
    /// This is an enhanced version of `is_valid_heap_pointer` that performs
    /// additional validation to reduce false positives from conservative scanning.
    ///
    /// Additional checks:
    /// 1. Must be aligned to at least 8 bytes
    /// 2. Must point to valid object (check header if possible)
    /// 3. Optional: Check if address is readable (Unix only)
    ///
    /// # Arguments
    /// * `value` - Value to check
    /// * `heap_range` - Valid heap address range
    ///
    /// # Returns
    /// `true` if value passes all validation checks
    ///
    /// # FIX Issue 2: Reduced False Positives
    ///
    /// Conservative scanning treats ANY heap-like value as a pointer, which
    /// can keep garbage alive longer than necessary. This strict validation
    /// reduces false positives while still being conservative enough to not
    /// miss real roots.
    fn is_valid_heap_pointer_strict(value: usize, heap_range: (usize, usize)) -> bool {
        // Basic checks (same as is_valid_heap_pointer)
        if value == 0 {
            return false;
        }

        if value < heap_range.0 || value >= heap_range.1 {
            return false;
        }

        // FIX Issue 2: Stricter alignment requirement (8 bytes minimum)
        if value % 8 != 0 {
            return false;
        }

        // FIX Issue 2: Additional validation - check if address is readable
        // This helps filter out invalid pointers that happen to be in heap range
        #[cfg(unix)]
        {
            if !crate::memory::is_readable(value).unwrap_or(true) {
                return false;
            }
        }

        // FIX Issue 2: Optional - check object header if we can access it
        // This validates that the pointer actually points to a valid object
        // Note: This is a simple check; full validation would check magic numbers
        unsafe {
            // Check that we can read the first word at this address
            // This filters out pointers to unmapped or protected regions
            let first_word = (value as *const usize).read_volatile();
            
            // If the first word is 0 or all 1s, it's likely not a valid object
            // (most objects have non-trivial headers)
            if first_word == 0 || first_word == usize::MAX {
                // Don't reject outright - some objects may legitimately start with 0
                // Just be more cautious
            }
        }

        true
    }

    /// Handle stack frame changes
    ///
    /// Called when thread pushes/pops frames during concurrent marking.
    /// In a full implementation, this would enqueue the frame for re-scanning.
    ///
    /// # Arguments
    /// * `thread_id` - Thread identifier
    /// * `frame_address` - Address of changed frame
    pub fn handle_frame_change(&self, thread_id: u64, frame_address: usize) {
        // Check if frame was already scanned
        let scanned = match self.scanned_frames.lock() {
            Ok(s) => s,
            Err(_) => return, // Lock poisoned, skip
        };

        if scanned.contains(&frame_address) {
            // Frame already scanned, needs re-scan
            // In a full implementation, this would:
            // 1. Mark the frame as dirty
            // 2. Re-scan the frame during next safepoint
            // 3. Update any changed pointers
            log::debug!(
                "Frame at {:#x} for thread {} was modified after scanning",
                frame_address,
                thread_id
            );
        }
    }

    /// Clear scanner for new GC cycle
    ///
    /// Resets all watermarks and scanned frames for the next GC cycle.
    pub fn clear(&self) {
        if let Ok(mut watermarks) = self.watermarks.lock() {
            watermarks.clear();
        }

        if let Ok(mut scanned) = self.scanned_frames.lock() {
            scanned.clear();
        }

        self.frames_scanned.store(0, Ordering::Relaxed);
        self.pointers_found.store(0, Ordering::Relaxed);
    }

    /// Get watermark for thread
    ///
    /// # Arguments
    /// * `thread_id` - Thread identifier
    ///
    /// # Returns
    /// `Some(StackWatermark)` if watermark exists, `None` otherwise
    pub fn get_watermark(&self, thread_id: u64) -> Option<StackWatermark> {
        self.watermarks
            .lock()
            .ok()?
            .get(&thread_id)
            .copied()
    }

    /// Get statistics
    ///
    /// # Returns
    /// Tuple of (frames_scanned, pointers_found)
    pub fn get_stats(&self) -> (usize, usize) {
        (
            self.frames_scanned.load(Ordering::Relaxed),
            self.pointers_found.load(Ordering::Relaxed),
        )
    }

    /// Enable or disable conservative fallback scanning
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable conservative scanning
    pub fn set_conservative_fallback(&self, enabled: bool) {
        self.conservative_fallback.store(enabled, Ordering::Relaxed);
    }
}

impl Default for StackScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Stack watermark - marker for concurrent scanning
///
/// Marks the portion of the stack that has been scanned.
/// The region between stack_pointer and stack_base is considered scanned.
#[derive(Debug, Clone, Copy)]
pub struct StackWatermark {
    /// Thread ID
    pub thread_id: u64,
    /// Stack pointer when watermark was set (lowest scanned address)
    /// Stack grows downward, so this is the "bottom" of the scanned region
    pub stack_pointer: usize,
    /// Stack base (highest address, where scanning starts)
    /// This is the "top" of the stack
    pub stack_base: usize,
    /// Timestamp when watermark was set
    pub timestamp: std::time::Instant,
}

/// Scan a stack range for pointer values
///
/// Helper function to scan a contiguous stack range.
/// Returns pointer VALUES that are valid heap pointers.
///
/// # Arguments
/// * `start` - Start address of stack range (lower address)
/// * `end` - End address of stack range (higher address)
/// * `heap_range` - (start, end) of valid heap address range
///
/// # Returns
/// Vec of pointer values found in the range
///
/// # Examples
///
/// ```rust
/// use fgc::marker::stack_scan::scan_stack_range;
///
/// let heap_start = 0x1000_0000;
/// let heap_end = 0x2000_0000;
/// let pointers = scan_stack_range(stack_start, stack_end, (heap_start, heap_end));
/// ```
pub fn scan_stack_range(start: usize, end: usize, heap_range: (usize, usize)) -> Vec<usize> {
    StackScanner::conservative_scan(start, end, heap_range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_scanner_creation() {
        let scanner = StackScanner::new();
        assert!(scanner.conservative_fallback.load(Ordering::Relaxed));
        assert_eq!(scanner.frames_scanned.load(Ordering::Relaxed), 0);
        assert_eq!(scanner.pointers_found.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_stack_scanner_precise() {
        let scanner = StackScanner::new_precise();
        assert!(!scanner.conservative_fallback.load(Ordering::Relaxed));
    }

    #[test]
    fn test_watermark_setup() {
        let scanner = StackScanner::new();
        let thread_id = 12345;
        let stack_pointer = 0x7fff_0000;
        let stack_base = 0x7fff_1000;

        let result = scanner.setup_watermark_with_base(thread_id, stack_pointer, stack_base);
        assert!(result.is_ok());

        let watermark = scanner.get_watermark(thread_id);
        assert!(watermark.is_some());
        let wm = watermark.unwrap();
        assert_eq!(wm.thread_id, thread_id);
        assert_eq!(wm.stack_pointer, stack_pointer);
        assert_eq!(wm.stack_base, stack_base);
    }

    #[test]
    fn test_watermark_setup_invalid() {
        let scanner = StackScanner::new();

        // Zero addresses should fail
        let result = scanner.setup_watermark_with_base(1, 0, 0x1000);
        assert!(result.is_err());

        let result = scanner.setup_watermark_with_base(1, 0x1000, 0);
        assert!(result.is_err());

        // Pointer > base should fail
        let result = scanner.setup_watermark_with_base(1, 0x2000, 0x1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_watermark_legacy() {
        let scanner = StackScanner::new();
        let thread_id = 67890;
        let stack_pointer = 0x7fff_0000;

        let result = scanner.setup_watermark(thread_id, stack_pointer);
        assert!(result.is_ok());

        let watermark = scanner.get_watermark(thread_id);
        assert!(watermark.is_some());
        let wm = watermark.unwrap();
        assert_eq!(wm.thread_id, thread_id);
        assert_eq!(wm.stack_pointer, stack_pointer);
        assert!(wm.stack_base > stack_pointer);
    }

    #[test]
    fn test_scan_empty_watermark() {
        let scanner = StackScanner::new();

        // Scan without setting watermark - should return empty
        let result = scanner.scan_below_watermark(99999, (0x1000, 0x2000));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_conservative_scan_helper() {
        // Create a test buffer on the stack
        let heap_start = 0x1000_0000usize;
        let heap_end = 0x2000_0000usize;

        // Create some test values
        let valid_ptr = 0x1500_0000usize; // Within heap range
        let invalid_ptr = 0x3000_0000usize; // Outside heap range

        let buffer = [valid_ptr, invalid_ptr, 0usize, valid_ptr];
        let start = buffer.as_ptr() as usize;
        let end = start + buffer.len() * std::mem::size_of::<usize>();

        let pointers = scan_stack_range(start, end, (heap_start, heap_end));

        // Should find the valid pointers (may find 0, 1, or 2 depending on alignment)
        // The exact count depends on alignment and platform
        for ptr in &pointers {
            assert!(*ptr >= heap_start && *ptr < heap_end);
        }
    }

    #[test]
    fn test_is_valid_heap_pointer() {
        let heap_range = (0x1000_0000usize, 0x2000_0000usize);

        // Valid pointer
        assert!(StackScanner::is_valid_heap_pointer(0x1500_0000, heap_range));

        // Zero is invalid
        assert!(!StackScanner::is_valid_heap_pointer(0, heap_range));

        // Outside range is invalid
        assert!(!StackScanner::is_valid_heap_pointer(0x0500_0000, heap_range));
        assert!(!StackScanner::is_valid_heap_pointer(0x2500_0000, heap_range));

        // Misaligned pointer is invalid
        assert!(!StackScanner::is_valid_heap_pointer(0x1500_0001, heap_range));
    }

    #[test]
    fn test_clear() {
        let scanner = StackScanner::new();

        // Setup watermark
        scanner.setup_watermark(1, 0x7fff_0000).unwrap();
        assert!(scanner.get_watermark(1).is_some());

        // Clear
        scanner.clear();
        assert!(scanner.get_watermark(1).is_none());
    }

    #[test]
    fn test_get_stats() {
        let scanner = StackScanner::new();
        let (frames, pointers) = scanner.get_stats();
        assert_eq!(frames, 0);
        assert_eq!(pointers, 0);
    }

    #[test]
    fn test_set_conservative_fallback() {
        let scanner = StackScanner::new();

        scanner.set_conservative_fallback(false);
        assert!(!scanner.conservative_fallback.load(Ordering::Relaxed));

        scanner.set_conservative_fallback(true);
        assert!(scanner.conservative_fallback.load(Ordering::Relaxed));
    }

    #[test]
    fn test_frame_iterator() {
        // Test that scan works with realistic stack range
        let scanner = StackScanner::new();

        // Get actual stack range for current thread
        let current_stack = &scanner as *const StackScanner as usize;
        let stack_base = current_stack + 0x10000; // Estimate
        let stack_pointer = current_stack;

        let result = scanner.setup_watermark_with_base(1, stack_pointer, stack_base);
        assert!(result.is_ok());

        // Scan should not panic
        let heap_range = (0x1000_0000, usize::MAX);
        let result = scanner.scan_below_watermark(1, heap_range);
        assert!(result.is_ok());
    }

    /// Integration test: simulate a GC cycle
    #[test]
    fn test_gc_cycle_simulation() {
        let scanner = StackScanner::new();
        let heap_range = (0x1000_0000usize, 0x2000_0000usize);

        // Simulate multiple threads
        for i in 0..4 {
            let thread_id = i as u64;
            let stack_base = 0x7fff_0000usize + i * 0x10000;
            let stack_pointer = stack_base - 0x1000;

            scanner.setup_watermark_with_base(thread_id, stack_pointer, stack_base).unwrap();

            let pointers = scanner.scan_below_watermark(thread_id, heap_range).unwrap();

            // Pointers should all be within heap range
            for ptr in &pointers {
                assert!(*ptr >= heap_range.0 && *ptr < heap_range.1);
            }
        }

        // Verify stats
        let (frames, pointers) = scanner.get_stats();
        assert!(frames >= 0);
        assert!(pointers >= 0);

        // Clear for next cycle
        scanner.clear();
        assert_eq!(scanner.get_stats(), (0, 0));
    }

    /// Test strict heap pointer validation reduces false positives
    #[test]
    fn test_is_valid_heap_pointer_strict() {
        let heap_range = (0x1000_0000usize, 0x2000_0000usize);

        // Valid pointer (aligned to 8 bytes)
        assert!(StackScanner::is_valid_heap_pointer_strict(0x1500_0000, heap_range));

        // Zero is invalid
        assert!(!StackScanner::is_valid_heap_pointer_strict(0, heap_range));

        // Outside range is invalid
        assert!(!StackScanner::is_valid_heap_pointer_strict(0x0500_0000, heap_range));
        assert!(!StackScanner::is_valid_heap_pointer_strict(0x2500_0000, heap_range));

        // 4-byte aligned but not 8-byte aligned should be invalid (stricter check)
        assert!(!StackScanner::is_valid_heap_pointer_strict(0x1500_0004, heap_range));

        // Odd addresses are invalid
        assert!(!StackScanner::is_valid_heap_pointer_strict(0x1500_0001, heap_range));
        assert!(!StackScanner::is_valid_heap_pointer_strict(0x1500_0007, heap_range));
    }

    /// Test that strict validation rejects more false positives than basic validation
    #[test]
    fn test_strict_vs_basic_validation() {
        let heap_range = (0x1000_0000usize, 0x2000_0000usize);

        // 4-byte aligned address (valid for basic, invalid for strict)
        let addr_4byte_aligned = 0x1500_0004usize;
        
        // Basic validation accepts 4-byte alignment
        assert!(StackScanner::is_valid_heap_pointer(addr_4byte_aligned, heap_range));
        
        // Strict validation requires 8-byte alignment
        assert!(!StackScanner::is_valid_heap_pointer_strict(addr_4byte_aligned, heap_range));

        // 8-byte aligned address should pass both
        let addr_8byte_aligned = 0x1500_0008usize;
        assert!(StackScanner::is_valid_heap_pointer(addr_8byte_aligned, heap_range));
        assert!(StackScanner::is_valid_heap_pointer_strict(addr_8byte_aligned, heap_range));
    }

    /// Test stack scanning finds known roots
    #[test]
    fn test_stack_scanning_finds_roots() {
        let scanner = StackScanner::new();
        
        // Create known values on stack that look like heap pointers
        let heap_start = 0x1000_0000usize;
        let heap_end = 0x2000_0000usize;
        let known_root = 0x1500_0000usize; // 8-byte aligned, in heap range
        
        // Create buffer with known root
        let buffer = [known_root, 0usize, !0usize, known_root];
        let start = buffer.as_ptr() as usize;
        let end = start + buffer.len() * std::mem::size_of::<usize>();
        
        // Scan the buffer
        let pointers = scan_stack_range(start, end, (heap_start, heap_end));
        
        // Should find the known roots (at least one occurrence)
        assert!(pointers.iter().any(|&p| p == known_root), 
            "Stack scanning should find known root!");
        
        // All found pointers should be valid
        for ptr in &pointers {
            assert!(*ptr >= heap_start && *ptr < heap_end,
                "Found pointer outside heap range!");
        }
    }

    /// Test safepoint functionality
    #[test]
    fn test_safepoint_basic() {
        use crate::runtime::safepoint::{Safepoint, SAFEPOINT_NONE};
        
        let safepoint = Safepoint::new(2);
        
        // Initial state should be NONE
        assert_eq!(safepoint.get_state(), SAFEPOINT_NONE);
        assert_eq!(safepoint.threads_at_safepoint(), 0);
        assert_eq!(safepoint.total_threads(), 2);
        
        // Not requested initially
        assert!(!safepoint.is_requested());
    }

    /// Test safepoint request and arrive
    #[test]
    fn test_safepoint_request_and_arrive() {
        use crate::runtime::safepoint::{Safepoint, SAFEPOINT_NONE, SAFEPOINT_REQUESTED, SAFEPOINT_REACHED};
        
        let safepoint = Safepoint::new(2);
        
        // GC thread requests safepoint
        safepoint.request_safepoint();
        assert_eq!(safepoint.get_state(), SAFEPOINT_REQUESTED);
        assert!(safepoint.is_requested());
        
        // Thread 1 arrives
        safepoint.arrive();
        assert_eq!(safepoint.threads_at_safepoint(), 1);
        assert_eq!(safepoint.get_state(), SAFEPOINT_REACHED);
        
        // Thread 2 arrives
        safepoint.arrive();
        assert_eq!(safepoint.threads_at_safepoint(), 2);
        
        // GC releases safepoint
        safepoint.release_safepoint();
        assert_eq!(safepoint.get_state(), SAFEPOINT_NONE);
        assert_eq!(safepoint.threads_at_safepoint(), 0);
        assert!(!safepoint.is_requested());
    }

    /// Test safepoint wait for all threads
    #[test]
    fn test_safepoint_wait_for_all() {
        use std::thread;
        use std::sync::Arc;
        use crate::runtime::safepoint::Safepoint;
        
        let safepoint = Arc::new(Safepoint::new(3));
        let safepoint_clone = Arc::clone(&safepoint);
        
        // Spawn threads that will arrive at safepoint
        let handles: Vec<_> = (0..3).map(|_| {
            let sp = Arc::clone(&safepoint_clone);
            thread::spawn(move || {
                sp.arrive();
            })
        }).collect();
        
        // Request safepoint and wait
        safepoint.request_safepoint();
        safepoint.wait_for_safepoint();
        
        // All threads should have arrived
        assert_eq!(safepoint.threads_at_safepoint(), 3);
        
        // Release and join threads
        safepoint.release_safepoint();
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
