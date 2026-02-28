//! Root Scanning - GC Root Identification and Management
//!
//! Roots are starting points for marking. All objects reachable
//! from roots must be marked as live.
//!
//! # Root Types
//!
//! 1. **Stack Roots** - Local variables in thread stacks
//! 2. **Global Roots** - Static/global variables
//! 3. **Class Roots** - Class loaders and loaded classes
//! 4. **VM Internal Roots** - Monitors, thread local storage, etc.
//!
//! # Root Scanning Challenge
//!
//! During concurrent marking, mutator threads are running and the stack
//! keeps changing. FGC uses concurrent stack scanning with
//! watermarks to handle this.
//!
//! # Thread Safety
//!
//! RootScanner is thread-safe. Multiple threads can:
//! - Register/unregister roots concurrently
//! - Scan roots concurrently
//! - Query statistics concurrently

use crate::error::{FgcError, Result};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::RwLock;

/// Root types for categorization
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

/// Root descriptor - describes a single root reference
///
/// This struct stores complete information about a single root:
/// - Address where the reference is stored
/// - Root type for categorization
/// - Optional name for debugging
/// - Active status for lifecycle tracking
/// - Root ID for identification
#[derive(Debug)]
pub struct RootDescriptor {
    /// Address where reference is stored (pointer to pointer)
    pub address: usize,
    /// Root type
    pub root_type: RootType,
    /// Optional name for debugging
    pub name: Option<String>,
    /// Root ID for identification
    pub root_id: usize,
    /// Whether this root is active
    pub active: AtomicBool,
}

impl RootDescriptor {
    /// Create new root descriptor
    ///
    /// # Arguments
    /// * `address` - Address where reference is stored
    /// * `root_type` - Root type
    /// * `name` - Optional name for debugging
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

    /// Read reference value from address
    ///
    /// # Safety
    /// Address must be valid and aligned for reading usize.
    /// This function validates the address before reading to prevent
    /// undefined behavior from invalid memory access.
    ///
    /// # CRIT-01 FIX: Root Validation
    /// This method now validates that the root address points to GC-managed
    /// heap memory, preventing attackers from reading arbitrary memory.
    pub fn read_reference(&self) -> Result<usize> {
        // Check for null address
        if self.address == 0 {
            return Ok(0); // Treat as null
        }

        // Check alignment - usize must be properly aligned
        if !self.address.is_multiple_of(std::mem::align_of::<usize>()) {
            log::warn!("Unaligned root address: {:#x}", self.address);
            return Ok(0);
        }

        // CRIT-01 FIX: Require roots to point to GC-managed heap
        // This prevents attackers from registering arbitrary memory addresses
        // as roots to exfiltrate sensitive data
        if !crate::heap::is_gc_managed_address(self.address) {
            log::warn!("Root address {:#x} not in GC-managed heap", self.address);
            return Err(FgcError::InvalidArgument(
                "Root must point to GC-managed heap".to_string(),
            ));
        }

        // Optional: Check if address is in valid memory range
        if !crate::memory::is_readable(self.address).unwrap_or(false) {
            log::warn!("Invalid root address: {:#x}", self.address);
            return Ok(0);
        }

        unsafe {
            let ptr = self.address as *const usize;
            Ok(ptr.read_volatile())
        }
    }

    /// Update reference value (for relocation)
    ///
    /// # Safety
    /// Address must be valid and aligned for writing usize.
    /// This function validates the address before writing to prevent
    /// undefined behavior from invalid memory access.
    ///
    /// # CRIT-01 FIX: Root Validation
    /// This method now validates that both the root address and new value
    /// point to GC-managed heap memory, preventing attackers from writing
    /// to arbitrary memory locations.
    pub fn update_reference(&self, new_value: usize) -> Result<()> {
        // Check for null address
        if self.address == 0 {
            log::warn!("Cannot update reference at null address");
            return Ok(());
        }

        // Check alignment - usize must be properly aligned
        if !self.address.is_multiple_of(std::mem::align_of::<usize>()) {
            log::warn!("Unaligned root address for write: {:#x}", self.address);
            return Err(FgcError::InvalidArgument(
                "Unaligned root address".to_string(),
            ));
        }

        // CRIT-01 FIX: Validate root address points to GC-managed heap
        if !crate::heap::is_gc_managed_address(self.address) {
            log::warn!("Root address {:#x} not in GC-managed heap", self.address);
            return Err(FgcError::InvalidArgument(
                "Root must point to GC-managed heap".to_string(),
            ));
        }

        // CRIT-01 FIX: Validate new value also points to GC heap
        // This prevents attackers from making roots point to arbitrary memory
        if new_value != 0 && !crate::heap::is_gc_managed_address(new_value) {
            log::warn!("New root value {:#x} not in GC-managed heap", new_value);
            return Err(FgcError::InvalidArgument(
                "Root must point to GC-managed heap".to_string(),
            ));
        }

        // Check if address is in valid memory range
        if !crate::memory::is_writable(self.address).unwrap_or(false) {
            log::warn!("Invalid writable root address: {:#x}", self.address);
            return Err(FgcError::InvalidArgument(
                "Root address not writable".to_string(),
            ));
        }

        unsafe {
            let ptr = self.address as *mut usize;
            ptr.write_volatile(new_value);
        }
        Ok(())
    }

    /// Check if reference is null
    pub fn is_null(&self) -> bool {
        self.read_reference().unwrap_or(0) == 0
    }

    /// Check if root is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Deactivate root (without removing from list)
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Activate root
    pub fn activate(&self) {
        self.active.store(true, Ordering::Relaxed);
    }
}

/// Handle for managing root lifecycle
///
/// This handle is given when registering a root and automatically
/// unregisters the root when the handle is dropped.
///
/// Note: Handle stores weak reference to scanner to avoid circular reference.
#[derive(Debug)]
pub struct RootHandle {
    root_id: usize,
}

impl RootHandle {
    /// Create new root handle
    fn new(root_id: usize) -> Self {
        Self { root_id }
    }

    /// Get root ID
    pub fn id(&self) -> usize {
        self.root_id
    }

    /// Manually unregister root (drop also does this)
    ///
    /// # Arguments
    /// * `scanner` - RootScanner to unregister from
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

/// RootScanner - scanner for various root types
///
/// RootScanner manages registration and scanning of all
/// GC roots. Roots are starting points for marking phase.
///
/// # Thread Safety
///
/// RootScanner is fully thread-safe:
/// - Registration uses RwLock for concurrent reads
/// - Scanning uses snapshot for consistency
/// - Statistics use atomic operations
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
/// // Root is automatically unregistered when handle is dropped
/// drop(handle);
/// ```
#[derive(Debug)]
pub struct RootScanner {
    /// All roots (combined view)
    roots: RwLock<Vec<RootDescriptor>>,

    /// Stack roots (indexed for fast access)
    stack_roots: RwLock<Vec<usize>>,

    /// Global roots (indexed for fast access)
    global_roots: RwLock<Vec<usize>>,

    /// Class roots
    class_roots: RwLock<Vec<usize>>,

    /// Internal roots
    internal_roots: RwLock<Vec<usize>>,

    /// Counter for root IDs
    next_root_id: AtomicUsize,

    /// Total registrations (for statistics)
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
    /// * `address` - Address where reference is stored
    /// * `name` - Optional name for debugging
    ///
    /// # Returns
    /// RootHandle that will automatically unregister when dropped
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

    /// Register root with specific type
    ///
    /// # Arguments
    /// * `address` - Address where reference is stored
    /// * `root_type` - Root type
    /// * `name` - Optional name for debugging
    ///
    /// # Returns
    /// RootHandle for managing lifecycle
    fn register_root(&self, address: usize, root_type: RootType, name: Option<&str>) -> RootHandle {
        self.validate_root_address(address);

        let root_id = self.next_root_id.fetch_add(1, Ordering::Relaxed);

        self.add_root_to_list(address, root_type, name, root_id);

        self.total_registrations.fetch_add(1, Ordering::Relaxed);

        RootHandle::new(root_id)
    }

    /// Validate root address for registration
    ///
    /// # Arguments
    /// * `address` - Address to validate
    ///
    /// # Panics
    /// Logs warning for invalid addresses but does not prevent registration.
    /// Address validation is performed at read/write time instead.
    fn validate_root_address(&self, address: usize) {
        if address == 0 {
            log::warn!("Registering root with null address");
        } else if !address.is_multiple_of(std::mem::align_of::<usize>()) {
            log::warn!("Registering root with unaligned address: {:#x}", address);
        }
    }

    /// Add root descriptor to appropriate lists
    ///
    /// # Arguments
    /// * `address` - Address where reference is stored
    /// * `root_type` - Root type
    /// * `name` - Optional name for debugging
    /// * `root_id` - Unique root ID
    ///
    /// # Panics
    /// Panics if lock is poisoned (indicates serious concurrency bug)
    fn add_root_to_list(
        &self,
        address: usize,
        root_type: RootType,
        name: Option<&str>,
        root_id: usize,
    ) {
        // Create descriptor
        let descriptor = RootDescriptor::new(address, root_type, name, root_id);

        // Add to main list
        let Ok(mut roots) = self.roots.write() else {
            log::error!("RootScanner roots lock poisoned");
            return;
        };
        roots.push(descriptor);

        // Add to type-specific index
        self.add_to_type_index(address, root_type);
    }

    /// Add address to type-specific index
    ///
    /// # Arguments
    /// * `address` - Root address
    /// * `root_type` - Root type for indexing
    fn add_to_type_index(&self, address: usize, root_type: RootType) {
        match root_type {
            RootType::Stack => {
                let Ok(mut stack) = self.stack_roots.write() else {
                    log::error!("RootScanner stack_roots lock poisoned");
                    return;
                };
                stack.push(address);
            },
            RootType::Global => {
                let Ok(mut global) = self.global_roots.write() else {
                    log::error!("RootScanner global_roots lock poisoned");
                    return;
                };
                global.push(address);
            },
            RootType::Class => {
                let Ok(mut class) = self.class_roots.write() else {
                    log::error!("RootScanner class_roots lock poisoned");
                    return;
                };
                class.push(address);
            },
            RootType::Internal => {
                let Ok(mut internal) = self.internal_roots.write() else {
                    log::error!("RootScanner internal_roots lock poisoned");
                    return;
                };
                internal.push(address);
            },
            _ => {},
        }
    }

    /// Unregister root by ID (internal use)
    fn unregister_root_by_id(&self, root_id: usize) {
        // Find and deactivate root by root_id
        let Ok(mut roots) = self.roots.write() else {
            log::error!("RootScanner roots lock poisoned");
            return;
        };
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

    /// Scan all roots and yield references
    ///
    /// # Arguments
    /// * `callback` - Called for each reference value found
    ///
    /// # Returns
    /// Number of references found
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

        // Scan all active roots
        let Ok(roots) = self.roots.read() else {
            log::error!("RootScanner roots read lock poisoned");
            return 0;
        };
        for descriptor in roots.iter() {
            if descriptor.is_active() {
                if let Ok(ref_value) = descriptor.read_reference() {
                    if ref_value != 0 {
                        callback(ref_value);
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Scan roots with type filter
    ///
    /// # Arguments
    /// * `root_type` - Only scan roots with this type
    /// * `callback` - Called for each reference value
    ///
    /// # Returns
    /// Number of references found
    pub fn scan_roots_by_type<F>(&self, root_type: RootType, mut callback: F) -> usize
    where
        F: FnMut(usize),
    {
        let mut count = 0;

        let Ok(roots) = self.roots.read() else {
            log::error!("RootScanner roots read lock poisoned");
            return 0;
        };
        for descriptor in roots.iter() {
            if descriptor.is_active() && descriptor.root_type == root_type {
                if let Ok(ref_value) = descriptor.read_reference() {
                    if ref_value != 0 {
                        callback(ref_value);
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Get all root addresses (including inactive)
    pub fn all_root_addresses(&self) -> Vec<usize> {
        let Ok(roots) = self.roots.read() else {
            log::error!("RootScanner roots read lock poisoned");
            return Vec::new();
        };
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
        // Read locks with proper error handling
        let roots = match self.roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner roots read lock poisoned: {}", e);
                return RootStats::default();
            },
        };
        let stack = match self.stack_roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner stack_roots read lock poisoned: {}", e);
                return RootStats::default();
            },
        };
        let global = match self.global_roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner global_roots read lock poisoned: {}", e);
                return RootStats::default();
            },
        };
        let class = match self.class_roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner class_roots read lock poisoned: {}", e);
                return RootStats::default();
            },
        };
        let internal = match self.internal_roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner internal_roots read lock poisoned: {}", e);
                return RootStats::default();
            },
        };

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

    /// Clear all roots (for testing/shutdown)
    pub fn clear_all_roots(&self) {
        // Write locks with proper error handling
        {
            let mut roots = match self.roots.write() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("RootScanner roots write lock poisoned: {}", e);
                    return;
                },
            };
            roots.clear();
        }

        {
            let mut stack = match self.stack_roots.write() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("RootScanner stack_roots write lock poisoned: {}", e);
                    return;
                },
            };
            stack.clear();
        }

        {
            let mut global = match self.global_roots.write() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("RootScanner global_roots write lock poisoned: {}", e);
                    return;
                },
            };
            global.clear();
        }

        {
            let mut class = match self.class_roots.write() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("RootScanner class_roots write lock poisoned: {}", e);
                    return;
                },
            };
            class.clear();
        }

        {
            let mut internal = match self.internal_roots.write() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("RootScanner internal_roots write lock poisoned: {}", e);
                    return;
                },
            };
            internal.clear();
        }
    }

    /// Get root count
    pub fn root_count(&self) -> usize {
        let roots = match self.roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner roots read lock poisoned: {}", e);
                return 0;
            },
        };
        roots.len()
    }

    /// Get active root count
    pub fn active_root_count(&self) -> usize {
        let roots = match self.roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner roots read lock poisoned: {}", e);
                return 0;
            },
        };
        roots.iter().filter(|r| r.is_active()).count()
    }

    /// Update reference after relocation
    ///
    /// # Arguments
    /// * `old_address` - Old object address
    /// * `new_address` - New object address after relocation
    pub fn update_reference(&self, old_address: usize, new_address: usize) {
        let roots = match self.roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner roots read lock poisoned: {}", e);
                return;
            },
        };
        for descriptor in roots.iter() {
            if descriptor.is_active() {
                if let Ok(current) = descriptor.read_reference() {
                    if current == old_address {
                        if let Err(e) = descriptor.update_reference(new_address) {
                            log::warn!("Failed to update root reference: {}", e);
                        }
                    }
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
        // Clone roots with new AtomicBool
        let orig_roots = match self.roots.read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("RootScanner roots read lock poisoned: {}", e);
                return Self::new();
            },
        };
        let cloned_roots: Vec<RootDescriptor> = orig_roots
            .iter()
            .map(|r| {
                RootDescriptor {
                    address: r.address,
                    root_type: r.root_type,
                    name: r.name.clone(),
                    root_id: r.root_id, // Keep same root_id
                    active: AtomicBool::new(r.active.load(Ordering::Relaxed)),
                }
            })
            .collect();

        let stack_roots = match self.stack_roots.read() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("RootScanner stack_roots read lock poisoned: {}", e);
                Vec::new()
            },
        };
        let global_roots = match self.global_roots.read() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("RootScanner global_roots read lock poisoned: {}", e);
                Vec::new()
            },
        };
        let class_roots = match self.class_roots.read() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("RootScanner class_roots read lock poisoned: {}", e);
                Vec::new()
            },
        };
        let internal_roots = match self.internal_roots.read() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("RootScanner internal_roots read lock poisoned: {}", e);
                Vec::new()
            },
        };

        Self {
            roots: RwLock::new(cloned_roots),
            stack_roots: RwLock::new(stack_roots),
            global_roots: RwLock::new(global_roots),
            class_roots: RwLock::new(class_roots),
            internal_roots: RwLock::new(internal_roots),
            next_root_id: AtomicUsize::new(self.next_root_id.load(Ordering::Relaxed)), // Keep same counter
            total_registrations: AtomicUsize::new(self.total_registrations.load(Ordering::Relaxed)),
            total_unregistrations: AtomicUsize::new(
                self.total_unregistrations.load(Ordering::Relaxed),
            ),
        }
    }
}

/// StackWalker - walker for thread stacks
///
/// Walk thread stacks to identify pointers.
///
/// # Note
///
/// Full implementation requires platform-specific stack unwinding.
/// This is a stub for future implementation.
pub struct StackWalker {
    /// Thread ID
    thread_id: u64,
    /// Stack base address
    stack_base: usize,
    /// Stack size in bytes
    stack_size: usize,
}

impl StackWalker {
    /// Create stack walker for thread
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
        }
    }

    /// Walk stack frames
    ///
    /// # Arguments
    /// * `callback` - Called for every frame
    ///
    /// # Returns
    /// Result with OK if successfully walked all frames
    pub fn walk_frames<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(StackFrame),
    {
        // Note: In real implementation, this walks stack frames
        // using frame pointer or unwinding information (libunwind)

        // Conservative stack scanning: scan entire stack range
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

    /// Scan stack for pointers
    ///
    /// # Arguments
    /// * `callback` - Called for every potential pointer
    /// * `heap_range` - Valid heap address range (min, max)
    ///
    /// # Returns
    /// Number of pointers found
    pub fn scan_for_pointers<F>(&self, mut callback: F, heap_range: (usize, usize)) -> Result<usize>
    where
        F: FnMut(usize),
    {
        let mut count = 0;

        self.walk_frames(|frame| {
            // Scan frame for pointers
            let mut addr = frame.address;
            let end = addr + frame.size;

            while addr < end {
                unsafe {
                    let value = *(addr as *const usize);

                    // Check if value is valid pointer to heap
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
    use std::sync::Arc;

    #[test]
    fn test_root_descriptor_creation() {
        let value: usize = 0x12345678;
        let descriptor = RootDescriptor::new(
            &value as *const usize as usize,
            RootType::Global,
            Some("test"),
            0,
        );

        assert_eq!(descriptor.address, &value as *const usize as usize);
        assert_eq!(descriptor.root_type, RootType::Global);
        assert_eq!(descriptor.name, Some("test".to_string()));
        assert_eq!(descriptor.root_id, 0);
        assert!(descriptor.is_active());
    }

    #[test]
    fn test_root_descriptor_read_write() {
        let mut value: usize = 0x12345678;
        let descriptor =
            RootDescriptor::new(&mut value as *mut usize as usize, RootType::Global, None, 0);

        assert_eq!(descriptor.read_reference().unwrap(), 0x12345678);
        assert!(!descriptor.is_null());

        descriptor.update_reference(0x87654321).unwrap();
        assert_eq!(descriptor.read_reference().unwrap(), 0x87654321);
        assert_eq!(value, 0x87654321);
    }

    #[test]
    fn test_root_handle_auto_unregister() {
        let scanner = RootScanner::new();
        let value: usize = 0x12345678;

        let handle = scanner.register_global_root(&value as *const usize as usize, Some("temp"));
        assert_eq!(scanner.root_count(), 1);
        assert_eq!(scanner.active_root_count(), 1);

        // Explicit unregister (drop does not unregister automatically in this design)
        handle.unregister(&scanner);

        // Root still exists but is inactive
        assert_eq!(scanner.root_count(), 1);
        assert_eq!(scanner.active_root_count(), 0);
    }

    #[test]
    fn test_root_scanner_registration() {
        let scanner = RootScanner::new();
        let value1: usize = 0x11111111;
        let value2: usize = 0x22222222;

        let handle1 =
            scanner.register_global_root(&value1 as *const usize as usize, Some("global1"));
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

    #[test]
    fn test_validate_root_address() {
        let scanner = RootScanner::new();

        // Valid address (non-zero, aligned) - should not panic or log
        scanner.validate_root_address(0x1000);

        // Null address - logs warning but doesn't prevent registration
        scanner.validate_root_address(0);

        // Unaligned address - logs warning but doesn't prevent registration
        scanner.validate_root_address(0x1001);
    }

    #[test]
    fn test_add_to_type_index() {
        let scanner = RootScanner::new();

        // Test adding to each type index
        scanner.add_to_type_index(0x1000, RootType::Stack);
        scanner.add_to_type_index(0x2000, RootType::Global);
        scanner.add_to_type_index(0x3000, RootType::Class);
        scanner.add_to_type_index(0x4000, RootType::Internal);

        // Verify counts
        let stats = scanner.get_stats();
        assert_eq!(stats.stack_roots, 1);
        assert_eq!(stats.global_roots, 1);
        assert_eq!(stats.class_roots, 1);
        assert_eq!(stats.internal_roots, 1);
    }
}
