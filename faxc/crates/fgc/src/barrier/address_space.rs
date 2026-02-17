//! Address Space Management - Multi-Mapping Virtual Memory
//!
//! Address space module mengelola virtual address mapping untuk colored pointers.
//! Teknik multi-mapping memungkinkan physical memory yang sama diakses via
//! multiple virtual addresses.
//!
//! Virtual Address Layout (64-bit):
//! ```
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
//! 1. Pointer dengan bit Marked0/Marked1 tetap menunjuk ke object yang sama
//! 2. Tidak perlu mengubah warna pointer saat flip mark bits
//! 3. Hardware MMU menangani translation secara otomatis
//! 4. Zero software overhead untuk address translation
//!
//! Implementation:
//! Menggunakan mmap/MAP_FIXED untuk map physical memory ke multiple virtual addresses.
//! Setiap view memiliki offset tetap dari base address.

use crate::error::{FgcError, Result};
use std::collections::HashMap;

/// Base addresses untuk setiap view
const REMAPPED_BASE: usize = 0x0000_0000_0000_0000;
const MARKED0_BASE: usize = 0x0001_0000_0000_0000;
const MARKED1_BASE: usize = 0x0002_0000_0000_0000;

/// View size (16TB per view)
const VIEW_SIZE: usize = 16 * 1024 * 1024 * 1024 * 1024; // 16TB

/// AddressSpace - mengelola virtual address space dan multi-mapping
///
/// Mengatur 3 views dari heap: Remapped, Marked0, Marked1.
/// Setiap region di-map ke ketiga views secara simultan.
///
/// # Examples
///
/// ```rust
/// let mut addr_space = AddressSpace::new();
/// addr_space.map_region(0x1000, 0x200000).unwrap(); // Map 2MB region
///
/// // Get pointer di different views
/// let remapped = addr_space.get_view(0x1000, View::Remapped);
/// let marked0 = addr_space.get_view(0x1000, View::Marked0);
/// ```
pub struct AddressSpace {
    /// Mapped regions: physical address -> size
    regions: HashMap<usize, usize>,

    /// Memory mapping handles untuk cleanup
    mappings: Vec<MemoryMapping>,

    /// Total mapped size
    total_mapped: usize,
}

impl AddressSpace {
    /// Create new address space manager
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            mappings: Vec::new(),
            total_mapped: 0,
        }
    }

    /// Setup multi-mapping untuk region
    ///
    /// Map physical memory ke 3 virtual addresses (Remapped, Marked0, Marked1).
    ///
    /// # Arguments
    /// * `physical_addr` - Physical address region
    /// * `size` - Size region dalam bytes
    ///
    /// # Returns
    /// Result atau error jika mapping gagal
    pub fn map_region(&mut self, physical_addr: usize, size: usize) -> Result<()> {
        // Check jika sudah mapped
        if self.regions.contains_key(&physical_addr) {
            return Err(FgcError::VirtualMemoryError(
                "Region already mapped".to_string()
            ));
        }

        // Map ke ketiga views
        // Note: Dalam implementasi nyata, ini menggunakan mmap:
        // - mmap(remapped_base + offset, size, ...)
        // - mmap(marked0_base + offset, size, ...)
        // - mmap(marked1_base + offset, size, ...)
        // Semua map ke physical address yang sama

        let remapped_addr = self.convert_to_view(physical_addr, View::Remapped);
        let marked0_addr = self.convert_to_view(physical_addr, View::Marked0);
        let marked1_addr = self.convert_to_view(physical_addr, View::Marked1);

        // Create memory mappings (dummy untuk sekarang)
        self.mappings.push(MemoryMapping {
            virtual_addr: remapped_addr,
            physical_addr,
            size,
            view: View::Remapped,
        });

        self.mappings.push(MemoryMapping {
            virtual_addr: marked0_addr,
            physical_addr,
            size,
            view: View::Marked0,
        });

        self.mappings.push(MemoryMapping {
            virtual_addr: marked1_addr,
            physical_addr,
            size,
            view: View::Marked1,
        });

        // Track region
        self.regions.insert(physical_addr, size);
        self.total_mapped += size;

        Ok(())
    }

    /// Unmap region dari semua views
    ///
    /// # Arguments
    /// * `physical_addr` - Physical address region untuk unmap
    pub fn unmap_region(&mut self, physical_addr: usize) -> Result<()> {
        let size = self.regions.remove(&physical_addr).ok_or_else(|| {
            FgcError::VirtualMemoryError("Region not found".to_string())
        })?;

        // Remove mappings
        self.mappings.retain(|m| m.physical_addr != physical_addr);
        self.total_mapped -= size;

        // Note: Dalam implementasi nyata, ini call munmap
        Ok(())
    }

    /// Convert pointer dari satu view ke view lain
    ///
    /// # Arguments
    /// * `address` - Address dalam satu view
    /// * `target` - Target view
    ///
    /// # Returns
    /// Address di target view
    pub fn convert_view(&self, address: usize, target: View) -> usize {
        // Extract offset dari base
        let offset = address & ((1 << 44) - 1); // 44 bit offset

        // Convert ke target view
        match target {
            View::Remapped => REMAPPED_BASE + offset,
            View::Marked0 => MARKED0_BASE + offset,
            View::Marked1 => MARKED1_BASE + offset,
        }
    }

    /// Get address untuk specific view dari physical address
    pub fn get_view(&self, physical_addr: usize, view: View) -> Option<usize> {
        if !self.regions.contains_key(&physical_addr) {
            return None;
        }

        Some(self.convert_to_view(physical_addr, view))
    }

    /// Convert physical address ke view address
    fn convert_to_view(&self, physical_addr: usize, view: View) -> usize {
        let offset = physical_addr & ((1 << 44) - 1);
        match view {
            View::Remapped => REMAPPED_BASE + offset,
            View::Marked0 => MARKED0_BASE + offset,
            View::Marked1 => MARKED1_BASE + offset,
        }
    }

    /// Check jika physical address mapped
    pub fn is_mapped(&self, physical_addr: usize) -> bool {
        self.regions.contains_key(&physical_addr)
    }

    /// Get total mapped size
    pub fn total_mapped(&self) -> usize {
        self.total_mapped
    }

    /// Get mapped regions count
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Unmap semua regions (cleanup)
    pub fn unmap_all(&mut self) {
        self.regions.clear();
        self.mappings.clear();
        self.total_mapped = 0;
    }
}

impl Default for AddressSpace {
    fn default() -> Self {
        Self::new()
    }
}

/// View type untuk multi-mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Remapped view (normal access)
    Remapped,
    /// Marked0 view (GC cycle even)
    Marked0,
    /// Marked1 view (GC cycle odd)
    Marked1,
}

/// Memory mapping record untuk tracking
#[derive(Debug)]
struct MemoryMapping {
    /// Virtual address
    virtual_addr: usize,
    /// Physical address
    physical_addr: usize,
    /// Size mapping
    size: usize,
    /// View type
    view: View,
}

/// Helper untuk convert colored pointer ke view address
pub fn pointer_to_view(pointer: usize, view: View) -> usize {
    let offset = pointer & ((1 << 44) - 1);
    match view {
        View::Remapped => REMAPPED_BASE + offset,
        View::Marked0 => MARKED0_BASE + offset,
        View::Marked1 => MARKED1_BASE + offset,
    }
}

/// Helper untuk extract physical address dari view address
pub fn view_to_physical(view_addr: usize) -> usize {
    view_addr & ((1 << 44) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_conversion() {
        let addr_space = AddressSpace::new();

        let physical = 0x1234;
        let remapped = addr_space.convert_view(physical, View::Remapped);
        let marked0 = addr_space.convert_view(physical, View::Marked0);

        assert_eq!(remapped, 0x0000_0000_0000_1234);
        assert_eq!(marked0, 0x0001_0000_0000_1234);
    }

    #[test]
    fn test_view_roundtrip() {
        let addr_space = AddressSpace::new();

        let physical = 0x5678;
        let marked0 = addr_space.convert_view(physical, View::Marked0);

        // Extract offset kembali
        let recovered = marked0 & ((1 << 44) - 1);
        assert_eq!(recovered, physical);
    }
}
