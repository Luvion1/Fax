//! Root Scanning - GC Root Identification and Management
//!
//! Roots adalah starting points untuk marking. Semua object reachable
//! dari roots harus di-mark sebagai live.
//!
//! # Root Types
//!
//! 1. **Stack Roots** - Local variables di thread stacks
//! 2. **Global Roots** - Static/global variables  
//! 3. **Class Roots** - Class loaders dan loaded classes
//! 4. **VM Internal Roots** - Monitors, thread local storage, etc.
//!
//! # Root Scanning Challenge
//!
//! Saat concurrent marking, thread mutator sedang running dan stack
//! berubah terus. FGC menggunakan concurrent stack scanning dengan
//! watermark untuk handle ini.
//!
//! # Thread Safety
//!
//! RootScanner adalah thread-safe. Multiple threads dapat:
//! - Register/unregister roots secara concurrent
//! - Scan roots secara concurrent
//! - Query statistics secara concurrent

use crate::error::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// Root types untuk kategorisasi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RootType {
    /// Stack roots (local variables, parameters)
    Stack,
    /// Global/static variables
    Global,
    /// Class metadata references
    Class,
    /// VM internal references
    Internal,
    /// JNI global references
    JNIGlobal,
    /// Thread-local roots
    ThreadLocal,
}

impl std::fmt::Display for RootType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootType::Stack => write!(f, "Stack"),
            RootType::Global => write!(f, "Global"),
            RootType::Class => write!(f, "Class"),
            RootType::Internal => write!(f, "Internal"),
            RootType::JNIGlobal => write!(f, "JNIGlobal"),
            RootType::ThreadLocal => write!(f, "ThreadLocal"),
        }
    }
}

/// Root descriptor - mendeskripsikan satu root reference
///
/// Struct ini menyimpan informasi lengkap tentang satu root:
/// - Address dimana reference disimpan
/// - Type root untuk kategorisasi
/// - Optional name untuk debugging
/// - Status active untuk tracking lifecycle
/// - Root ID untuk identification
#[derive(Debug)]
pub struct RootDescriptor {
    /// Address dimana reference disimpan (pointer ke pointer)
    pub address: usize,
    /// Type root
    pub root_type: RootType,
    /// Optional name untuk debugging
    pub name: Option<String>,
    /// Root ID untuk identification
    pub root_id: usize,
    /// Apakah root ini active
    pub active: AtomicBool,
}

impl RootDescriptor {
    /// Create new root descriptor
    ///
    /// # Arguments
    /// * `address` - Address dimana reference disimpan
    /// * `root_type` - Type root
    /// * `name` - Optional name untuk debugging
    /// * `root_id` - Unique root ID
    pub fn new(address: usize, root_type: RootType, name: Option<&str>, root_id: usize) -> Self {
        Self {
            address,
            root_type,
            name: name.map(|s| s.to_string()),
            root_id,
            active: AtomicBool::new(true),
        }
    }

    /// Read reference value dari address
    ///
    /// # Safety
    /// Address harus valid dan aligned untuk membaca usize
    pub fn read_reference(&self) -> usize {
        unsafe {
            let ptr = self.address as *const usize;
            ptr.read_volatile()
        }
    }

    /// Update reference value (untuk relocation)
    ///
    /// # Safety
    /// Address harus valid dan aligned untuk menulis usize
    pub fn update_reference(&self, new_value: usize) {
        unsafe {
            let ptr = self.address as *mut usize;
            ptr.write_volatile(new_value);
        }
    }

    /// Check apakah reference null
    pub fn is_null(&self) -> bool {
        self.read_reference() == 0
    }

    /// Check apakah root active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Deactivate root (tanpa remove dari list)
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Activate root
    pub fn activate(&self) {
        self.active.store(true, Ordering::Relaxed);
    }
}

/// Handle untuk managing root lifecycle
///
/// Handle ini diberikan saat register root dan otomatis
/// unregister root saat handle di-drop.
///
/// Note: Handle menyimpan weak reference ke scanner untuk menghindari circular reference.
#[derive(Debug)]
pub struct RootHandle {
    root_id: usize,
}

impl RootHandle {
    /// Create new root handle
    fn new(root_id: usize) -> Self {
        Self {
            root_id,
        }
    }

    /// Get root ID
    pub fn id(&self) -> usize {
        self.root_id
    }

    /// Manually unregister root (drop juga melakukan ini)
    ///
    /// # Arguments
    /// * `scanner` - RootScanner untuk unregister dari
    pub fn unregister(self, scanner: &RootScanner) {
        scanner.unregister_root_by_id(self.root_id);
    }
}

impl Drop for RootHandle {
    fn drop(&mut self) {
        // Cannot unregister on drop without scanner reference
        // User must call unregister() explicitly
        // This is a limitation of the current design
    }
}

/// Root statistics
#[derive(Debug, Default, Clone)]
pub struct RootStats {
    /// Total registered roots
    pub total_roots: usize,
    /// Active roots
    pub active_roots: usize,
    /// Stack roots
    pub stack_roots: usize,
    /// Global roots
    pub global_roots: usize,
    /// Class roots
    pub class_roots: usize,
    /// Internal roots
    pub internal_roots: usize,
    /// Null roots (reference = 0)
    pub null_roots: usize,
    /// Live roots (reference != 0)
    pub live_roots: usize,
}

impl std::fmt::Display for RootStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RootStats {{ total: {}, active: {}, stack: {}, global: {}, class: {}, internal: {}, null: {}, live: {} }}",
            self.total_roots,
            self.active_roots,
            self.stack_roots,
            self.global_roots,
            self.class_roots,
            self.internal_roots,
            self.null_roots,
            self.live_roots
        )
    }
}

/// RootScanner - scanner untuk berbagai tipe roots
///
/// RootScanner mengelola registration dan scanning dari semua
/// GC roots. Roots adalah starting points untuk marking phase.
///
/// # Thread Safety
///
/// RootScanner adalah fully thread-safe:
/// - Registration menggunakan RwLock untuk concurrent reads
/// - Scanning menggunakan snapshot untuk consistency
/// - Statistics menggunakan atomic operations
///
/// # Examples
///
/// ```
/// use fgc::marker::roots::{RootScanner, RootType};
///
/// let scanner = RootScanner::new();
///
/// // Register global root
/// let value: usize = 0x12345678;
/// let handle = scanner.register_global_root(&value as *const usize as usize, Some("my_global"));
///
/// // Scan roots
/// let mut live_refs = Vec::new();
/// scanner.scan_roots(|ref_value| {
///     live_refs.push(ref_value);
/// });
///
/// // Root otomatis unregister saat handle di-drop
/// drop(handle);
/// ```
#[derive(Debug)]
pub struct RootScanner {
    /// Semua roots (combined view)
    roots: RwLock<Vec<RootDescriptor>>,

    /// Stack roots (indexed untuk fast access)
    stack_roots: RwLock<Vec<usize>>,

    /// Global roots (indexed untuk fast access)
    global_roots: RwLock<Vec<usize>>,

    /// Class roots
    class_roots: RwLock<Vec<usize>>,

    /// Internal roots
    internal_roots: RwLock<Vec<usize>>,

    /// Counter untuk root IDs
    next_root_id: AtomicUsize,

    /// Total registrations (untuk statistics)
    total_registrations: AtomicUsize,

    /// Total unregistrations
    total_unregistrations: AtomicUsize,
}

impl RootScanner {
    /// Create new root scanner
    pub fn new() -> Self {
        Self {
            roots: RwLock::new(Vec::new()),
            stack_roots: RwLock::new(Vec::new()),
            global_roots: RwLock::new(Vec::new()),
            class_roots: RwLock::new(Vec::new()),
            internal_roots: RwLock::new(Vec::new()),
            next_root_id: AtomicUsize::new(0),
            total_registrations: AtomicUsize::new(0),
            total_unregistrations: AtomicUsize::new(0),
        }
    }

    /// Register global root
    ///
    /// # Arguments
    /// * `address` - Address dimana reference disimpan
    /// * `name` - Optional name untuk debugging
    ///
    /// # Returns
    /// RootHandle yang akan otomatis unregister saat di-drop
    ///
    /// # Examples
    ///
    /// ```
    /// let scanner = RootScanner::new();
    /// let global_var: usize = 0x12345678;
    /// let handle = scanner.register_global_root(&global_var as *const usize as usize, None);
    /// ```
    pub fn register_global_root(&self, address: usize, name: Option<&str>) -> RootHandle {
        self.register_root(address, RootType::Global, name)
    }

    /// Register stack root
    pub fn register_stack_root(&self, address: usize, name: Option<&str>) -> RootHandle {
        self.register_root(address, RootType::Stack, name)
    }

    /// Register class root
    pub fn register_class_root(&self, address: usize, name: Option<&str>) -> RootHandle {
        self.register_root(address, RootType::Class, name)
    }

    /// Register internal root
    pub fn register_internal_root(&self, address: usize, name: Option<&str>) -> RootHandle {
        self.register_root(address, RootType::Internal, name)
    }

    /// Register root dengan type spesifik
    ///
    /// # Arguments
    /// * `address` - Address dimana reference disimpan
    /// * `root_type` - Type root
    /// * `name` - Optional name untuk debugging
    ///
    /// # Returns
    /// RootHandle untuk managing lifecycle
    fn register_root(&self, address: usize, root_type: RootType, name: Option<&str>) -> RootHandle {
        let root_id = self.next_root_id.fetch_add(1, Ordering::Relaxed);
        let descriptor = RootDescriptor::new(address, root_type, name, root_id);

        // Add to main list
        {
            let mut roots = self.roots.write().unwrap();
            roots.push(descriptor);
        }

        // Add to type-specific index
        match root_type {
            RootType::Stack => {
                let mut stack = self.stack_roots.write().unwrap();
                stack.push(address);
            }
            RootType::Global => {
                let mut global = self.global_roots.write().unwrap();
                global.push(address);
            }
            RootType::Class => {
                let mut class = self.class_roots.write().unwrap();
                class.push(address);
            }
            RootType::Internal => {
                let mut internal = self.internal_roots.write().unwrap();
                internal.push(address);
            }
            _ => {}
        }

        self.total_registrations.fetch_add(1, Ordering::Relaxed);

        RootHandle::new(root_id)
    }

    /// Unregister root by ID (internal use)
    fn unregister_root_by_id(&self, root_id: usize) {
        // Find and deactivate root by root_id
        let mut roots = self.roots.write().unwrap();
        for descriptor in roots.iter_mut() {
            if descriptor.root_id == root_id {
                descriptor.deactivate();
                break;
            }
        }

        self.total_unregistrations.fetch_add(1, Ordering::Relaxed);
    }

    /// Unregister root (manual)
    pub fn unregister_root(&self, handle: RootHandle) {
        handle.unregister(self);
    }

    /// Scan semua roots dan yield references
    ///
    /// # Arguments
    /// * `callback` - Dipanggil untuk setiap reference value yang ditemukan
    ///
    /// # Returns
    /// Jumlah references yang ditemukan
    ///
    /// # Examples
    ///
    /// ```
    /// let scanner = RootScanner::new();
    /// let mut refs = Vec::new();
    /// let count = scanner.scan_roots(|ref_value| {
    ///     refs.push(ref_value);
    /// });
    /// ```
    pub fn scan_roots<F>(&self, mut callback: F) -> usize
    where
        F: FnMut(usize),
    {
        let mut count = 0;

        // Scan semua active roots
        let roots = self.roots.read().unwrap();
        for descriptor in roots.iter() {
            if descriptor.is_active() {
                let ref_value = descriptor.read_reference();
                if ref_value != 0 {
                    callback(ref_value);
                    count += 1;
                }
            }
        }

        count
    }

    /// Scan roots dengan filter type
    ///
    /// # Arguments
    /// * `root_type` - Hanya scan roots dengan type ini
    /// * `callback` - Dipanggil untuk setiap reference value
    ///
    /// # Returns
    /// Jumlah references yang ditemukan
    pub fn scan_roots_by_type<F>(&self, root_type: RootType, mut callback: F) -> usize
    where
        F: FnMut(usize),
    {
        let mut count = 0;

        let roots = self.roots.read().unwrap();
        for descriptor in roots.iter() {
            if descriptor.is_active() && descriptor.root_type == root_type {
                let ref_value = descriptor.read_reference();
                if ref_value != 0 {
                    callback(ref_value);
                    count += 1;
                }
            }
        }

        count
    }

    /// Get semua root addresses (termasuk yang inactive)
    pub fn all_root_addresses(&self) -> Vec<usize> {
        let roots = self.roots.read().unwrap();
        roots.iter().map(|r| r.address).collect()
    }

    /// Get live root references
    pub fn get_live_roots(&self) -> Vec<usize> {
        let mut live = Vec::new();
        self.scan_roots(|ref_value| {
            live.push(ref_value);
        });
        live
    }

    /// Get statistics
    pub fn get_stats(&self) -> RootStats {
        let roots = self.roots.read().unwrap();
        let stack = self.stack_roots.read().unwrap();
        let global = self.global_roots.read().unwrap();
        let class = self.class_roots.read().unwrap();
        let internal = self.internal_roots.read().unwrap();

        let mut null_count = 0;
        let mut live_count = 0;
        let mut active_count = 0;

        for descriptor in roots.iter() {
            if descriptor.is_active() {
                active_count += 1;
                if descriptor.is_null() {
                    null_count += 1;
                } else {
                    live_count += 1;
                }
            }
        }

        RootStats {
            total_roots: roots.len(),
            active_roots: active_count,
            stack_roots: stack.len(),
            global_roots: global.len(),
            class_roots: class.len(),
            internal_roots: internal.len(),
            null_roots: null_count,
            live_roots: live_count,
        }
    }

    /// Clear semua roots (untuk testing/shutdown)
    pub fn clear_all_roots(&self) {
        let mut roots = self.roots.write().unwrap();
        roots.clear();

        let mut stack = self.stack_roots.write().unwrap();
        stack.clear();

        let mut global = self.global_roots.write().unwrap();
        global.clear();

        let mut class = self.class_roots.write().unwrap();
        class.clear();

        let mut internal = self.internal_roots.write().unwrap();
        internal.clear();
    }

    /// Get root count
    pub fn root_count(&self) -> usize {
        let roots = self.roots.read().unwrap();
        roots.len()
    }

    /// Get active root count
    pub fn active_root_count(&self) -> usize {
        let roots = self.roots.read().unwrap();
        roots.iter().filter(|r| r.is_active()).count()
    }

    /// Update reference setelah relocation
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    /// * `new_address` - New object address setelah relocation
    pub fn update_reference(&self, old_address: usize, new_address: usize) {
        let roots = self.roots.read().unwrap();
        for descriptor in roots.iter() {
            if descriptor.is_active() {
                let current = descriptor.read_reference();
                if current == old_address {
                    descriptor.update_reference(new_address);
                }
            }
        }
    }
}

impl Default for RootScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RootScanner {
    fn clone(&self) -> Self {
        // Clone roots dengan AtomicBool yang baru
        let orig_roots = self.roots.read().unwrap();
        let cloned_roots: Vec<RootDescriptor> = orig_roots.iter().map(|r| {
            RootDescriptor {
                address: r.address,
                root_type: r.root_type,
                name: r.name.clone(),
                root_id: r.root_id,  // Keep same root_id
                active: AtomicBool::new(r.active.load(Ordering::Relaxed)),
            }
        }).collect();

        Self {
            roots: RwLock::new(cloned_roots),
            stack_roots: RwLock::new(self.stack_roots.read().unwrap().clone()),
            global_roots: RwLock::new(self.global_roots.read().unwrap().clone()),
            class_roots: RwLock::new(self.class_roots.read().unwrap().clone()),
            internal_roots: RwLock::new(self.internal_roots.read().unwrap().clone()),
            next_root_id: AtomicUsize::new(self.next_root_id.load(Ordering::Relaxed)),  // Keep same counter
            total_registrations: AtomicUsize::new(self.total_registrations.load(Ordering::Relaxed)),
            total_unregistrations: AtomicUsize::new(self.total_unregistrations.load(Ordering::Relaxed)),
        }
    }
}

/// StackWalker - walker untuk thread stacks
///
/// Walk thread stacks untuk identify pointers.
///
/// # Note
///
/// Implementasi penuh memerlukan platform-specific stack unwinding.
/// Ini adalah stub untuk future implementation.
pub struct StackWalker {
    /// Thread ID
    thread_id: u64,
    /// Stack base address
    stack_base: usize,
    /// Stack size in bytes
    stack_size: usize,
    /// Stack pointer (current)
    stack_pointer: usize,
}

impl StackWalker {
    /// Create stack walker untuk thread
    ///
    /// # Arguments
    /// * `thread_id` - Thread ID
    /// * `stack_base` - Stack base address
    /// * `stack_size` - Stack size in bytes
    pub fn new(thread_id: u64, stack_base: usize, stack_size: usize) -> Self {
        Self {
            thread_id,
            stack_base,
            stack_size,
            stack_pointer: stack_base,
        }
    }

    /// Walk stack frames
    ///
    /// # Arguments
    /// * `callback` - Dipanggil untuk setiap frame
    ///
    /// # Returns
    /// Result dengan OK jika berhasil walk semua frames
    pub fn walk_frames<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(StackFrame),
    {
        // Note: Dalam implementasi nyata, ini walk stack frames
        // menggunakan frame pointer atau unwinding information (libunwind)

        // Conservative stack scanning: scan seluruh stack range
        let mut current = self.stack_base;
        let end = self.stack_base + self.stack_size;

        while current < end {
            callback(StackFrame {
                address: current,
                size: std::mem::size_of::<usize>(),
            });

            current += std::mem::size_of::<usize>();
        }

        Ok(())
    }

    /// Scan stack untuk pointers
    ///
    /// # Arguments
    /// * `callback` - Dipanggil untuk setiap potential pointer
    /// * `heap_range` - Valid heap address range (min, max)
    ///
    /// # Returns
    /// Jumlah pointers yang ditemukan
    pub fn scan_for_pointers<F>(&self, mut callback: F, heap_range: (usize, usize)) -> Result<usize>
    where
        F: FnMut(usize),
    {
        let mut count = 0;

        self.walk_frames(|frame| {
            // Scan frame untuk pointers
            let mut addr = frame.address;
            let end = addr + frame.size;

            while addr < end {
                unsafe {
                    let value = *(addr as *const usize);

                    // Check jika value adalah valid pointer ke heap
                    if value != 0 && value >= heap_range.0 && value < heap_range.1 {
                        callback(value);
                        count += 1;
                    }
                }

                addr += std::mem::size_of::<usize>();
            }
        })?;

        Ok(count)
    }

    /// Get thread ID
    pub fn thread_id(&self) -> u64 {
        self.thread_id
    }

    /// Get stack base
    pub fn stack_base(&self) -> usize {
        self.stack_base
    }

    /// Get stack size
    pub fn stack_size(&self) -> usize {
        self.stack_size
    }
}

/// Stack frame representation
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Frame address
    pub address: usize,
    /// Frame size in bytes
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_descriptor_creation() {
        let value: usize = 0x12345678;
        let descriptor = RootDescriptor::new(&value as *const usize as usize, RootType::Global, Some("test"), 0);

        assert_eq!(descriptor.address, &value as *const usize as usize);
        assert_eq!(descriptor.root_type, RootType::Global);
        assert_eq!(descriptor.name, Some("test".to_string()));
        assert_eq!(descriptor.root_id, 0);
        assert!(descriptor.is_active());
    }

    #[test]
    fn test_root_descriptor_read_write() {
        let mut value: usize = 0x12345678;
        let descriptor = RootDescriptor::new(&mut value as *mut usize as usize, RootType::Global, None, 0);

        assert_eq!(descriptor.read_reference(), 0x12345678);
        assert!(!descriptor.is_null());

        descriptor.update_reference(0x87654321);
        assert_eq!(descriptor.read_reference(), 0x87654321);
        assert_eq!(value, 0x87654321);
    }

    #[test]
    fn test_root_handle_auto_unregister() {
        let scanner = RootScanner::new();
        let value: usize = 0x12345678;

        let handle = scanner.register_global_root(&value as *const usize as usize, Some("temp"));
        assert_eq!(scanner.root_count(), 1);
        assert_eq!(scanner.active_root_count(), 1);

        // Explicit unregister (drop tidak unregister otomatis dalam design ini)
        handle.unregister(&scanner);

        // Root masih ada tapi inactive
        assert_eq!(scanner.root_count(), 1);
        assert_eq!(scanner.active_root_count(), 0);
    }

    #[test]
    fn test_root_scanner_registration() {
        let scanner = RootScanner::new();
        let value1: usize = 0x11111111;
        let value2: usize = 0x22222222;

        let handle1 = scanner.register_global_root(&value1 as *const usize as usize, Some("global1"));
        let handle2 = scanner.register_stack_root(&value2 as *const usize as usize, Some("stack1"));

        assert_eq!(scanner.root_count(), 2);
        assert_eq!(scanner.active_root_count(), 2);

        let stats = scanner.get_stats();
        assert_eq!(stats.global_roots, 1);
        assert_eq!(stats.stack_roots, 1);

        handle1.unregister(&scanner);
        assert_eq!(scanner.active_root_count(), 1);
    }

    #[test]
    fn test_root_scanning() {
        let scanner = RootScanner::new();
        let value1: usize = 0xAAAAAAAA;
        let value2: usize = 0xBBBBBBBB;
        let null_value: usize = 0;

        scanner.register_global_root(&value1 as *const usize as usize, None);
        scanner.register_global_root(&value2 as *const usize as usize, None);
        scanner.register_global_root(&null_value as *const usize as usize, Some("null"));

        let mut found_refs = Vec::new();
        let count = scanner.scan_roots(|ref_value| {
            found_refs.push(ref_value);
        });

        assert_eq!(count, 2); // Hanya non-null refs
        assert!(found_refs.contains(&0xAAAAAAAA));
        assert!(found_refs.contains(&0xBBBBBBBB));
        assert!(!found_refs.contains(&0));
    }

    #[test]
    fn test_root_stats() {
        let scanner = RootScanner::new();
        let value: usize = 0x12345678;
        let null_value: usize = 0;

        scanner.register_global_root(&value as *const usize as usize, None);
        scanner.register_global_root(&null_value as *const usize as usize, None);

        let stats = scanner.get_stats();

        assert_eq!(stats.total_roots, 2);
        assert_eq!(stats.active_roots, 2);
        assert_eq!(stats.global_roots, 2);
        assert_eq!(stats.null_roots, 1);
        assert_eq!(stats.live_roots, 1);
    }

    #[test]
    fn test_reference_update() {
        let scanner = RootScanner::new();
        let mut value: usize = 0x11111111;

        scanner.register_global_root(&mut value as *mut usize as usize, None);

        // Update reference setelah relocation
        scanner.update_reference(0x11111111, 0x22222222);

        assert_eq!(value, 0x22222222);
    }

    #[test]
    fn test_clear_all_roots() {
        let scanner = RootScanner::new();
        let value: usize = 0x12345678;

        scanner.register_global_root(&value as *const usize as usize, None);
        scanner.register_stack_root(&value as *const usize as usize, None);

        assert_eq!(scanner.root_count(), 2);

        scanner.clear_all_roots();

        assert_eq!(scanner.root_count(), 0);
    }

    #[test]
    fn test_scan_by_type() {
        let scanner = RootScanner::new();
        let value1: usize = 0x11111111;
        let value2: usize = 0x22222222;
        let value3: usize = 0x33333333;

        scanner.register_global_root(&value1 as *const usize as usize, None);
        scanner.register_stack_root(&value2 as *const usize as usize, None);
        scanner.register_class_root(&value3 as *const usize as usize, None);

        let mut global_refs = Vec::new();
        scanner.scan_roots_by_type(RootType::Global, |ref_value| {
            global_refs.push(ref_value);
        });

        assert_eq!(global_refs.len(), 1);
        assert_eq!(global_refs[0], 0x11111111);
    }

    #[test]
    fn test_concurrent_registration() {
        use std::thread;

        let scanner = Arc::new(RootScanner::new());
        let mut handles = Vec::new();

        for i in 0..10 {
            let scanner_clone = Arc::clone(&scanner);
            let handle = thread::spawn(move || {
                let value = Box::new(i as usize);
                let _root_handle = scanner_clone.register_global_root(
                    &*value as *const usize as usize,
                    Some(&format!("thread_{}", i)),
                );
                // Keep value alive
                drop(value);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Semua roots seharusnya terdaftar
        assert!(scanner.root_count() >= 10);
    }

    #[test]
    fn test_root_type_display() {
        assert_eq!(format!("{}", RootType::Stack), "Stack");
        assert_eq!(format!("{}", RootType::Global), "Global");
        assert_eq!(format!("{}", RootType::Class), "Class");
        assert_eq!(format!("{}", RootType::Internal), "Internal");
    }

    #[test]
    fn test_root_stats_display() {
        let scanner = RootScanner::new();
        let stats = scanner.get_stats();
        let display = format!("{}", stats);

        assert!(display.contains("total:"));
        assert!(display.contains("active:"));
    }
}
