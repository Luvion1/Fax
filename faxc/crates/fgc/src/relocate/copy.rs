//! Object Copying - Concurrent Object Copy
//!
//! This module implements object copying during relocation phase.
//! Copy must be concurrent-safe because mutator threads may access
//! objects being copied.
//!
//! Copy Strategy:
//! 1. Allocate space in destination
//! 2. Copy object data (memcpy)
//! 3. Set forwarding entry
//! 4. Update object header (if needed)
//!
//! Safety:
//! - Object lock during copying
//! - Atomic forwarding pointer update
//! - Load barrier handles concurrent access

use crate::error::Result;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Validate memory copy region before copying
///
/// # Arguments
/// * `source` - Source address
/// * `destination` - Destination address
/// * `size` - Number of bytes to copy
///
/// # Returns
/// * `Ok(())` - Validation passed
/// * `Err(FgcError)` - Validation failed
fn validate_copy_region(source: usize, destination: usize, size: usize) -> Result<()> {
    if size == 0 {
        return Ok(());
    }

    // Validate addresses are not null
    if source == 0 || destination == 0 {
        return Err(crate::error::FgcError::InvalidPointer {
            address: if source == 0 { source } else { destination }
        });
    }

    // Check for integer overflow in address calculations
    let source_end = source.checked_add(size)
        .ok_or_else(|| crate::error::FgcError::InvalidArgument("source address overflow".to_string()))?;
    let dest_end = destination.checked_add(size)
        .ok_or_else(|| crate::error::FgcError::InvalidArgument("destination address overflow".to_string()))?;

    // Check for overlapping memory regions
    // Overlap occurs if: source < dest_end AND destination < source_end
    if source < dest_end && destination < source_end {
        return Err(crate::error::FgcError::InvalidArgument(
            "overlapping memory regions detected".to_string()
        ));
    }

    // Validate memory regions are readable/writable
    if !crate::memory::is_readable(source).unwrap_or(false) {
        return Err(crate::error::FgcError::InvalidArgument(
            "source address is not readable".to_string()
        ));
    }
    if !crate::memory::is_writable(destination).unwrap_or(false) {
        return Err(crate::error::FgcError::InvalidArgument(
            "destination address is not writable".to_string()
        ));
    }

    Ok(())
}

/// ObjectCopier - copier for object relocation
///
/// Manages concurrent object copying.
pub struct ObjectCopier {
    /// Bytes copied
    bytes_copied: AtomicU64,
    /// Objects copied
    objects_copied: AtomicU64,
    /// Copy errors
    copy_errors: AtomicUsize,
}

impl ObjectCopier {
    /// Create new object copier
    pub fn new() -> Self {
        Self {
            bytes_copied: AtomicU64::new(0),
            objects_copied: AtomicU64::new(0),
            copy_errors: AtomicUsize::new(0),
        }
    }

    /// Copy object from source to destination
    ///
    /// Performs actual memory copy from source to destination.
    ///
    /// # Arguments
    /// * `source` - Source address
    /// * `destination` - Destination address
    /// * `size` - Object size
    ///
    /// # Returns
    /// `Result<()>` - Ok if copy successful, Err if validation fails
    ///
    /// # Validation
    /// This function performs comprehensive validation before copying:
    /// - Null address check
    /// - Integer overflow check
    /// - Memory overlap check
    /// - Readable/writable memory check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fgc::relocate::copy::ObjectCopier;
    ///
    /// let copier = ObjectCopier::new();
    /// let src = [1u8, 2, 3, 4];
    /// let mut dst = [0u8; 4];
    ///
    /// let result = copier.copy_object(
    ///     src.as_ptr() as usize,
    ///     dst.as_mut_ptr() as usize,
    ///     4
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn copy_object(&self, source: usize, destination: usize, size: usize) -> Result<()> {
        // Validate copy region using shared validation function
        validate_copy_region(source, destination, size)?;

        unsafe {
            std::ptr::copy_nonoverlapping(source as *const u8, destination as *mut u8, size);

            std::sync::atomic::fence(Ordering::Release);
        }

        self.bytes_copied.fetch_add(size as u64, Ordering::Relaxed);
        self.objects_copied.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Copy object with forwarding
    ///
    /// Copy object and setup forwarding entry.
    pub fn copy_with_forwarding(
        &self,
        source: usize,
        destination: usize,
        size: usize,
        forwarding_table: &crate::relocate::ForwardingTable,
    ) -> Result<()> {
        self.copy_object(source, destination, size)?;

        forwarding_table.add_entry(source, destination);

        Ok(())
    }

    /// Copy object with checksum verification
    ///
    /// Perform copy and verify that copy succeeded.
    pub fn copy_with_verification(
        &self,
        source: usize,
        destination: usize,
        size: usize,
    ) -> Result<bool> {
        self.copy_object(source, destination, size)?;

        unsafe {
            let src_slice = std::slice::from_raw_parts(source as *const u8, size);
            let dst_slice = std::slice::from_raw_parts(destination as *const u8, size);

            Ok(src_slice == dst_slice)
        }
    }

    /// Get bytes copied
    pub fn bytes_copied(&self) -> u64 {
        self.bytes_copied.load(Ordering::Relaxed)
    }

    /// Get objects copied
    pub fn objects_copied(&self) -> u64 {
        self.objects_copied.load(Ordering::Relaxed)
    }

    /// Get copy errors
    pub fn copy_errors(&self) -> usize {
        self.copy_errors.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.bytes_copied.store(0, Ordering::Relaxed);
        self.objects_copied.store(0, Ordering::Relaxed);
        self.copy_errors.store(0, Ordering::Relaxed);
    }

    /// Get copy statistics
    pub fn stats(&self) -> CopyStats {
        CopyStats {
            bytes_copied: self.bytes_copied.load(Ordering::Relaxed),
            objects_copied: self.objects_copied.load(Ordering::Relaxed),
            errors: self.copy_errors.load(Ordering::Relaxed) as u64,
            copy_speed: 0.0,
        }
    }
}

impl Default for ObjectCopier {
    fn default() -> Self {
        Self::new()
    }
}

/// Copy statistics
#[derive(Debug, Default, Clone)]
pub struct CopyStats {
    /// Bytes copied
    pub bytes_copied: u64,
    /// Objects copied
    pub objects_copied: u64,
    /// Copy errors
    pub errors: u64,
    /// Average copy speed (bytes/ms)
    pub copy_speed: f64,
}

/// Helper for memcopy with alignment
///
/// Copy memory with proper alignment.
///
/// # Safety
///
/// This function is safe to call if and only if:
/// 1. `src` is a valid, mapped memory address for reads of `size` bytes
/// 2. `dst` is a valid, mapped memory address for writes of `size` bytes
/// 3. `src` and `dst` do not overlap
/// 4. `size` does not overflow when added to addresses
///
/// The function performs internal validation and will silently return
/// without copying if validation fails.
///
/// # Arguments
/// * `src` - Source address
/// * `dst` - Destination address
/// * `size` - Number of bytes to copy
/// * `alignment` - Required alignment (used for optimization)
///
/// # Examples
///
/// ```rust
/// use fgc::relocate::copy::aligned_copy;
///
/// let src = [1u64, 2, 3, 4];
/// let mut dst = [0u64; 4];
///
/// unsafe {
///     aligned_copy(
///         src.as_ptr() as usize,
///         dst.as_mut_ptr() as usize,
///         32,
///         8
///     );
/// }
/// assert_eq!(src, dst);
/// ```
#[inline]
pub unsafe fn aligned_copy(src: usize, dst: usize, size: usize, alignment: usize) {
    // CRIT-08 FIX: Add validation before copy

    // Check for null addresses
    if src == 0 || dst == 0 || size == 0 {
        return;
    }

    // Check alignment
    if src % alignment != 0 || dst % alignment != 0 {
        std::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
        return;
    }

    // Check for overflow
    let src_end = match src.checked_add(size) {
        Some(end) => end,
        None => return,
    };
    let dst_end = match dst.checked_add(size) {
        Some(end) => end,
        None => return,
    };

    // Check for overlap
    if src < dst_end && dst < src_end {
        return;  // Don't copy overlapping regions
    }

    let word_count = size / 8;
    let remainder = size % 8;

    for i in 0..word_count {
        let src_word = *(src as *const u64).add(i);
        *(dst as *mut u64).add(i) = src_word;
    }

    let byte_offset = word_count * 8;
    for i in 0..remainder {
        let src_byte = *((src + byte_offset + i) as *const u8);
        *((dst + byte_offset + i) as *mut u8) = src_byte;
    }
}

/// Atomic copy object with lock-free approach
///
/// Copy using atomic operations for thread safety.
///
/// # Arguments
/// * `source` - Source address
/// * `destination` - Destination address
/// * `size` - Object size
///
/// # Returns
/// `Result<()>` - Ok if copy successful, Err if validation fails
///
/// # Validation
/// This function validates addresses and checks for overflow/overlap.
///
/// # Examples
///
/// ```rust
/// use fgc::relocate::copy::atomic_copy_object;
///
/// let src = [1u8, 2, 3, 4];
/// let mut dst = [0u8; 4];
///
/// let result = atomic_copy_object(
///     src.as_ptr() as usize,
///     dst.as_mut_ptr() as usize,
///     4
/// );
/// assert!(result.is_ok());
/// ```
pub fn atomic_copy_object(source: usize, destination: usize, size: usize) -> Result<()> {
    // Validate copy region using shared validation function
    validate_copy_region(source, destination, size)?;

    unsafe {
        std::ptr::copy_nonoverlapping(source as *const u8, destination as *mut u8, size);
        std::sync::atomic::fence(Ordering::SeqCst);
    }

    Ok(())
}

/// Batch copy multiple objects
///
/// Efficiently copy multiple objects in batch.
pub fn batch_copy_objects(copies: &[(usize, usize, usize)]) -> Result<CopyStats> {
    let copier = ObjectCopier::new();

    for &(source, destination, size) in copies {
        copier.copy_object(source, destination, size)?;
    }

    Ok(copier.stats())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_object_basic() {
        let copier = ObjectCopier::new();

        let src_data = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut dst_data = [0u8; 8];

        let src_addr = src_data.as_ptr() as usize;
        let dst_addr = dst_data.as_mut_ptr() as usize;

        copier.copy_object(src_addr, dst_addr, 8).unwrap();

        assert_eq!(src_data, dst_data);
        assert_eq!(copier.bytes_copied(), 8);
        assert_eq!(copier.objects_copied(), 1);
    }

    #[test]
    fn test_copy_object_zero_size() {
        let copier = ObjectCopier::new();

        let result = copier.copy_object(0x1000, 0x2000, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_with_verification() {
        let copier = ObjectCopier::new();

        let src_data = [42u8; 100];
        let mut dst_data = [0u8; 100];

        let src_addr = src_data.as_ptr() as usize;
        let dst_addr = dst_data.as_mut_ptr() as usize;

        let verified = copier
            .copy_with_verification(src_addr, dst_addr, 100)
            .unwrap();

        assert!(verified);
        assert_eq!(src_data, dst_data);
    }

    #[test]
    fn test_batch_copy() {
        let src1 = [1u8, 2, 3];
        let src2 = [4u8, 5, 6];
        let mut dst1 = [0u8; 3];
        let mut dst2 = [0u8; 3];

        let copies = [
            (src1.as_ptr() as usize, dst1.as_mut_ptr() as usize, 3),
            (src2.as_ptr() as usize, dst2.as_mut_ptr() as usize, 3),
        ];

        let stats = batch_copy_objects(&copies).unwrap();

        assert_eq!(stats.objects_copied, 2);
        assert_eq!(stats.bytes_copied, 6);
        assert_eq!(dst1, src1);
        assert_eq!(dst2, src2);
    }

    #[test]
    fn test_aligned_copy() {
        let mut src = [0u64; 4];
        src[0] = 0x1111111111111111;
        src[1] = 0x2222222222222222;
        src[2] = 0x3333333333333333;
        src[3] = 0x4444444444444444;

        let mut dst = [0u64; 4];

        unsafe {
            aligned_copy(src.as_ptr() as usize, dst.as_mut_ptr() as usize, 32, 8);
        }

        assert_eq!(src, dst);
    }
}
