//! Read Barrier Macros - Macros for Injecting Barriers in Code
//!
//! This module provides macros for injecting load barrier code
//! at various points in the application. These macros are used by:
//! - Code generators
//! - Runtime instrumentation
//! - Manual barrier insertion
//!
//! Available Macros:
//! - `read_barrier!` - Basic pointer read barrier
//! - `read_field_barrier!` - Field read with barrier
//! - `write_field_barrier!` - Field write with write barrier
//! - `array_read_barrier!` - Array element read barrier
//! - `object_read_barrier!` - Object read barrier

/// Macro for reading pointer with load barrier
///
/// Usage:
/// ```rust,no_run
/// let ptr = read_barrier!(my_pointer);
/// ```
///
/// This macro:
/// 1. Checks if pointer is null
/// 2. Applies load barrier for pointer healing
/// 3. Returns healed pointer
///
/// # FIX Issue 10
/// This macro uses the re-exported `heal_pointer` function from the barrier module.
/// The function is always available (no conditional compilation).
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::read_barrier;
///
/// let mut ptr: *mut i32 = get_object();
/// let healed_ptr = read_barrier!(ptr);
/// // Use healed_ptr...
/// ```
#[macro_export]
macro_rules! read_barrier {
    ($ptr:expr) => {{
        let mut addr = $ptr as usize;
        // FIX Issue 10: Use re-exported path for robustness
        $crate::barrier::heal_pointer(&mut addr);
        addr as *mut std::ffi::c_void
    }};
}

/// Macro for reading field from object with barrier
///
/// This macro reads a field from an object with load barrier
/// applied to the object.
///
/// # Arguments
/// * `$obj` - Object pointer
/// * `$field:ty` - Field type
/// * `$offset:expr` - Field offset from object start
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::read_field_barrier;
///
/// let obj = get_object();
/// let value: i32 = read_field_barrier!(obj, i32, 24); // Offset 24 = after header
/// ```
#[macro_export]
macro_rules! read_field_barrier {
    ($obj:expr, $field:ty, $offset:expr) => {{
        let obj_addr = $obj as *mut $field as usize;
        let field_addr = obj_addr + $offset;

        // Load barrier for object
        $crate::barrier::load_barrier::on_object_read(obj_addr);

        // Read field
        unsafe { *(field_addr as *const $field) }
    }};
}

/// Macro for writing field with write barrier (for generational mode)
///
/// This macro writes a field to an object with write barrier
/// applied to track cross-generational references.
///
/// # Arguments
/// * `$obj` - Object pointer
/// * `$field:ty` - Field type
/// * `$offset:expr` - Field offset from object start
/// * `$value:expr` - Value to write
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::write_field_barrier;
///
/// let obj = get_object();
/// write_field_barrier!(obj, *mut i32, 24, 42);
/// ```
#[macro_export]
macro_rules! write_field_barrier {
    ($obj:expr, $field:ty, $offset:expr, $value:expr) => {{
        let obj_addr = $obj as *mut $field as usize;

        // Write barrier (if generational mode active)
        if $crate::barrier::write_barrier::is_active() {
            $crate::barrier::write_barrier::on_field_write(obj_addr, $value as usize);
        }

        // Write field
        unsafe {
            *((obj_addr + $offset) as *mut $field) = $value;
        }
    }};
}

/// Macro for array element read with barrier
///
/// This macro reads an element from an array with load barrier applied.
///
/// # Arguments
/// * `$array` - Array pointer
/// * `$index:expr` - Element index
/// * `$elem_type:ty` - Element type
/// * `$elem_size:expr` - Size per element
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::array_read_barrier;
///
/// let array = get_array();
/// let elem: i32 = array_read_barrier!(array, 5, i32, 4);
/// ```
#[macro_export]
macro_rules! array_read_barrier {
    ($array:expr, $index:expr, $elem_type:ty, $elem_size:expr) => {{
        let array_addr = $array as usize;
        let elem_addr = array_addr + ($index * $elem_size);

        // Load barrier for array object
        $crate::barrier::load_barrier::on_object_read(array_addr);

        unsafe { *(elem_addr as *const $elem_type) }
    }};
}

/// Macro for array element write with barrier
///
/// # Arguments
/// * `$array` - Array pointer
/// * `$index:expr` - Element index
/// * `$elem_type:ty` - Element type
/// * `$elem_size:expr` - Size per element
/// * `$value:expr` - Value to write
#[macro_export]
macro_rules! array_write_barrier {
    ($array:expr, $index:expr, $elem_type:ty, $elem_size:expr, $value:expr) => {{
        let array_addr = $array as usize;
        let elem_addr = array_addr + ($index * $elem_size);

        // Write barrier
        if $crate::barrier::write_barrier::is_active() {
            $crate::barrier::write_barrier::on_field_write(array_addr, $value as usize);
        }

        unsafe {
            *(elem_addr as *mut $elem_type) = $value;
        }
    }};
}

/// Macro for object read with explicit barrier call
///
/// This macro applies load barrier to object and returns pointer
/// that has been healed.
///
/// # Arguments
/// * `$obj` - Object pointer
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::object_read_barrier;
///
/// let obj = get_object();
/// let healed = object_read_barrier!(obj);
/// ```
#[macro_export]
macro_rules! object_read_barrier {
    ($obj:expr) => {{
        let obj_addr = $obj as *mut _ as usize;
        $crate::barrier::load_barrier::on_object_read(obj_addr);
        $obj
    }};
}

/// Macro for reference read with barrier
///
/// This macro reads a reference and applies load barrier.
///
/// # Arguments
/// * `$ref` - Reference to read
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::ref_read_barrier;
///
/// let my_ref = get_reference();
/// let value = ref_read_barrier!(my_ref);
/// ```
#[macro_export]
macro_rules! ref_read_barrier {
    ($ref:expr) => {{
        let addr = &$ref as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $ref
    }};
}

/// Macro for pointer dereference with barrier
///
/// This macro dereferences a pointer with load barrier applied
/// to the object being pointed to.
///
/// # Arguments
/// * `$ptr` - Pointer to dereference
/// * `$type:ty` - Pointer type
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::ptr_deref_barrier;
///
/// let ptr: *mut i32 = get_pointer();
/// let value = ptr_deref_barrier!(ptr, i32);
/// ```
#[macro_export]
macro_rules! ptr_deref_barrier {
    ($ptr:expr, $type:ty) => {{
        let addr = $ptr as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        unsafe { *($ptr as *const $type) }
    }};
}

/// Macro for pointer dereference mutable with barrier
///
/// # Arguments
/// * `$ptr` - Pointer to dereference
/// * `$type:ty` - Pointer type
#[macro_export]
macro_rules! ptr_deref_mut_barrier {
    ($ptr:expr, $type:ty) => {{
        let addr = $ptr as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        unsafe { &mut *($ptr as *mut $type) }
    }};
}

/// Macro for slice element access with barrier
///
/// # Arguments
/// * `$slice` - Slice
/// * `$index:expr` - Element index
#[macro_export]
macro_rules! slice_access_barrier {
    ($slice:expr, $index:expr) => {{
        let slice_ptr = $slice.as_ptr() as usize;
        $crate::barrier::load_barrier::on_object_read(slice_ptr);
        &$slice[$index]
    }};
}

/// Macro for slice mutable element access with barrier
///
/// # Arguments
/// * `$slice` - Slice
/// * `$index:expr` - Element index
#[macro_export]
macro_rules! slice_access_mut_barrier {
    ($slice:expr, $index:expr) => {{
        let slice_ptr = $slice.as_ptr() as usize;
        $crate::barrier::load_barrier::on_object_read(slice_ptr);
        &mut $slice[$index]
    }};
}

/// Macro for box dereference with barrier
///
/// # Arguments
/// * `$box` - Box to dereference
#[macro_export]
macro_rules! box_deref_barrier {
    ($box:expr) => {{
        let addr = $box.as_ref() as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &$box
    }};
}

/// Macro for rc dereference with barrier
///
/// # Arguments
/// * `$rc` - Rc to dereference
#[macro_export]
macro_rules! rc_deref_barrier {
    ($rc:expr) => {{
        let addr = $rc.as_ref() as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &$rc
    }};
}

/// Macro for arc dereference with barrier
///
/// # Arguments
/// * `$arc` - Arc to dereference
#[macro_export]
macro_rules! arc_deref_barrier {
    ($arc:expr) => {{
        let addr = $arc.as_ref() as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &$arc
    }};
}

/// Macro for method call with barrier
///
/// This macro applies load barrier before method call.
///
/// # Arguments
/// * `$obj` - Object for method call
/// * `$method:ident` - Method name
/// * `$($args:expr),*` - Method arguments
///
/// # Examples
///
/// ```rust,no_run
/// use fgc::method_call_barrier;
///
/// let obj = get_object();
/// let result = method_call_barrier!(obj, my_method, arg1, arg2);
/// ```
#[macro_export]
macro_rules! method_call_barrier {
    ($obj:expr, $method:ident) => {{
        let addr = std::ptr::addr_of!($obj) as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $obj.$method()
    }};
    ($obj:expr, $method:ident, $($args:expr),*) => {{
        let addr = std::ptr::addr_of!($obj) as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $obj.$method($($args),*)
    }};
}

/// Macro for vtable call with barrier
///
/// This macro applies load barrier before virtual method call.
///
/// # Arguments
/// * `$trait_obj` - Trait object
/// * `$method:ident` - Method name
/// * `$($args:expr),*` - Method arguments
#[macro_export]
macro_rules! vtable_call_barrier {
    ($trait_obj:expr, $method:ident) => {{
        let addr = $trait_obj.as_ref() as *const _ as *const () as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $trait_obj.$method()
    }};
    ($trait_obj:expr, $method:ident, $($args:expr),*) => {{
        let addr = $trait_obj.as_ref() as *const _ as *const () as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $trait_obj.$method($($args),*)
    }};
}

/// Macro for closure call with barrier
///
/// # Arguments
/// * `$closure` - Closure
/// * `$($args:expr),*` - Closure arguments
#[macro_export]
macro_rules! closure_call_barrier {
    ($closure:expr) => {{
        let addr = &$closure as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $closure()
    }};
    ($closure:expr, $($args:expr),*) => {{
        let addr = &$closure as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $closure($($args),*)
    }};
}

/// Macro for struct field access with named field
///
/// # Arguments
/// * `$obj` - Object
/// * `$field:ident` - Field name
#[macro_export]
macro_rules! field_access_barrier {
    ($obj:expr, $field:ident) => {{
        let addr = &$obj as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &$obj.$field
    }};
}

/// Macro for struct field mutable access
///
/// # Arguments
/// * `$obj` - Object
/// * `$field:ident` - Field name
#[macro_export]
macro_rules! field_access_mut_barrier {
    ($obj:expr, $field:ident) => {{
        let addr = &$obj as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &mut $obj.$field
    }};
}

/// Macro for tuple element access
///
/// # Arguments
/// * `$tuple` - Tuple
/// * `$index:tt` - Element index (0-based)
#[macro_export]
macro_rules! tuple_access_barrier {
    ($tuple:expr, $index:tt) => {{
        let addr = &$tuple as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &$tuple.$index
    }};
}

/// Macro for option unwrap with barrier
///
/// # Arguments
/// * `$opt` - Option
#[macro_export]
macro_rules! option_unwrap_barrier {
    ($opt:expr) => {{
        let addr = &$opt as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $opt.unwrap()
    }};
}

/// Macro for result unwrap with barrier
///
/// # Arguments
/// * `$res` - Result
#[macro_export]
macro_rules! result_unwrap_barrier {
    ($res:expr) => {{
        let addr = &$res as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $res.unwrap()
    }};
}

/// Macro for iterator next with barrier
///
/// # Arguments
/// * `$iter` - Iterator
#[macro_export]
macro_rules! iterator_next_barrier {
    ($iter:expr) => {{
        let addr = &$iter as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        $iter.next()
    }};
}

/// Macro for index operation with barrier
///
/// # Arguments
/// * `$container` - Container
/// * `$index:expr` - Index
#[macro_export]
macro_rules! index_barrier {
    ($container:expr, $index:expr) => {{
        let addr = &$container as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &$container[$index]
    }};
}

/// Macro for index mutable operation with barrier
///
/// # Arguments
/// * `$container` - Container
/// * `$index:expr` - Index
#[macro_export]
macro_rules! index_mut_barrier {
    ($container:expr, $index:expr) => {{
        let addr = &$container as *const _ as usize;
        $crate::barrier::load_barrier::on_object_read(addr);
        &mut $container[$index]
    }};
}

/// Write barrier module placeholder
///
/// Write barrier is used for generational GC to track
/// cross-generational references.
pub mod write_barrier {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    static WRITE_BARRIER_ACTIVE: AtomicBool = AtomicBool::new(false);

    lazy_static::lazy_static! {
        static ref REMEMBERED_SET: Mutex<RememberedSet> = Mutex::new(RememberedSet::new());
    }

    #[allow(dead_code)]
    const CARD_SIZE: usize = 512;
    #[allow(dead_code)]
    const CARD_SHIFT: usize = 9;

    /// Remembered set for tracking cross-generational references
    struct RememberedSet {
        cards: Vec<u8>,
        card_count: usize,
    }

    impl RememberedSet {
        fn new() -> Self {
            Self {
                cards: Vec::new(),
                card_count: 0,
            }
        }

        fn mark_card(&mut self, address: usize) {
            let card_index = address >> CARD_SHIFT;
            let byte_index = card_index / 8;
            let bit_index = card_index % 8;

            if byte_index >= self.cards.len() {
                self.cards.resize(byte_index + 1, 0);
            }

            let mask = 1 << bit_index;
            if self.cards[byte_index] & mask == 0 {
                self.card_count += 1;
                self.cards[byte_index] |= mask;
            }
        }

        fn is_card_marked(&self, address: usize) -> bool {
            let card_index = address >> CARD_SHIFT;
            let byte_index = card_index / 8;
            let bit_index = card_index % 8;

            if byte_index >= self.cards.len() {
                return false;
            }

            (self.cards[byte_index] & (1 << bit_index)) != 0
        }

        fn clear(&mut self) {
            for card in self.cards.iter_mut() {
                *card = 0;
            }
            self.card_count = 0;
        }

        fn count(&self) -> usize {
            self.card_count
        }
    }

    /// Check if write barrier is active
    #[inline]
    pub fn is_active() -> bool {
        WRITE_BARRIER_ACTIVE.load(Ordering::Relaxed)
    }

    /// Enable write barrier
    #[inline]
    pub fn enable() {
        WRITE_BARRIER_ACTIVE.store(true, Ordering::Relaxed);
    }

    /// Disable write barrier
    #[inline]
    pub fn disable() {
        WRITE_BARRIER_ACTIVE.store(false, Ordering::Relaxed);
    }

    /// Called on field write
    ///
    /// # Arguments
    /// * `obj_addr` - Object address (in old generation)
    /// * `value` - Value being written (reference to young generation)
    #[inline]
    pub fn on_field_write(obj_addr: usize, value: usize) {
        if !is_active() || value == 0 {
            return;
        }

        if is_cross_generational(obj_addr, value) {
            if let Ok(mut remembered_set) = REMEMBERED_SET.lock() {
                remembered_set.mark_card(obj_addr);
            }
        }
    }

    /// Check if reference crosses generations
    fn is_cross_generational(old_addr: usize, young_addr: usize) -> bool {
        let old_gen = get_generation(old_addr);
        let young_gen = get_generation(young_addr);

        old_gen == 1 && young_gen == 0
    }

    /// Get generation for address (simplified)
    fn get_generation(addr: usize) -> u8 {
        if addr < 0x8000_0000 {
            0
        } else {
            1
        }
    }

    /// Get remembered set size
    pub fn remembered_set_size() -> usize {
        if let Ok(remembered_set) = REMEMBERED_SET.lock() {
            remembered_set.count()
        } else {
            0
        }
    }

    /// Clear remembered set
    pub fn clear_remembered_set() {
        if let Ok(mut remembered_set) = REMEMBERED_SET.lock() {
            remembered_set.clear();
        }
    }

    /// Check if card is marked
    pub fn is_card_marked(address: usize) -> bool {
        if let Ok(remembered_set) = REMEMBERED_SET.lock() {
            remembered_set.is_card_marked(address)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::write_barrier;
    use crate::object::{ObjectHeader, HEADER_SIZE};

    // Helper to create test object
    fn create_test_object() -> (Vec<u8>, usize) {
        let size = HEADER_SIZE + 64;
        let mut buffer = vec![0u8; size];

        unsafe {
            std::ptr::write(
                buffer.as_mut_ptr() as *mut ObjectHeader,
                ObjectHeader::new(0x1000, size),
            );
        }

        let addr = buffer.as_ptr() as usize;
        (buffer, addr)
    }

    // === Macro Expansion Tests ===

    #[test]
    fn test_read_barrier_macro() {
        let (_buffer, addr) = create_test_object();
        let ptr = addr as *mut i32;

        // Macro should expand without error
        let _healed = read_barrier!(ptr);
    }

    #[test]
    fn test_read_field_barrier_macro() {
        let (_buffer, addr) = create_test_object();
        let obj = addr as *mut i32;

        // Macro should expand without error
        let _value: i32 = read_field_barrier!(obj, i32, 0);
    }

    #[test]
    fn test_write_field_barrier_macro() {
        let (_buffer, addr) = create_test_object();
        let obj = addr as *mut i32;

        // Macro should expand without error
        write_field_barrier!(obj, i32, 0, 42);

        unsafe {
            assert_eq!(*obj, 42);
        }
    }

    #[test]
    fn test_array_read_barrier_macro() {
        let mut array = [1i32, 2, 3, 4, 5];
        let ptr = array.as_mut_ptr();

        let value: i32 = array_read_barrier!(ptr, 2, i32, 4);
        assert_eq!(value, 3);
    }

    #[test]
    fn test_array_write_barrier_macro() {
        let mut array = [1i32, 2, 3, 4, 5];
        let ptr = array.as_mut_ptr();

        array_write_barrier!(ptr, 2, i32, 4, 42);
        assert_eq!(array[2], 42);
    }

    #[test]
    fn test_object_read_barrier_macro() {
        let (_buffer, addr) = create_test_object();
        let obj = addr as *mut i32;

        let _healed = object_read_barrier!(obj);
    }

    #[test]
    fn test_ref_read_barrier_macro() {
        let value = 42i32;
        let my_ref = &value;

        let _val = ref_read_barrier!(my_ref);
        assert_eq!(_val, &42);
    }

    #[test]
    fn test_ptr_deref_barrier_macro() {
        let value = 42i32;
        let ptr = &value as *const i32;

        let _val = ptr_deref_barrier!(ptr, i32);
        assert_eq!(_val, 42);
    }

    #[test]
    fn test_ptr_deref_mut_barrier_macro() {
        let mut value = 42i32;
        let ptr = &mut value as *mut i32;

        let _val = ptr_deref_mut_barrier!(ptr, i32);
        *_val = 100;
        assert_eq!(value, 100);
    }

    #[test]
    fn test_slice_access_barrier_macro() {
        let slice = [1i32, 2, 3, 4, 5];

        let val = slice_access_barrier!(slice, 2);
        assert_eq!(*val, 3);
    }

    #[test]
    fn test_slice_access_mut_barrier_macro() {
        let mut slice = [1i32, 2, 3, 4, 5];

        let val = slice_access_mut_barrier!(slice, 2);
        *val = 42;
        assert_eq!(slice[2], 42);
    }

    #[test]
    fn test_box_deref_barrier_macro() {
        let box_val = Box::new(42i32);

        let _val = box_deref_barrier!(box_val);
        assert_eq!(**_val, 42);
    }

    #[test]
    fn test_rc_deref_barrier_macro() {
        use std::rc::Rc;

        let rc_val = Rc::new(42i32);
        let _val = rc_deref_barrier!(rc_val);
        assert_eq!(**_val, 42);
    }

    #[test]
    fn test_arc_deref_barrier_macro() {
        use std::sync::Arc;

        let arc_val = Arc::new(42i32);
        let _val = arc_deref_barrier!(arc_val);
        assert_eq!(**_val, 42);
    }

    #[test]
    fn test_method_call_barrier_macro() {
        struct TestStruct {
            value: i32,
        }

        impl TestStruct {
            fn get_value(&self) -> i32 {
                self.value
            }

            fn add(&self, x: i32) -> i32 {
                self.value + x
            }
        }

        let obj = Box::new(TestStruct { value: 42 });
        let obj_ref = &*obj;

        let result = method_call_barrier!(obj_ref, get_value);
        assert_eq!(result, 42);

        let result = method_call_barrier!(obj_ref, add, 10);
        assert_eq!(result, 52);
    }

    #[test]
    fn test_field_access_barrier_macro() {
        struct TestStruct {
            value: i32,
        }

        let obj = TestStruct { value: 42 };

        let val = field_access_barrier!(obj, value);
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_field_access_mut_barrier_macro() {
        struct TestStruct {
            value: i32,
        }

        let mut obj = TestStruct { value: 42 };

        let val = field_access_mut_barrier!(obj, value);
        *val = 100;
        assert_eq!(obj.value, 100);
    }

    #[test]
    fn test_tuple_access_barrier_macro() {
        let tuple = (1i32, 2i32, 3i32);

        let val = tuple_access_barrier!(tuple, 1);
        assert_eq!(*val, 2);
    }

    #[test]
    fn test_option_unwrap_barrier_macro() {
        let opt = Some(42i32);

        let val = option_unwrap_barrier!(opt);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_result_unwrap_barrier_macro() {
        let res: Result<i32, ()> = Ok(42);

        let val = result_unwrap_barrier!(res);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_iterator_next_barrier_macro() {
        let mut iter = vec![1, 2, 3].into_iter();

        let val = iterator_next_barrier!(iter);
        assert_eq!(val, Some(1));
    }

    #[test]
    fn test_index_barrier_macro() {
        let vec = vec![1i32, 2, 3, 4, 5];

        let val = index_barrier!(vec, 2);
        assert_eq!(*val, 3);
    }

    #[test]
    fn test_index_mut_barrier_macro() {
        let mut vec = vec![1i32, 2, 3, 4, 5];

        let val = index_mut_barrier!(vec, 2);
        *val = 42;
        assert_eq!(vec[2], 42);
    }

    #[test]
    fn test_closure_call_barrier_macro() {
        let closure = || 42i32;

        let val = closure_call_barrier!(closure);
        assert_eq!(val, 42);

        let closure_with_args = |x: i32| x + 10;
        let val = closure_call_barrier!(closure_with_args, 32);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_vtable_call_barrier_macro() {
        trait Trait {
            fn value(&self) -> i32;
            fn add(&self, x: i32) -> i32;
        }

        struct Impl {
            v: i32,
        }

        impl Trait for Impl {
            fn value(&self) -> i32 {
                self.v
            }

            fn add(&self, x: i32) -> i32 {
                self.v + x
            }
        }

        let obj: Box<dyn Trait> = Box::new(Impl { v: 42 });

        let val = vtable_call_barrier!(&obj, value);
        assert_eq!(val, 42);

        let val = vtable_call_barrier!(&obj, add, 10);
        assert_eq!(val, 52);
    }

    // === Write Barrier Tests ===

    #[test]
    fn test_write_barrier_toggle() {
        assert!(!write_barrier::is_active());

        write_barrier::enable();
        assert!(write_barrier::is_active());

        write_barrier::disable();
        assert!(!write_barrier::is_active());
    }

    #[test]
    fn test_write_barrier_on_field_write() {
        write_barrier::enable();

        // Should not panic
        write_barrier::on_field_write(0x1000, 0x2000);

        write_barrier::disable();
    }

    // === Integration Tests ===

    #[test]
    fn test_macro_chain() {
        let mut vec = vec![1i32, 2, 3];

        // Chain multiple barrier macros
        let val = index_barrier!(vec, 0);
        let doubled = *val * 2;

        let val_mut = index_mut_barrier!(vec, 1);
        *val_mut = doubled;

        assert_eq!(vec[1], 2);
    }

    #[test]
    fn test_nested_macro_calls() {
        let tuple = (Some(42i32), vec![1, 2, 3]);

        // Nested macro calls
        let opt_val = option_unwrap_barrier!(tuple.0);
        let vec_val = index_barrier!(tuple.1, 0);

        assert_eq!(opt_val, 42);
        assert_eq!(*vec_val, 1);
    }
}
