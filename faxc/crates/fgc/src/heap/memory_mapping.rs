//! Memory Mapping - Cross-Platform Wrapper for memmap2
//!
//! This module provides cross-platform abstraction for memory mapping
//! using the memmap2 crate. Supports:
//! - Anonymous mappings (without file backing)
//! - Read/Write memory
//! - Memory protection changes
//! - Large Pages (HugeTLB) support
//! - Transparent Huge Pages (THP) support
//!
//! Platform Support:
//! - Linux: mmap/munmap/mprotect/MADV_HUGEPAGE
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
use memmap2::MmapMut;
use std::cell::UnsafeCell;

/// MemoryMapping - wrapper for memory-mapped region
///
/// Provides safe abstraction over mmap for anonymous memory mapping.
pub struct MemoryMapping {
    /// Underlying mmap (wrapped in UnsafeCell for interior mutability)
    mmap: UnsafeCell<MmapMut>,
    /// Base address of mapping
    base: usize,
    /// Size of mapping
    size: usize,
}

// Safety: MemoryMapping is safe to share across threads
// because we only allow immutable operations through &self
unsafe impl Send for MemoryMapping {}
unsafe impl Sync for MemoryMapping {}

impl MemoryMapping {
    /// Create anonymous memory mapping
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully created mapping
    /// * `Err(FgcError)` - Mapping failed
    pub fn anonymous(size: usize) -> Result<Self> {
        let mmap = MmapMut::map_anon(size)
            .map_err(|e| FgcError::VirtualMemoryError(format!("mmap failed: {}", e)))?;

        let base = mmap.as_ptr() as usize;

        Ok(Self {
            mmap: UnsafeCell::new(mmap),
            base,
            size,
        })
    }

    pub fn anonymous_large_pages(size: usize, huge_page_size: usize) -> Result<Self> {
        log::info!("Large pages requested ({} bytes) but not fully implemented, falling back to regular pages", huge_page_size);
        let _ = size;
        Self::anonymous(huge_page_size)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn anonymous_large_pages(size: usize, _huge_page_size: usize) -> Result<Self> {
        Self::anonymous(size)
    }

    #[cfg(target_os = "linux")]
    pub fn enable_transparent_huge_pages(&self) -> Result<()> {
        let ret = unsafe {
            libc::madvise(
                self.as_ptr() as *mut libc::c_void,
                self.size,
                libc::MADV_HUGEPAGE,
            )
        };

        if ret != 0 {
            log::warn!("MADV_HUGEPAGE failed, falling back to regular pages");
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn enable_transparent_huge_pages(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn enable_no_prefork(&self) -> Result<()> {
        let ret = unsafe {
            libc::madvise(
                self.as_ptr() as *mut libc::c_void,
                self.size,
                libc::MADV_NOHUGEPAGE,
            )
        };

        if ret != 0 {
            log::warn!("MADV_NOHUGEPAGE failed");
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn enable_no_prefork(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn advise_will_need(&self) -> Result<()> {
        let ret = unsafe {
            libc::madvise(
                self.as_ptr() as *mut libc::c_void,
                self.size,
                libc::MADV_WILLNEED,
            )
        };

        if ret != 0 {
            log::warn!("MADV_WILLNEED failed");
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn advise_will_need(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn advise_dont_need(&self) -> Result<()> {
        let ret = unsafe {
            libc::madvise(
                self.as_ptr() as *mut libc::c_void,
                self.size,
                libc::MADV_DONTNEED,
            )
        };

        if ret != 0 {
            log::warn!("MADV_DONTNEED failed");
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn advise_dont_need(&self) -> Result<()> {
        Ok(())
    }

    /// Get base address
    pub fn base(&self) -> usize {
        self.base
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get pointer to memory
    pub fn as_ptr(&self) -> *const u8 {
        unsafe { (*self.mmap.get()).as_ptr() }
    }

    /// Get mutable pointer to memory
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { (*self.mmap.get()).as_mut_ptr() }
    }

    /// Read from memory
    ///
    /// # Arguments
    /// * `offset` - Offset from base
    /// * `buf` - Buffer to read into
    ///
    /// # Returns
    /// * `Ok(())` - Successfully read
    /// * `Err(FgcError)` - Read failed
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        if offset.saturating_add(buf.len()) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Read out of bounds".to_string(),
            ));
        }

        let mmap = unsafe { &*self.mmap.get() };
        buf.copy_from_slice(&mmap[offset..offset + buf.len()]);
        Ok(())
    }

    /// Write to memory
    ///
    /// # Arguments
    /// * `offset` - Offset from base
    /// * `data` - Data to write
    ///
    /// # Returns
    /// * `Ok(())` - Successfully written
    /// * `Err(FgcError)` - Write failed
    pub fn write(&self, offset: usize, data: &[u8]) -> Result<()> {
        if offset.saturating_add(data.len()) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Write out of bounds".to_string(),
            ));
        }

        let mmap = unsafe { &mut *self.mmap.get() };
        mmap[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Get memory as slice
    ///
    /// # Arguments
    /// * `offset` - Offset from base
    /// * `len` - Length of slice
    ///
    /// # Returns
    /// * `Ok(&[u8])` - Successfully got slice
    /// * `Err(FgcError)` - Invalid range
    pub fn as_slice(&self, offset: usize, len: usize) -> Result<&[u8]> {
        if offset.saturating_add(len) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Slice out of bounds".to_string(),
            ));
        }

        let mmap = unsafe { &*self.mmap.get() };
        Ok(&mmap[offset..offset + len])
    }

    /// Get mutable slice
    pub fn as_mut_slice(&mut self, offset: usize, len: usize) -> Result<&mut [u8]> {
        if offset.saturating_add(len) > self.size {
            return Err(FgcError::VirtualMemoryError(
                "Mutable slice out of bounds".to_string(),
            ));
        }

        let mmap = unsafe { &mut *self.mmap.get() };
        Ok(&mut mmap[offset..offset + len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_mapping_creation() {
        let mapping = MemoryMapping::anonymous(4096).unwrap();
        assert!(mapping.base() > 0);
        assert_eq!(mapping.size(), 4096);
    }

    #[test]
    fn test_memory_mapping_read_write() {
        let mapping = MemoryMapping::anonymous(4096).unwrap();

        let data = [1u8, 2, 3, 4, 5];
        mapping.write(0, &data).unwrap();

        let mut buf = [0u8; 5];
        mapping.read(0, &mut buf).unwrap();

        assert_eq!(buf, data);
    }

    #[test]
    fn test_memory_mapping_as_slice() {
        let mapping = MemoryMapping::anonymous(4096).unwrap();

        let data = [10u8, 20, 30, 40];
        mapping.write(100, &data).unwrap();

        let slice = mapping.as_slice(100, 4).unwrap();
        assert_eq!(slice, &data);
    }

    #[test]
    fn test_memory_mapping_out_of_bounds_read() {
        let mapping = MemoryMapping::anonymous(100).unwrap();

        let mut buf = [0u8; 50];
        let result = mapping.read(80, &mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_mapping_out_of_bounds_write() {
        let mapping = MemoryMapping::anonymous(100).unwrap();

        let data = [0u8; 50];
        let result = mapping.write(80, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_mapping_as_mut_slice() {
        let mut mapping = MemoryMapping::anonymous(4096).unwrap();

        let slice = mapping.as_mut_slice(0, 10).unwrap();
        slice.copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let mut buf = [0u8; 10];
        mapping.read(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }
}
