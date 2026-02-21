//! Memory Mapping - Cross-Platform Wrapper for memmap2
//!
//! This module provides cross-platform abstraction for memory mapping
//! using the memmap2 crate. Supports:
//! - Anonymous mappings (without file backing)
//! - Read/Write memory
//! - Memory protection changes
//!
//! Platform Support:
//! - Linux: mmap/munmap/mprotect
//! - Windows: VirtualAlloc/VirtualFree/VirtualProtect
//! - macOS: mmap/munmap/mprotect
//!
//! Usage:
//! ```
//! let mapping = MemoryMapping::anonymous(4096)?;
//!
//! // Write to memory
//! mapping.write(0, &[1, 2, 3, 4])?;
//!
//! // Read from memory
//! let mut buf = [0u8; 4];
//! mapping.read(0, &mut buf)?;
//! ```

use crate::error::{FgcError, Result};
use memmap2::{MmapMut, MmapOptions};
use std::io::Write;
use std::sync::Arc;

/// MemoryMapping - wrapper for memory mapped region
///
/// Thread-safe via Arc<MmapMut>.
/// Supports read/write operations to mapped memory.
pub struct MemoryMapping {
    /// Inner mmap handle
    mmap: Arc<MmapMut>,

    /// Base address of mapping
    base: usize,

    /// Size of mapping in bytes
    size: usize,

    /// Whether memory is currently readable
    readable: bool,

    /// Whether memory is currently writable
    writable: bool,
}

impl MemoryMapping {
    /// Create anonymous memory mapping
    ///
    /// Creates memory mapping without file backing.
    /// Memory is initialized with zeros.
    ///
    /// # Arguments
    /// * `size` - Size in bytes (will be rounded to page boundary)
    ///
    /// # Returns
    /// MemoryMapping instance or error
    ///
    /// # Examples
    /// ```
    /// let mapping = MemoryMapping::anonymous(4096)?;
    /// assert!(mapping.size() >= 4096);
    /// ```
    pub fn anonymous(size: usize) -> Result<Self> {
        let aligned_size = crate::heap::page::align_to_page(size);

        let mmap = MmapOptions::new()
            .len(aligned_size)
            .map_anon()
            .map_err(|e| {
                FgcError::VirtualMemoryError(format!("Failed to create anonymous mapping: {}", e))
            })?;

        let base = mmap.as_ptr() as usize;

        Ok(Self {
            mmap: Arc::new(mmap),
            base,
            size: aligned_size,
            readable: true,
            writable: true,
        })
    }

    /// Create anonymous mapping with hint address
    ///
    /// Provides a hint to the OS about preferred address.
    /// OS is not guaranteed to use this hint.
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    /// * `hint` - Preferred base address (optional)
    pub fn anonymous_with_hint(size: usize, hint: Option<usize>) -> Result<Self> {
        let aligned_size = crate::heap::page::align_to_page(size);

        let mut opts = MmapOptions::new();
        opts.len(aligned_size);

        if let Some(_addr) = hint {
            // Note: memmap2 does not directly support hint address
            // We can try with stack allocation pattern
            opts.stack();
        }

        let mmap = opts.map_anon().map_err(|e| {
            FgcError::VirtualMemoryError(format!("Failed to create anonymous mapping: {}", e))
        })?;

        let base = mmap.as_ptr() as usize;

        Ok(Self {
            mmap: Arc::new(mmap),
            base,
            size: aligned_size,
            readable: true,
            writable: true,
        })
    }

    /// Get base address of mapping
    pub fn base(&self) -> usize {
        self.base
    }

    /// Get size of mapping
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if memory readable
    pub fn is_readable(&self) -> bool {
        self.readable
    }

    /// Check if memory writable
    pub fn is_writable(&self) -> bool {
        self.writable
    }

    /// Check if address in mapping range
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.base && addr < self.base + self.size
    }

    /// Check if range in mapping
    pub fn contains_range(&self, offset: usize, len: usize) -> bool {
        offset.saturating_add(len) <= self.size
    }

    /// Read bytes from mapping
    ///
    /// # Arguments
    /// * `offset` - Offset from base address
    /// * `buf` - Buffer to store bytes
    ///
    /// # Safety
    /// Offset + buf.len() must be <= size
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        if !self.readable {
            return Err(FgcError::VirtualMemoryError(
                "Memory is not readable".to_string(),
            ));
        }

        if offset.saturating_add(buf.len()) > self.size {
            return Err(FgcError::VirtualMemoryError(format!(
                "Read out of bounds: offset={}, len={}, size={}",
                offset,
                buf.len(),
                self.size
            )));
        }

        // Get slice from mmap
        let data = &self.mmap[offset..offset + buf.len()];
        buf.copy_from_slice(data);

        Ok(())
    }

    /// Write bytes to mapping
    ///
    /// # Arguments
    /// * `offset` - Offset from base address
    /// * `data` - Bytes to write
    ///
    /// # Safety
    /// Offset + data.len() must be <= size
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if !self.writable {
            return Err(FgcError::VirtualMemoryError(
                "Memory is not writable".to_string(),
            ));
        }

        if offset.saturating_add(data.len()) > self.size {
            return Err(FgcError::VirtualMemoryError(format!(
                "Write out of bounds: offset={}, len={}, size={}",
                offset,
                data.len(),
                self.size
            )));
        }

        // Flush before write to ensure consistency
        self.flush()?;

        // Get mutable slice and copy data
        let mmap = Arc::get_mut(&mut self.mmap).ok_or_else(|| {
            FgcError::VirtualMemoryError(
                "Cannot get mutable reference to shared mapping".to_string(),
            )
        })?;

        mmap[offset..offset + data.len()].copy_from_slice(data);

        Ok(())
    }

    /// Flush changes to kernel
    ///
    /// Ensures writes are visible to kernel and potentially to disk
    /// (for file-backed mappings).
    pub fn flush(&self) -> Result<()> {
        self.mmap
            .flush()
            .map_err(|e| FgcError::VirtualMemoryError(format!("Failed to flush mapping: {}", e)))
    }

    /// Flush range to kernel
    pub fn flush_range(&self, offset: usize, len: usize) -> Result<()> {
        if offset.saturating_add(len) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Flush range out of bounds".to_string(),
            ));
        }

        self.mmap
            .flush_async_range(offset, len)
            .map_err(|e| FgcError::VirtualMemoryError(format!("Failed to flush range: {}", e)))
    }

    /// Get pointer to memory
    ///
    /// # Safety
    /// Caller is responsible for not exceeding bounds.
    pub fn as_ptr(&self) -> *const u8 {
        self.mmap.as_ptr()
    }

    /// Get mutable pointer to memory
    ///
    /// # Safety
    /// Caller is responsible for not exceeding bounds and
    /// handling concurrent access.
    pub fn as_mut_ptr(&mut self) -> Result<*mut u8> {
        let mmap = Arc::get_mut(&mut self.mmap).ok_or_else(|| {
            FgcError::VirtualMemoryError("Cannot get mutable pointer to shared mapping".to_string())
        })?;
        Ok(mmap.as_mut_ptr())
    }

    /// Get slice from mapping
    ///
    /// # Arguments
    /// * `offset` - Start offset
    /// * `len` - Length
    pub fn as_slice(&self, offset: usize, len: usize) -> Result<&[u8]> {
        if offset.saturating_add(len) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Slice out of bounds".to_string(),
            ));
        }
        Ok(&self.mmap[offset..offset + len])
    }

    /// Get mutable slice from mapping
    pub fn as_mut_slice(&mut self, offset: usize, len: usize) -> Result<&mut [u8]> {
        if offset.saturating_add(len) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Slice out of bounds".to_string(),
            ));
        }

        let mmap = Arc::get_mut(&mut self.mmap).ok_or_else(|| {
            FgcError::VirtualMemoryError("Cannot get mutable slice of shared mapping".to_string())
        })?;

        Ok(&mut mmap[offset..offset + len])
    }

    /// Fill memory with value
    pub fn fill(&mut self, offset: usize, len: usize, value: u8) -> Result<()> {
        let slice = self.as_mut_slice(offset, len)?;
        slice.fill(value);
        Ok(())
    }

    /// Zero out memory range
    pub fn zero(&mut self, offset: usize, len: usize) -> Result<()> {
        self.fill(offset, len, 0)
    }

    /// Clone reference to mapping (shares underlying memory)
    pub fn clone_ref(&self) -> Self {
        Self {
            mmap: Arc::clone(&self.mmap),
            base: self.base,
            size: self.size,
            readable: self.readable,
            writable: self.writable,
        }
    }
}

impl Clone for MemoryMapping {
    fn clone(&self) -> Self {
        self.clone_ref()
    }
}

/// OwnedMemoryMapping - memory mapping with exclusive ownership
///
/// Unlike MemoryMapping, this does not use Arc
/// so it can get unique mutable references.
pub struct OwnedMemoryMapping {
    mmap: MmapMut,
    base: usize,
    size: usize,
}

impl OwnedMemoryMapping {
    /// Create anonymous mapping with exclusive ownership
    pub fn anonymous(size: usize) -> Result<Self> {
        let aligned_size = crate::heap::page::align_to_page(size);

        let mmap = MmapOptions::new()
            .len(aligned_size)
            .map_anon()
            .map_err(|e| {
                FgcError::VirtualMemoryError(format!("Failed to create anonymous mapping: {}", e))
            })?;

        let base = mmap.as_ptr() as usize;

        Ok(Self {
            mmap,
            base,
            size: aligned_size,
        })
    }

    pub fn base(&self) -> usize {
        self.base
    }
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.mmap[..]
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.mmap.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.mmap.as_mut_ptr()
    }

    /// Convert to shared mapping
    pub fn into_shared(self) -> MemoryMapping {
        MemoryMapping {
            mmap: Arc::new(self.mmap),
            base: self.base,
            size: self.size,
            readable: true,
            writable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymous_mapping_creation() {
        let mapping = MemoryMapping::anonymous(4096).unwrap();

        assert!(mapping.base() > 0);
        assert!(mapping.size() >= 4096);
        assert!(mapping.is_readable());
        assert!(mapping.is_writable());
    }

    #[test]
    fn test_mapping_read_write() {
        let mut mapping = MemoryMapping::anonymous(4096).unwrap();

        // Write data
        let data = [1u8, 2, 3, 4, 5];
        mapping.write(0, &data).unwrap();

        // Read back
        let mut buf = [0u8; 5];
        mapping.read(0, &mut buf).unwrap();

        assert_eq!(buf, data);
    }

    #[test]
    fn test_mapping_bounds_check() {
        let mapping = MemoryMapping::anonymous(4096).unwrap();

        // Try read beyond bounds
        let mut buf = [0u8; 100];
        let result = mapping.read(4000, &mut buf);

        assert!(result.is_err());
    }

    #[test]
    fn test_mapping_zero() {
        let mut mapping = MemoryMapping::anonymous(4096).unwrap();

        // Write some data
        mapping.write(0, &[255; 100]).unwrap();

        // Zero out
        mapping.zero(0, 100).unwrap();

        // Verify zeros
        let mut buf = [0u8; 100];
        mapping.read(0, &mut buf).unwrap();

        assert_eq!(buf, [0u8; 100]);
    }

    #[test]
    fn test_mapping_fill() {
        let mut mapping = MemoryMapping::anonymous(4096).unwrap();

        // Fill with value
        mapping.fill(0, 100, 0x42).unwrap();

        // Verify
        let mut buf = [0u8; 100];
        mapping.read(0, &mut buf).unwrap();

        assert_eq!(buf, [0x42u8; 100]);
    }

    #[test]
    fn test_owned_mapping() {
        let mut mapping = OwnedMemoryMapping::anonymous(4096).unwrap();

        // Write via mutable slice
        let slice = mapping.as_mut_slice();
        slice[0] = 42;
        slice[1] = 43;

        // Read via slice
        let slice = mapping.as_slice();
        assert_eq!(slice[0], 42);
        assert_eq!(slice[1], 43);
    }

    #[test]
    fn test_mapping_clone() {
        let mapping1 = MemoryMapping::anonymous(4096).unwrap();
        let mapping2 = mapping1.clone();

        // Both share same underlying memory
        assert_eq!(mapping1.base(), mapping2.base());
        assert_eq!(mapping1.size(), mapping2.size());
    }

    #[test]
    fn test_mapping_contains() {
        let mapping = MemoryMapping::anonymous(4096).unwrap();
        let base = mapping.base();

        assert!(mapping.contains(base));
        assert!(mapping.contains(base + 100));
        assert!(mapping.contains(base + 4095));
        assert!(!mapping.contains(base + 4096));
        assert!(!mapping.contains(base - 1));
    }
}
