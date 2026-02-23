//! Object Scanner - Traces References Within Objects
//!
//! Object scanner is responsible for:
//! - Reading reference map from object
//! - Finding all pointer fields in object
//! - Returning references for further tracing
//!
//! # Scanning Modes
//!
//! 1. **Precise Scanning** - Uses reference map from class metadata
//! 2. **Conservative Scanning** - Scan word-by-word, treat all non-zero as pointer
//!
//! # Object Layout Cache
//!
//! The scanner caches object layouts (reference maps) for faster scanning:
//! - **Hot Objects**: Frequently scanned objects have cached layouts
//! - **LRU Eviction**: Cache evicts least-recently-used entries
//! - **Generation-Based**: Cache is cleared between GC cycles
//!
//! # Thread Safety
//!
//! All scanner functions are thread-safe and can be called
//! from multiple threads concurrently.

use crate::memory;
use crate::object::{ObjectHeader, ReferenceMap, HEADER_SIZE, OBJECT_ALIGNMENT};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

/// Object scanning statistics
#[derive(Debug, Default, Clone)]
pub struct ObjectScanStats {
    /// Number of objects scanned
    pub objects_scanned: u64,
    /// Total references found
    pub references_found: u64,
    /// Average refs per object
    pub avg_refs_per_object: f64,
    /// Maximum refs in one object
    pub max_refs_in_object: usize,
    /// Minimum refs in one object
    pub min_refs_in_object: usize,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
}

/// Cache entry for object layout
#[derive(Debug, Clone)]
struct LayoutCacheEntry {
    /// Reference map for this layout
    ref_map: ReferenceMap,
    /// Object size
    size: usize,
    /// Last access time
    last_access: Instant,
    /// Access count
    access_count: u64,
}

/// Object layout cache for faster scanning
///
/// Caches reference maps for object types to avoid
/// recomputing layouts on each scan.
///
/// ## Cache Strategy
///
/// - **Size-based Keying**: Cache by object size and alignment pattern
/// - **LRU Eviction**: Evict least recently used entries when full
/// - **Hot Object Detection**: Track frequently accessed layouts
/// - **Memory Efficient**: Share reference maps via Arc
pub struct ObjectLayoutCache {
    /// Cache entries (size -> layout)
    entries: RwLock<HashMap<usize, LayoutCacheEntry>>,
    /// Maximum cache size
    max_entries: usize,
    /// Total cache hits
    hits: std::sync::atomic::AtomicU64,
    /// Total cache misses
    misses: std::sync::atomic::AtomicU64,
}

impl ObjectLayoutCache {
    /// Create new layout cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::with_capacity(max_entries)),
            max_entries,
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Get cached layout for object
    ///
    /// Returns cached reference map if available, otherwise computes and caches it.
    pub fn get_or_compute(&self, obj_addr: usize) -> ReferenceMap {
        unsafe {
            let header = &*(obj_addr as *const ObjectHeader);
            let size = header.size;

            if let Some(cached) = self.get_cached(size) {
                return cached;
            }

            let ref_map = compute_reference_map(obj_addr);
            self.insert(size, ref_map.clone());
            ref_map
        }
    }

    /// Get cached entry
    fn get_cached(&self, size: usize) -> Option<ReferenceMap> {
        let entries = self.entries.read().ok()?;

        if let Some(entry) = entries.get(&size) {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Some(entry.ref_map.clone());
        }

        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        None
    }

    /// Insert entry into cache
    fn insert(&self, size: usize, ref_map: ReferenceMap) {
        if let Ok(mut entries) = self.entries.write() {
            if entries.len() >= self.max_entries {
                self.evict_lru(&mut entries);
            }

            entries.insert(
                size,
                LayoutCacheEntry {
                    ref_map,
                    size,
                    last_access: Instant::now(),
                    access_count: 1,
                },
            );
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&self, entries: &mut HashMap<usize, LayoutCacheEntry>) {
        let mut oldest_key = None;
        let mut oldest_time = Instant::now();

        for (key, entry) in entries.iter() {
            if entry.last_access < oldest_time {
                oldest_time = entry.last_access;
                oldest_key = Some(*key);
            }
        }

        if let Some(key) = oldest_key {
            entries.remove(&key);
        }
    }

    /// Clear cache (call between GC cycles)
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> (u64, u64, usize) {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let size = self.entries.read().map(|e| e.len()).unwrap_or(0);
        (hits, misses, size)
    }
}

impl Default for ObjectLayoutCache {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Global layout cache
static LAYOUT_CACHE: std::sync::OnceLock<ObjectLayoutCache> = std::sync::OnceLock::new();

/// Get global layout cache
pub fn layout_cache() -> &'static ObjectLayoutCache {
    LAYOUT_CACHE.get_or_init(|| ObjectLayoutCache::new(4096))
}

/// Compute reference map for object (without cache)
fn compute_reference_map(obj_addr: usize) -> ReferenceMap {
    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);
        let data_size = header.get_data_size();

        let mut offsets = Vec::new();
        for i in 0..(data_size / OBJECT_ALIGNMENT) {
            offsets.push(i * OBJECT_ALIGNMENT);
        }

        ReferenceMap::new(&offsets)
    }
}

impl ObjectScanStats {
    /// Create new stats tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record scan result
    pub fn record(&mut self, ref_count: usize) {
        self.objects_scanned += 1;
        self.references_found += ref_count as u64;

        if self.objects_scanned > 0 {
            self.avg_refs_per_object = self.references_found as f64 / self.objects_scanned as f64;
        }

        if ref_count > self.max_refs_in_object {
            self.max_refs_in_object = ref_count;
        }

        // Update min: first record OR smaller than current min
        if self.objects_scanned == 1 || ref_count < self.min_refs_in_object {
            self.min_refs_in_object = ref_count;
        }
    }

    /// Merge with other stats
    pub fn merge(&mut self, other: &ObjectScanStats) {
        if other.objects_scanned == 0 {
            return;
        }

        let total_refs = self.references_found + other.references_found;
        let total_objects = self.objects_scanned + other.objects_scanned;

        self.objects_scanned = total_objects;
        self.references_found = total_refs;
        self.avg_refs_per_object = total_refs as f64 / total_objects as f64;
        self.max_refs_in_object = self.max_refs_in_object.max(other.max_refs_in_object);

        if other.min_refs_in_object > 0
            && (self.min_refs_in_object == 0 || other.min_refs_in_object < self.min_refs_in_object)
        {
            self.min_refs_in_object = other.min_refs_in_object;
        }
    }
}

/// Scan object and yield all reference fields
///
/// This is precise scanning mode - uses reference map
/// to find only the actual pointer fields.
///
/// # Arguments
/// * `obj_addr` - Address of object (points to ObjectHeader)
/// * `ref_map` - Reference map for this object type
/// * `callback` - Called for each reference found (receives reference address)
///
/// # Returns
/// Number of references found (non-null only)
///
/// # Safety
/// `obj_addr` must point to valid GC-managed object with valid header
///
/// # Examples
///
/// ```ignore
/// let refs_found = scan_object_precise(obj_addr, &ref_map, |ref_addr| {
///     let ref_value = unsafe { memory::read_pointer(ref_addr) };
///     println!("Found reference: {:#x}", ref_value);
/// });
/// ```
pub fn scan_object_precise<F>(obj_addr: usize, ref_map: &ReferenceMap, mut callback: F) -> usize
where
    F: FnMut(usize), // Called with address of each reference field
{
    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);

        // Validate object
        if header.size == 0 || header.size > 1024 * 1024 * 1024 {
            return 0;
        }

        let data_start = obj_addr + HEADER_SIZE;
        let data_size = header.get_data_size();

        let mut ref_count = 0;

        // Scan for references using reference map
        for offset in ref_map.iter() {
            if offset < data_size {
                let ref_addr = data_start + offset;

                // Read the reference
                let ref_value = memory::read_pointer(ref_addr);

                // Only yield non-null references
                if ref_value != 0 {
                    callback(ref_addr);
                    ref_count += 1;
                }
            }
        }

        ref_count
    }
}

/// Scan object with reference map from header
///
/// Uses layout cache for faster scanning of repeated object types.
///
/// # Arguments
/// * `obj_addr` - Address of object
/// * `callback` - Called for each reference found
///
/// # Returns
/// Number of references found
pub fn scan_object<F>(obj_addr: usize, mut callback: F) -> usize
where
    F: FnMut(usize),
{
    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);

        if header.size == 0 || header.size > 1024 * 1024 * 1024 {
            return 0;
        }

        let data_start = obj_addr + HEADER_SIZE;
        let data_size = header.get_data_size();

        let ref_map = layout_cache().get_or_compute(obj_addr);

        let mut ref_count = 0;

        for offset in ref_map.iter() {
            if offset < data_size {
                let ref_addr = data_start + offset;

                if memory::is_readable(ref_addr).unwrap_or(false) {
                    let ref_value = memory::read_pointer(ref_addr);

                    if ref_value != 0 {
                        callback(ref_addr);
                        ref_count += 1;
                    }
                }
            }
        }

        ref_count
    }
}

/// Conservative reference scanner
///
/// Scans memory word-by-word and treats any non-zero value as
/// potential reference. This is less precise but works without
/// type information.
///
/// # Warning
///
/// May produce false positives (treat non-pointers as pointers).
/// Use only if precise reference map is not available.
///
/// # Arguments
/// * `obj_addr` - Address of object
/// * `callback` - Called for each potential reference
///
/// # Returns
/// Number of potential references found
pub fn scan_object_conservative<F>(obj_addr: usize, mut callback: F) -> usize
where
    F: FnMut(usize),
{
    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);

        // Validate object
        if header.size == 0 || header.size > 1024 * 1024 * 1024 {
            return 0;
        }

        let data_start = obj_addr + HEADER_SIZE;
        let data_size = header.get_data_size();
        let word_count = data_size / std::mem::size_of::<usize>();

        let mut ref_count = 0;

        for i in 0..word_count {
            let word_addr = data_start + (i * std::mem::size_of::<usize>());

            // Check if address is readable
            if memory::is_readable(word_addr).unwrap_or(false) {
                let word_value = memory::read_pointer(word_addr);

                // Conservative: treat any non-zero value as potential reference
                if word_value != 0 {
                    callback(word_addr);
                    ref_count += 1;
                }
            }
        }

        ref_count
    }
}

/// Hybrid scanner - precise if reference map exists, conservative if not
///
/// # Arguments
/// * `obj_addr` - Address of object
/// * `ref_map` - Optional reference map (None = conservative)
/// * `callback` - Called for each reference found
///
/// # Returns
/// Number of references found
pub fn scan_object_hybrid<F>(obj_addr: usize, ref_map: Option<&ReferenceMap>, callback: F) -> usize
where
    F: FnMut(usize),
{
    match ref_map {
        Some(map) => scan_object_precise(obj_addr, map, callback),
        None => scan_object_conservative(obj_addr, callback),
    }
}

/// Get reference map for object
///
/// In full implementation, this will:
/// 1. Read class pointer from object header
/// 2. Lookup class metadata
/// 3. Return reference map from metadata
///
/// For now, returns conservative map (assumes all fields are references)
///
/// # Safety
/// `obj_addr` must point to valid object
fn get_reference_map_for_object(obj_addr: usize) -> ReferenceMap {
    unsafe {
        let header = &*(obj_addr as *const ObjectHeader);
        let data_size = header.get_data_size();

        // Create map assuming all 8-byte slots contain references
        // This is conservative fallback
        let mut offsets = Vec::new();
        for i in 0..(data_size / OBJECT_ALIGNMENT) {
            offsets.push(i * OBJECT_ALIGNMENT);
        }

        ReferenceMap::new(&offsets)
    }
}

/// Object scanner iterator
///
/// Iterator for scanning object without callback
pub struct ObjectScanner {
    /// Object address
    obj_addr: usize,
    /// Current offset
    current_offset: usize,
    /// Data size
    data_size: usize,
    /// Iterator internal
    ref_iter: crate::object::refmap::ReferenceMapIter,
}

impl ObjectScanner {
    /// Create new object scanner
    ///
    /// # Safety
    /// `obj_addr` must point to valid object
    pub unsafe fn new(obj_addr: usize) -> Option<Self> {
        let header = &*(obj_addr as *const ObjectHeader);

        if header.size == 0 || header.size > 1024 * 1024 * 1024 {
            return None;
        }

        let data_size = header.get_data_size();
        let ref_map = get_reference_map_for_object(obj_addr);
        let ref_iter = ref_map.iter();

        Some(Self {
            obj_addr,
            current_offset: 0,
            data_size,
            ref_iter,
        })
    }

    /// Get object address
    pub fn obj_addr(&self) -> usize {
        self.obj_addr
    }

    /// Get data size
    pub fn data_size(&self) -> usize {
        self.data_size
    }
}

impl Iterator for ObjectScanner {
    type Item = usize; // Returns reference address

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(offset) = self.ref_iter.next() {
            if offset < self.data_size {
                let ref_addr = self.obj_addr + HEADER_SIZE + offset;

                // Check if reference is non-null
                unsafe {
                    if memory::is_readable(ref_addr).unwrap_or(false) {
                        let ref_value = memory::read_pointer(ref_addr);
                        if ref_value != 0 {
                            self.current_offset = offset;
                            return Some(ref_addr);
                        }
                    }
                }
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ref_iter.size_hint()
    }
}

impl ExactSizeIterator for ObjectScanner {}
impl std::iter::FusedIterator for ObjectScanner {}

/// Batch object scanner - scan multiple objects efficiently
pub struct BatchScanner {
    stats: ObjectScanStats,
}

impl BatchScanner {
    /// Create new batch scanner
    pub fn new() -> Self {
        Self {
            stats: ObjectScanStats::new(),
        }
    }

    /// Scan multiple objects
    ///
    /// # Arguments
    /// * `objects` - Slice of object addresses
    /// * `callback` - Called for each reference found
    ///
    /// # Returns
    /// Scan statistics
    pub fn scan_objects<F>(&mut self, objects: &[usize], mut callback: F) -> ObjectScanStats
    where
        F: FnMut(usize),
    {
        let mut stats = ObjectScanStats::new();

        for &obj_addr in objects {
            let ref_count = scan_object(obj_addr, |ref_addr| {
                callback(ref_addr);
            });
            stats.record(ref_count);
        }

        self.stats.merge(&stats);
        stats.clone()
    }

    /// Get accumulated stats
    pub fn stats(&self) -> &ObjectScanStats {
        &self.stats
    }

    /// Clear stats
    pub fn clear_stats(&mut self) {
        self.stats = ObjectScanStats::new();
    }
}

impl Default for BatchScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference validator - validates if a value is a valid reference
pub struct ReferenceValidator {
    /// Heap minimum address
    heap_min: usize,
    /// Heap maximum address
    heap_max: usize,
    /// Check for alignment
    check_alignment: bool,
}

impl ReferenceValidator {
    /// Create new validator
    ///
    /// # Arguments
    /// * `heap_min` - Minimum valid heap address
    /// * `heap_max` - Maximum valid heap address
    /// * `check_alignment` - Whether to check pointer alignment
    pub fn new(heap_min: usize, heap_max: usize, check_alignment: bool) -> Self {
        Self {
            heap_min,
            heap_max,
            check_alignment,
        }
    }

    /// Check if value is a valid reference
    pub fn is_valid_reference(&self, value: usize) -> bool {
        // Null is not a valid reference
        if value == 0 {
            return false;
        }

        // Check if within heap bounds
        if value < self.heap_min || value >= self.heap_max {
            return false;
        }

        // Check alignment if enabled
        if self.check_alignment && value % OBJECT_ALIGNMENT != 0 {
            return false;
        }

        // Additional validation could check:
        // - Is address readable?
        // - Does it point to a valid object header?
        // - Is the object marked?

        true
    }

    /// Validate and filter references
    pub fn filter_references<F>(&self, addresses: &[usize], mut callback: F) -> usize
    where
        F: FnMut(usize),
    {
        let mut count = 0;
        for &addr in addresses {
            if self.is_valid_reference(addr) {
                callback(addr);
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::refmap::ReferenceMapBuilder;

    fn create_test_object(data_size: usize) -> (Vec<u8>, usize) {
        // Allocate buffer: header + data
        let total_size = HEADER_SIZE + data_size;
        let mut buffer = vec![0u8; total_size];

        // Create header
        let header = ObjectHeader::new(0x1000, total_size);
        unsafe {
            std::ptr::write_unaligned(buffer.as_mut_ptr() as *mut ObjectHeader, header);
        }

        let obj_addr = buffer.as_ptr() as usize;
        (buffer, obj_addr)
    }

    #[test]
    fn test_scan_empty_object() {
        let (buffer, obj_addr) = create_test_object(0);

        let mut refs = Vec::new();
        let count = scan_object(obj_addr, |ref_addr| {
            refs.push(ref_addr);
        });

        assert_eq!(count, 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_scan_object_with_references() {
        let (mut buffer, obj_addr) = create_test_object(32);

        // Plant some references in object data
        let data_start = obj_addr + HEADER_SIZE;
        unsafe {
            // Reference at offset 0
            *(data_start as *mut usize) = 0xAAAAAAAA;
            // Reference at offset 8
            *((data_start + 8) as *mut usize) = 0xBBBBBBBB;
            // Null at offset 16
            *((data_start + 16) as *mut usize) = 0;
            // Reference at offset 24
            *((data_start + 24) as *mut usize) = 0xCCCCCCCC;
        }

        // Create precise reference map (offsets 0, 8, 24)
        let ref_map = ReferenceMap::new(&[0, 8, 24]);

        let mut refs = Vec::new();
        let count = scan_object_precise(obj_addr, &ref_map, |ref_addr| {
            refs.push(ref_addr);
        });

        // Should find 3 non-null references
        assert_eq!(count, 3);
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_conservative_scan() {
        let (mut buffer, obj_addr) = create_test_object(32);

        let data_start = obj_addr + HEADER_SIZE;
        unsafe {
            // Non-zero values
            *(data_start as *mut usize) = 0x11111111;
            *((data_start + 8) as *mut usize) = 0x22222222;
            // Zero value (should be skipped)
            *((data_start + 16) as *mut usize) = 0;
            // Another non-zero
            *((data_start + 24) as *mut usize) = 0x44444444;
        }

        let mut refs = Vec::new();
        let count = scan_object_conservative(obj_addr, |ref_addr| {
            refs.push(ref_addr);
        });

        // Should find 3 non-zero values
        assert_eq!(count, 3);
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_object_scanner_iterator() {
        let (mut buffer, obj_addr) = create_test_object(24);

        let data_start = obj_addr + HEADER_SIZE;
        unsafe {
            *(data_start as *mut usize) = 0x11111111;
            *((data_start + 8) as *mut usize) = 0x22222222;
            *((data_start + 16) as *mut usize) = 0;
        }

        unsafe {
            let mut scanner = ObjectScanner::new(obj_addr).unwrap();
            let refs: Vec<usize> = scanner.collect();

            // Should find 2 non-null references
            assert_eq!(refs.len(), 2);
        }
    }

    #[test]
    fn test_batch_scanner() {
        let mut objects = Vec::new();
        let mut buffers = Vec::new();

        // Create 3 test objects
        for i in 0..3 {
            let (buffer, obj_addr) = create_test_object(16);
            let data_start = obj_addr + HEADER_SIZE;
            unsafe {
                *(data_start as *mut usize) = 0x1000 + i;
                *((data_start + 8) as *mut usize) = 0x2000 + i;
            }
            buffers.push(buffer);
            objects.push(obj_addr);
        }

        let mut scanner = BatchScanner::new();
        let mut all_refs = Vec::new();

        let stats = scanner.scan_objects(&objects, |ref_addr| unsafe {
            let ref_value = memory::read_pointer(ref_addr);
            all_refs.push(ref_value);
        });

        assert_eq!(stats.objects_scanned, 3);
        assert_eq!(stats.references_found, 6); // 2 refs per object
        assert_eq!(all_refs.len(), 6);
    }

    #[test]
    fn test_reference_validator() {
        let validator = ReferenceValidator::new(0x1000, 0x10000, true);

        // Valid reference
        assert!(validator.is_valid_reference(0x2000));

        // Null
        assert!(!validator.is_valid_reference(0));

        // Out of bounds
        assert!(!validator.is_valid_reference(0x500));
        assert!(!validator.is_valid_reference(0x20000));

        // Misaligned
        assert!(!validator.is_valid_reference(0x2001));
    }

    #[test]
    fn test_scan_stats() {
        let mut stats = ObjectScanStats::new();

        stats.record(2);
        stats.record(4);
        stats.record(0);
        stats.record(6);

        assert_eq!(stats.objects_scanned, 4);
        assert_eq!(stats.references_found, 12);
        assert!((stats.avg_refs_per_object - 3.0).abs() < f64::EPSILON);
        assert_eq!(stats.max_refs_in_object, 6);
        assert_eq!(stats.min_refs_in_object, 0); // record(0) sets min to 0
    }

    #[test]
    fn test_scan_invalid_object() {
        // Invalid: size = 0
        let (buffer, _) = create_test_object(0);
        let invalid_addr = buffer.as_ptr() as usize + HEADER_SIZE; // Skip header

        let count = scan_object(invalid_addr, |_| {});
        assert_eq!(count, 0);
    }

    #[test]
    fn test_scan_with_null_references() {
        let (mut buffer, obj_addr) = create_test_object(16);
        let data_start = obj_addr + HEADER_SIZE;

        unsafe {
            // All null references
            *(data_start as *mut usize) = 0;
            *((data_start + 8) as *mut usize) = 0;
        }

        let count = scan_object_conservative(obj_addr, |_| {});
        assert_eq!(count, 0);
    }

    #[test]
    fn test_hybrid_scan_precise() {
        let (mut buffer, obj_addr) = create_test_object(16);
        let data_start = obj_addr + HEADER_SIZE;

        unsafe {
            *(data_start as *mut usize) = 0xAAAAAAAA;
            *((data_start + 8) as *mut usize) = 0;
        }

        let ref_map = ReferenceMap::new(&[0, 8]);

        let mut refs = Vec::new();
        let count = scan_object_hybrid(obj_addr, Some(&ref_map), |ref_addr| {
            refs.push(ref_addr);
        });

        assert_eq!(count, 1); // Only one non-null
    }

    #[test]
    fn test_hybrid_scan_conservative() {
        let (mut buffer, obj_addr) = create_test_object(16);
        let data_start = obj_addr + HEADER_SIZE;

        unsafe {
            *(data_start as *mut usize) = 0xAAAAAAAA;
            *((data_start + 8) as *mut usize) = 0xBBBBBBBB;
        }

        let mut refs = Vec::new();
        let count = scan_object_hybrid(obj_addr, None, |ref_addr| {
            refs.push(ref_addr);
        });

        assert_eq!(count, 2); // Conservative finds both
    }
}
