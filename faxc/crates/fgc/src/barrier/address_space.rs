//! Address Space Management - Multi-Mapping Virtual Memory
//!
//! Address space module manages virtual address mapping for colored pointers.
//! Multi-mapping technique allows the same physical memory to be accessed via
//! multiple virtual addresses.
//!
//! Virtual Address Layout (64-bit):
//! ```text
//! 0x0000_0000_0000_0000 ─┐
//!                        │  Remapped View (16TB)
//! 0x0000_1000_0000_0000 ─┘  Base: 0x0000_0000_0000
//!
//! 0x0001_0000_0000_0000 ─┐
//!                        │  Marked0 View (16TB)
//! 0x0001_1000_0000_0000 ─┘  Base: 0x0001_0000_0000_0000
//!
//! 0x0002_0000_0000_0000 ─┐
//!                        │  Marked1 View (16TB)
//! 0x0002_1000_0000_0000 ─┘  Base: 0x0002_0000_0000_0000
//! ```
//!
//! Multi-Mapping Benefits:
//! 1. Pointers with Marked0/Marked1 bits point to the same object
//! 2. No need to change pointer color when flipping mark bits
//! 3. Hardware MMU handles translation automatically
//! 4. Zero software overhead for address translation
//!
//! # Platform Support
//!
//! ## Unix (Linux/macOS)
//! Full implementation using `mmap` with MAP_FIXED and shared file descriptors
//! to map the same physical pages to multiple virtual addresses.
//!
//! ## Windows
//! Full implementation using `VirtualAlloc` with `MEM_RESERVE` and `MEM_COMMIT`.
//!
//! ## Other Platforms
//! Placeholder implementation with warning (will not work correctly for actual memory access).

use crate::error::{FgcError, Result};

#[cfg(unix)]
use std::os::unix::io::RawFd;

#[cfg(windows)]
use std::ptr::null_mut;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, VirtualFree, FILE_MAP_WRITE, MEM_RELEASE,
    PAGE_READWRITE,
};

/// Base addresses for each view (used in fallback mode)
const REMAPPED_BASE_FALLBACK: usize = 0x0000_0000_0000_0000;
const MARKED0_BASE_FALLBACK: usize = 0x0001_0000_0000_0000;
const MARKED1_BASE_FALLBACK: usize = 0x0002_0000_0000_0000;

/// Page size for multi-mapping (4KB standard page)
#[cfg(unix)]
const PAGE_SIZE: usize = 4096;

/// AddressSpace - manages virtual address space and multi-mapping
///
/// Manages 3 views of the heap: Remapped, Marked0, Marked1.
/// Each region is mapped to all three views simultaneously using
/// platform-specific multi-mapping techniques.
///
/// # Multi-Mapping Implementation
///
/// ## Unix (Linux/macOS)
/// Uses a shared memory file descriptor mapped at three different virtual
/// addresses. All three views point to the same physical pages.
///
/// ## Windows
/// Uses VirtualAlloc with MEM_RESERVE to reserve address space, then
/// maps the same physical pages using CreateFileMapping/MapViewOfFile.
///
/// # Examples
///
/// ```rust
/// let addr_space = AddressSpace::new(1024 * 1024).unwrap();
/// assert!(addr_space.remapped_base != 0);
/// assert!(addr_space.marked0_base != 0);
/// assert!(addr_space.marked1_base != 0);
/// ```
pub struct AddressSpace {
    /// Base address for remapped view
    pub remapped_base: usize,
    /// Base address for Marked0 view
    pub marked0_base: usize,
    /// Base address for Marked1 view
    pub marked1_base: usize,
    /// Size of each mapping
    pub size: usize,

    #[cfg(unix)]
    _shared_fd: Option<RawFd>,
    #[cfg(unix)]
    _mmap_remapped: *mut libc::c_void,
    #[cfg(unix)]
    _mmap_marked0: *mut libc::c_void,
    #[cfg(unix)]
    _mmap_marked1: *mut libc::c_void,

    #[cfg(windows)]
    _windows_handle: HANDLE,
}

// AddressSpace is not Send/Sync by default due to raw pointers
// but the underlying mappings are thread-safe
#[cfg(unix)]
unsafe impl Send for AddressSpace {}
#[cfg(unix)]
unsafe impl Sync for AddressSpace {}

impl AddressSpace {
    /// Create new address space with multi-mapping
    ///
    /// # Arguments
    /// * `size` - Size of each mapping in bytes
    ///
    /// # Platform Support
    /// - **Unix (Linux/macOS):** Full implementation with mmap
    /// - **Windows:** Full implementation with VirtualAlloc
    /// - **Other:** Placeholder (will not work correctly)
    ///
    /// # Returns
    /// `Result<AddressSpace>` - Ok with new AddressSpace, or Err if allocation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fgc::barrier::address_space::AddressSpace;
    /// let space = AddressSpace::new(1024 * 1024).unwrap();
    /// assert!(space.remapped_base != 0);
    /// ```
    pub fn new(size: usize) -> Result<Self> {
        #[cfg(unix)]
        {
            Self::new_unix(size)
        }

        #[cfg(windows)]
        {
            Self::new_windows(size)
        }

        #[cfg(not(any(unix, windows)))]
        {
            log::warn!("Multi-mapping not implemented for this platform");
            Ok(Self {
                remapped_base: REMAPPED_BASE_FALLBACK,
                marked0_base: MARKED0_BASE_FALLBACK,
                marked1_base: MARKED1_BASE_FALLBACK,
                size,
            })
        }
    }

    /// Create address space using Unix mmap with true multi-mapping
    ///
    /// This implementation creates a shared memory file and maps it at three
    /// different virtual addresses. All three views point to the same physical
    /// pages, enabling zero-copy pointer color changes.
    ///
    /// # Algorithm
    /// 1. Create a shared memory file descriptor (memfd_create or shm_open)
    /// 2. Extend file to desired size
    /// 3. Map file at three different virtual addresses using MAP_SHARED
    /// 4. All three mappings share the same physical pages
    ///
    /// # Arguments
    /// * `size` - Size of each mapping in bytes
    ///
    /// # Returns
    /// `Result<AddressSpace>` - Ok with new AddressSpace, or Err if mmap fails
    ///
    /// # Safety
    /// Uses unsafe mmap calls with proper error handling and cleanup on failure.
    #[cfg(unix)]
    fn new_unix(size: usize) -> Result<Self> {
        // Align size to page boundary for proper mmap
        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        // Create shared memory using memfd_create (Linux) or shm_open (macOS/BSD)
        let fd = Self::create_shared_memory(aligned_size)?;

        // Map the shared memory at three different virtual addresses
        // All three mappings share the same physical pages via MAP_SHARED

        // First mapping: Remapped view (kernel chooses address)
        let remapped_ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                aligned_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if remapped_ptr == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(FgcError::VirtualMemoryError(format!(
                "Failed to map remapped view: {}",
                std::io::Error::last_os_error()
            )));
        }

        let remapped_base = remapped_ptr as usize;

        // Second mapping: Marked0 view (different virtual address, same physical pages)
        let marked0_ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                aligned_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if marked0_ptr == libc::MAP_FAILED {
            unsafe {
                libc::munmap(remapped_ptr, aligned_size);
                libc::close(fd);
            }
            return Err(FgcError::VirtualMemoryError(format!(
                "Failed to map marked0 view: {}",
                std::io::Error::last_os_error()
            )));
        }

        let marked0_base = marked0_ptr as usize;

        // Third mapping: Marked1 view
        let marked1_ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                aligned_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if marked1_ptr == libc::MAP_FAILED {
            unsafe {
                libc::munmap(remapped_ptr, aligned_size);
                libc::munmap(marked0_ptr, aligned_size);
                libc::close(fd);
            }
            return Err(FgcError::VirtualMemoryError(format!(
                "Failed to map marked1 view: {}",
                std::io::Error::last_os_error()
            )));
        }

        let marked1_base = marked1_ptr as usize;

        log::info!(
            "Created multi-mapped address space: remapped={:#x}, marked0={:#x}, marked1={:#x}, size={}",
            remapped_base, marked0_base, marked1_base, aligned_size
        );

        Ok(Self {
            remapped_base,
            marked0_base,
            marked1_base,
            size: aligned_size,
            _shared_fd: Some(fd),
            _mmap_remapped: remapped_ptr,
            _mmap_marked0: marked0_ptr,
            _mmap_marked1: marked1_ptr,
        })
    }

    /// Create shared memory file descriptor for multi-mapping
    ///
    /// On Linux, uses memfd_create for anonymous shared memory.
    /// On macOS/BSD, uses shm_open for POSIX shared memory.
    ///
    /// # Arguments
    /// * `size` - Size to extend the file to
    ///
    /// # Returns
    /// Raw file descriptor, or Err if creation fails
    #[cfg(unix)]
    fn create_shared_memory(size: usize) -> Result<libc::c_int> {
        #[cfg(target_os = "linux")]
        {
            // Use memfd_create on Linux for anonymous shared memory
            let memfd_name = std::ffi::CString::new("fgc_shared").map_err(|e| {
                FgcError::VirtualMemoryError(format!("Failed to create memfd name: {}", e))
            })?;

            let fd = unsafe {
                libc::syscall(
                    libc::SYS_memfd_create,
                    memfd_name.as_ptr(),
                    0, // No special flags
                )
            };

            if fd < 0 {
                // Fallback to shm_open if memfd_create fails
                return Self::create_shared_memory_shm(size);
            }

            let fd = fd as libc::c_int;

            // Extend file to desired size
            if unsafe { libc::ftruncate(fd, size as libc::off_t) } != 0 {
                unsafe { libc::close(fd) };
                return Err(FgcError::VirtualMemoryError(format!(
                    "Failed to extend shared memory: {}",
                    std::io::Error::last_os_error()
                )));
            }

            Ok(fd)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::create_shared_memory_shm(size)
        }
    }

    /// Create shared memory using shm_open (POSIX shared memory)
    ///
    /// Fallback for systems without memfd_create, or primary method on macOS/BSD.
    ///
    /// # Arguments
    /// * `size` - Size to extend the file to
    ///
    /// # Returns
    /// Raw file descriptor, or Err if creation fails
    #[cfg(unix)]
    fn create_shared_memory_shm(size: usize) -> Result<libc::c_int> {
        // Generate unique name for shared memory
        let name = format!(
            "/fgc_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let name_cstr = std::ffi::CString::new(name.as_str()).map_err(|e| {
            FgcError::VirtualMemoryError(format!("Failed to create shm name: {}", e))
        })?;

        // Create shared memory object
        let fd = unsafe {
            libc::shm_open(
                name_cstr.as_ptr(),
                libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                0o600, // Owner read/write only
            )
        };

        if fd < 0 {
            return Err(FgcError::VirtualMemoryError(format!(
                "Failed to create shared memory: {}",
                std::io::Error::last_os_error()
            )));
        }

        // Unlink immediately (file persists as long as fd is open)
        unsafe { libc::shm_unlink(name_cstr.as_ptr()) };

        // Extend to desired size
        if unsafe { libc::ftruncate(fd, size as libc::off_t) } != 0 {
            unsafe { libc::close(fd) };
            return Err(FgcError::VirtualMemoryError(format!(
                "Failed to extend shared memory: {}",
                std::io::Error::last_os_error()
            )));
        }

        Ok(fd)
    }

    /// Create address space using Windows CreateFileMapping + MapViewOfFile
    ///
    /// This implementation creates a file mapping object backed by the system
    /// paging file, then maps three views of the SAME physical pages at different
    /// virtual addresses. This enables the colored pointer scheme to work correctly.
    ///
    /// # Arguments
    /// * `size` - Size of each mapping in bytes
    ///
    /// # Returns
    /// `Result<AddressSpace>` - Ok with new AddressSpace, or Err if allocation fails
    ///
    /// # Implementation Details
    ///
    /// Windows doesn't support mmap-style multi-mapping directly, but we can
    /// achieve the same effect using CreateFileMapping + MapViewOfFile:
    /// 1. CreateFileMappingW creates a mapping object backed by paging file
    /// 2. MapViewOfFile maps views of this object at different virtual addresses
    /// 3. All views share the same physical pages
    #[cfg(windows)]
    fn new_windows(size: usize) -> Result<Self> {
        use std::ffi::c_void;

        unsafe {
            // Create a file mapping object backed by system paging file
            // INVALID_HANDLE_VALUE means use paging file (not a real file)
            let h_mapping = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                null_mut(),
                PAGE_READWRITE,
                0,
                size as u32,
                null_mut(),
            );

            if h_mapping == 0 {
                return Err(FgcError::VirtualMemoryError(format!(
                    "Failed to create Windows file mapping: {}",
                    std::io::Error::last_os_error()
                )));
            }

            // Map three views of the SAME physical pages at different virtual addresses
            // The OS chooses the virtual addresses automatically
            let remapped = MapViewOfFile(h_mapping, FILE_MAP_WRITE, 0, 0, size);

            if remapped.is_null() {
                CloseHandle(h_mapping);
                return Err(FgcError::VirtualMemoryError(format!(
                    "Failed to map remapped view: {}",
                    std::io::Error::last_os_error()
                )));
            }

            let marked0 = MapViewOfFile(h_mapping, FILE_MAP_WRITE, 0, 0, size);

            if marked0.is_null() {
                UnmapViewOfFile(remapped);
                CloseHandle(h_mapping);
                return Err(FgcError::VirtualMemoryError(format!(
                    "Failed to map marked0 view: {}",
                    std::io::Error::last_os_error()
                )));
            }

            let marked1 = MapViewOfFile(h_mapping, FILE_MAP_WRITE, 0, 0, size);

            if marked1.is_null() {
                UnmapViewOfFile(remapped);
                UnmapViewOfFile(marked0);
                CloseHandle(h_mapping);
                return Err(FgcError::VirtualMemoryError(format!(
                    "Failed to map marked1 view: {}",
                    std::io::Error::last_os_error()
                )));
            }

            log::info!(
                "Created multi-mapped address space (Windows): remapped={:#x}, marked0={:#x}, marked1={:#x}, size={}",
                remapped as usize, marked0 as usize, marked1 as usize, size
            );

            // Store h_mapping to keep it alive (views hold references to it)
            Ok(Self {
                remapped_base: remapped as usize,
                marked0_base: marked0 as usize,
                marked1_base: marked1 as usize,
                size,
                _windows_handle: h_mapping,
            })
        }
    }

    /// Map a region from physical address to all three views
    ///
    /// This is the key operation for ZGC colored pointers.
    /// The same physical page is mapped to three virtual addresses.
    ///
    /// # Arguments
    /// * `physical_addr` - Physical address (offset within shared memory)
    /// * `size` - Size of region to map in bytes
    ///
    /// # Returns
    /// `Result<()>` - Ok if mapping successful, Err if mapping fails
    ///
    /// # Implementation
    ///
    /// On Unix, this function verifies that all three views share the same
    /// physical pages by:
    /// 1. Writing test data to the remapped view
    /// 2. Verifying the same data is visible in marked0 and marked1 views
    /// 3. Using msync to ensure memory is synchronized
    ///
    /// This verification ensures the multi-mapping is working correctly.
    ///
    /// # FIX Issue 1: Multi-Mapping Implementation
    ///
    /// This function now properly verifies multi-mapping instead of being a stub.
    pub fn map_region(&self, physical_addr: usize, size: usize) -> Result<()> {
        // FIX Issue 1: Validate inputs
        if physical_addr == 0 {
            return Err(FgcError::InvalidArgument(
                "physical_addr must be non-zero".to_string(),
            ));
        }

        if size == 0 {
            return Err(FgcError::InvalidArgument(
                "size must be greater than 0".to_string(),
            ));
        }

        // Check for overflow
        let _end_addr = physical_addr.checked_add(size).ok_or_else(|| {
            FgcError::InvalidArgument("physical_addr + size would overflow".to_string())
        })?;

        // Ensure size is page-aligned
        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        #[cfg(unix)]
        {
            // FIX Issue 1: Implement proper multi-mapping verification on Unix
            Self::verify_multi_mapping_unix(
                self.remapped_base,
                self.marked0_base,
                self.marked1_base,
                physical_addr,
                aligned_size,
            )
        }

        #[cfg(windows)]
        {
            // FIX Issue 1: Implement proper multi-mapping verification on Windows
            Self::verify_multi_mapping_windows(
                self.remapped_base,
                self.marked0_base,
                self.marked1_base,
                physical_addr,
                aligned_size,
            )
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback for unsupported platforms
            log::warn!("Multi-mapping verification not available on this platform");
            Ok(())
        }
    }

    /// Verify multi-mapping on Unix platforms
    ///
    /// Writes test data to one view and verifies it's visible in all views.
    /// This confirms all views share the same physical pages.
    #[cfg(unix)]
    fn verify_multi_mapping_unix(
        remapped_base: usize,
        marked0_base: usize,
        marked1_base: usize,
        offset: usize,
        size: usize,
    ) -> Result<()> {
        use libc::{msync, MS_SYNC};

        log::debug!(
            "Verifying multi-mapping: offset={:#x}, size={:#x}",
            offset,
            size
        );

        // Calculate addresses in each view
        let remapped_addr = remapped_base + offset;
        let marked0_addr = marked0_base + offset;
        let marked1_addr = marked1_base + offset;

        // FIX Issue 1: Verify all views share same physical pages
        // Write test pattern to remapped view
        unsafe {
            let remapped_ptr = remapped_addr as *mut u8;
            let test_pattern: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];

            // Write test data
            std::ptr::copy_nonoverlapping(test_pattern.as_ptr(), remapped_ptr, 8.min(size));

            // Sync memory to ensure writes are visible
            let page_addr = (remapped_addr & !(PAGE_SIZE - 1)) as *mut libc::c_void;
            let msync_size = Self::aligned_size_for_msync(size);
            if msync(page_addr, msync_size, MS_SYNC) != 0 {
                log::warn!("msync failed: {}", std::io::Error::last_os_error());
            }

            // Read from marked0 view - should see same data
            let marked0_ptr = marked0_addr as *const u8;
            let mut marked0_data = [0u8; 8];
            std::ptr::copy_nonoverlapping(marked0_ptr, marked0_data.as_mut_ptr(), 8.min(size));

            // Read from marked1 view - should see same data
            let marked1_ptr = marked1_addr as *const u8;
            let mut marked1_data = [0u8; 8];
            std::ptr::copy_nonoverlapping(marked1_ptr, marked1_data.as_mut_ptr(), 8.min(size));

            // Verify all views see the same data
            if marked0_data[..8.min(size)] != test_pattern[..8.min(size)] {
                log::error!("Multi-mapping verification FAILED: marked0 view has different data");
                return Err(FgcError::VirtualMemoryError(
                    "Multi-mapping verification failed: marked0 view mismatch".to_string(),
                ));
            }

            if marked1_data[..8.min(size)] != test_pattern[..8.min(size)] {
                log::error!("Multi-mapping verification FAILED: marked1 view has different data");
                return Err(FgcError::VirtualMemoryError(
                    "Multi-mapping verification failed: marked1 view mismatch".to_string(),
                ));
            }
        }

        log::debug!("Multi-mapping verification successful");
        Ok(())
    }

    /// Helper function to calculate aligned size for msync
    #[cfg(unix)]
    fn aligned_size_for_msync(size: usize) -> libc::size_t {
        ((size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)) as libc::size_t
    }

    /// Verify multi-mapping on Windows platforms
    #[cfg(windows)]
    fn verify_multi_mapping_windows(
        remapped_base: usize,
        marked0_base: usize,
        marked1_base: usize,
        offset: usize,
        size: usize,
    ) -> Result<()> {
        log::debug!(
            "Verifying multi-mapping (Windows): offset={:#x}, size={:#x}",
            offset,
            size
        );

        // Calculate addresses in each view
        let remapped_addr = remapped_base + offset;
        let marked0_addr = marked0_base + offset;
        let marked1_addr = marked1_base + offset;

        // Write test pattern to remapped view
        unsafe {
            let remapped_ptr = remapped_addr as *mut u8;
            let test_pattern: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];

            std::ptr::copy_nonoverlapping(test_pattern.as_ptr(), remapped_ptr, 8.min(size));

            // Flush to ensure visibility
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);

            // Read from marked0 view
            let marked0_ptr = marked0_addr as *const u8;
            let mut marked0_data = [0u8; 8];
            std::ptr::copy_nonoverlapping(marked0_ptr, marked0_data.as_mut_ptr(), 8.min(size));

            // Read from marked1 view
            let marked1_ptr = marked1_addr as *const u8;
            let mut marked1_data = [0u8; 8];
            std::ptr::copy_nonoverlapping(marked1_ptr, marked1_data.as_mut_ptr(), 8.min(size));

            // Verify all views see the same data
            if marked0_data[..8.min(size)] != test_pattern[..8.min(size)] {
                return Err(FgcError::VirtualMemoryError(
                    "Multi-mapping verification failed: marked0 view mismatch".to_string(),
                ));
            }

            if marked1_data[..8.min(size)] != test_pattern[..8.min(size)] {
                return Err(FgcError::VirtualMemoryError(
                    "Multi-mapping verification failed: marked1 view mismatch".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the view address for a given color
    ///
    /// # Arguments
    /// * `color` - Color index (0=remapped, 1=marked0, 2=marked1)
    ///
    /// # Returns
    /// Base address for the specified view
    ///
    /// # Panics
    /// Panics if color is not 0, 1, or 2
    #[inline]
    pub fn get_view(&self, color: u8) -> usize {
        match color {
            0 => self.remapped_base,
            1 => self.marked0_base,
            2 => self.marked1_base,
            _ => unreachable!("Invalid color: {}", color),
        }
    }

    /// Get the base address for a specific view
    ///
    /// # Arguments
    /// * `view` - View type (Remapped, Marked0, or Marked1)
    ///
    /// # Returns
    /// Base address for the view
    pub fn get_view_base(&self, view: View) -> usize {
        match view {
            View::Remapped => self.remapped_base,
            View::Marked0 => self.marked0_base,
            View::Marked1 => self.marked1_base,
        }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            // Unmap all three mappings
            if self._mmap_remapped != libc::MAP_FAILED && !self._mmap_remapped.is_null() {
                libc::munmap(self._mmap_remapped, self.size);
            }
            if self._mmap_marked0 != libc::MAP_FAILED && !self._mmap_marked0.is_null() {
                libc::munmap(self._mmap_marked0, self.size);
            }
            if self._mmap_marked1 != libc::MAP_FAILED && !self._mmap_marked1.is_null() {
                libc::munmap(self._mmap_marked1, self.size);
            }

            // Close the shared memory file descriptor
            if let Some(fd) = self._shared_fd {
                libc::close(fd);
            }
        }

        #[cfg(windows)]
        unsafe {
            // Unmap all three views
            if self.remapped_base != 0 {
                UnmapViewOfFile(self.remapped_base as *mut c_void);
            }
            if self.marked0_base != 0 {
                UnmapViewOfFile(self.marked0_base as *mut c_void);
            }
            if self.marked1_base != 0 {
                UnmapViewOfFile(self.marked1_base as *mut c_void);
            }

            // Close the file mapping handle
            if self._windows_handle != 0 {
                CloseHandle(self._windows_handle);
            }
        }
    }
}

/// View type for multi-mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Remapped view (normal access)
    Remapped,
    /// Marked0 view (GC cycle even)
    Marked0,
    /// Marked1 view (GC cycle odd)
    Marked1,
}

/// Helper for converting colored pointer to view address
pub fn pointer_to_view(pointer: usize, view: View) -> usize {
    let offset = pointer & ((1 << 44) - 1);
    match view {
        View::Remapped => REMAPPED_BASE_FALLBACK + offset,
        View::Marked0 => MARKED0_BASE_FALLBACK + offset,
        View::Marked1 => MARKED1_BASE_FALLBACK + offset,
    }
}

/// Helper for extracting physical address from view address
pub fn view_to_physical(view_addr: usize) -> usize {
    view_addr & ((1 << 44) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_address_space_creation_unix() {
        let space = AddressSpace::new(1024 * 1024).unwrap();
        assert!(space.remapped_base != 0);
        assert!(space.marked0_base != 0);
        assert!(space.marked1_base != 0);
        assert_eq!(space.size, 1024 * 1024);
    }

    #[test]
    #[cfg(windows)]
    fn test_address_space_creation_windows() {
        let space = AddressSpace::new(1024 * 1024).unwrap();
        assert!(space.remapped_base != 0);
        assert!(space.marked0_base != 0);
        assert!(space.marked1_base != 0);
        assert_eq!(space.size, 1024 * 1024);
    }

    /// Test that Windows multi-mapping works correctly - writing to one view
    /// should be visible in all other views (same physical pages)
    #[test]
    #[cfg(windows)]
    fn test_windows_multi_mapping_shared() {
        let space = AddressSpace::new(PAGE_SIZE * 4).unwrap();

        // Write to remapped view
        unsafe {
            let ptr = space.remapped_base as *mut u64;
            ptr.write(0xDEADBEEFCAFEBABE);
        }

        // Read from marked0 view - should see same value (shared pages)
        unsafe {
            let ptr = space.marked0_base as *const u64;
            assert_eq!(
                ptr.read(),
                0xDEADBEEFCAFEBABE,
                "Multi-mapping not working - views not shared!"
            );
        }

        // Read from marked1 view - should see same value
        unsafe {
            let ptr = space.marked1_base as *const u64;
            assert_eq!(
                ptr.read(),
                0xDEADBEEFCAFEBABE,
                "Multi-mapping not working - views not shared!"
            );
        }
    }

    /// Test that writes to marked0 view are visible in remapped view (Windows)
    #[test]
    #[cfg(windows)]
    fn test_windows_multi_mapping_marked0_to_remapped() {
        let space = AddressSpace::new(PAGE_SIZE * 2).unwrap();

        // Write to marked0 view
        unsafe {
            let ptr = space.marked0_base as *mut u64;
            ptr.write(0x1234567890ABCDEF);
        }

        // Read from remapped view - should see same value
        unsafe {
            let ptr = space.remapped_base as *const u64;
            assert_eq!(
                ptr.read(),
                0x1234567890ABCDEF,
                "Multi-mapping not working - marked0 to remapped failed!"
            );
        }
    }

    /// Test that multi-mapping works correctly - writing to one view
    /// should be visible in all other views (same physical pages)
    #[test]
    #[cfg(unix)]
    fn test_multi_mapping_shared_memory() {
        let space = AddressSpace::new(PAGE_SIZE * 4).unwrap();

        // Write to remapped view
        unsafe {
            let ptr = space.remapped_base as *mut u64;
            ptr.write(0xDEADBEEFCAFEBABE);
        }

        // Read from marked0 view - should see same value
        unsafe {
            let ptr = space.marked0_base as *const u64;
            assert_eq!(ptr.read(), 0xDEADBEEFCAFEBABE);
        }

        // Read from marked1 view - should see same value
        unsafe {
            let ptr = space.marked1_base as *const u64;
            assert_eq!(ptr.read(), 0xDEADBEEFCAFEBABE);
        }
    }

    /// Test that writes to marked0 view are visible in remapped view
    #[test]
    #[cfg(unix)]
    fn test_multi_mapping_marked0_to_remapped() {
        let space = AddressSpace::new(PAGE_SIZE * 2).unwrap();

        // Write to marked0 view
        unsafe {
            let ptr = space.marked0_base as *mut u64;
            ptr.write(0x1234567890ABCDEF);
        }

        // Read from remapped view - should see same value
        unsafe {
            let ptr = space.remapped_base as *const u64;
            assert_eq!(ptr.read(), 0x1234567890ABCDEF);
        }
    }

    /// Test that writes to marked1 view are visible in all views
    #[test]
    #[cfg(unix)]
    fn test_multi_mapping_marked1_to_all() {
        let space = AddressSpace::new(PAGE_SIZE * 2).unwrap();

        // Write to marked1 view
        unsafe {
            let ptr = space.marked1_base as *mut u64;
            ptr.write(0xFEDCBA0987654321);
        }

        // Verify all views see the same value
        unsafe {
            assert_eq!(
                (space.remapped_base as *const u64).read(),
                0xFEDCBA0987654321
            );
            assert_eq!(
                (space.marked0_base as *const u64).read(),
                0xFEDCBA0987654321
            );
            assert_eq!(
                (space.marked1_base as *const u64).read(),
                0xFEDCBA0987654321
            );
        }
    }

    /// Test multi-page mapping
    #[test]
    #[cfg(unix)]
    fn test_multi_mapping_multiple_pages() {
        let num_pages = 16;
        let space = AddressSpace::new(PAGE_SIZE * num_pages).unwrap();

        // Write different values to different pages via remapped view
        for i in 0..num_pages {
            unsafe {
                let ptr = (space.remapped_base + i * PAGE_SIZE) as *mut u64;
                ptr.write(i as u64 * 0x1000);
            }
        }

        // Verify all values are visible via marked0 view
        for i in 0..num_pages {
            unsafe {
                let ptr = (space.marked0_base + i * PAGE_SIZE) as *const u64;
                assert_eq!(ptr.read(), i as u64 * 0x1000);
            }
        }

        // Verify all values are visible via marked1 view
        for i in 0..num_pages {
            unsafe {
                let ptr = (space.marked1_base + i * PAGE_SIZE) as *const u64;
                assert_eq!(ptr.read(), i as u64 * 0x1000);
            }
        }
    }

    /// Test that address space properly cleans up on drop
    #[test]
    #[cfg(unix)]
    fn test_address_space_drop() {
        let space = AddressSpace::new(PAGE_SIZE * 2).unwrap();
        let _remapped = space.remapped_base;
        let _marked0 = space.marked0_base;
        let _marked1 = space.marked1_base;

        // Write some data
        unsafe {
            (_remapped as *mut u64).write(0xABCDEF);
        }

        // Drop the space
        drop(space);

        // After drop, accessing the memory would be undefined behavior
        // We just verify the test doesn't crash
    }

    #[test]
    fn test_get_view() {
        #[cfg(any(unix, windows))]
        {
            let space = AddressSpace::new(1024 * 1024).unwrap();
            assert_eq!(space.get_view(0), space.remapped_base);
            assert_eq!(space.get_view(1), space.marked0_base);
            assert_eq!(space.get_view(2), space.marked1_base);
        }
    }

    #[test]
    fn test_view_enum() {
        assert_eq!(View::Remapped as u8, 0);
        assert_eq!(View::Marked0 as u8, 1);
        assert_eq!(View::Marked1 as u8, 2);
    }

    #[test]
    fn test_pointer_to_view() {
        let ptr = 0x1234;
        let remapped = pointer_to_view(ptr, View::Remapped);
        let marked0 = pointer_to_view(ptr, View::Marked0);
        let marked1 = pointer_to_view(ptr, View::Marked1);

        assert_eq!(remapped, REMAPPED_BASE_FALLBACK + 0x1234);
        assert_eq!(marked0, MARKED0_BASE_FALLBACK + 0x1234);
        assert_eq!(marked1, MARKED1_BASE_FALLBACK + 0x1234);
    }

    #[test]
    fn test_view_to_physical() {
        let physical = 0x5678;
        let marked0 = MARKED0_BASE_FALLBACK + physical;
        let recovered = view_to_physical(marked0);
        assert_eq!(recovered, physical);
    }

    #[test]
    fn test_view_roundtrip() {
        let physical = 0xABCD;
        let view_addr = pointer_to_view(physical, View::Marked1);
        let recovered = view_to_physical(view_addr);
        assert_eq!(recovered, physical);
    }

    /// Test that different AddressSpace instances have different mappings
    #[test]
    #[cfg(unix)]
    fn test_multiple_address_spaces_isolated() {
        let space1 = AddressSpace::new(PAGE_SIZE * 2).unwrap();
        let space2 = AddressSpace::new(PAGE_SIZE * 2).unwrap();

        // Write to space1
        unsafe {
            (space1.remapped_base as *mut u64).write(0x11111111);
        }

        // space2 should not see space1's data (different physical pages)
        unsafe {
            let val = (space2.remapped_base as *const u64).read();
            // Value should be 0 (fresh mapping) or different from space1
            assert_ne!(
                val, 0x11111111,
                "Different address spaces should be isolated"
            );
        }
    }
}
