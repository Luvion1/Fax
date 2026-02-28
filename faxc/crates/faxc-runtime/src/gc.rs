//! GC Runtime - C FFI wrapper for FGC
//!
//! Provides C-compatible functions for GC allocation and management

use fgc::{GcConfig, Runtime};
use std::sync::OnceLock;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn fax_gc_init() -> bool {
    if INITIALIZED.load(std::sync::atomic::Ordering::SeqCst) {
        return true;
    }

    let config = GcConfig::default();
    match Runtime::new(config) {
        Ok(runtime) => {
            if let Err(e) = runtime.start() {
                eprintln!("Failed to start GC runtime: {:?}", e);
                return false;
            }
            let _ = RUNTIME.set(runtime);
            INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
            true
        },
        Err(e) => {
            eprintln!("Failed to create GC runtime: {:?}", e);
            false
        },
    }
}

#[no_mangle]
pub extern "C" fn fax_gc_alloc(size: usize) -> *mut std::ffi::c_void {
    if !INITIALIZED.load(std::sync::atomic::Ordering::SeqCst) {
        fax_gc_init();
    }

    if let Some(runtime) = RUNTIME.get() {
        match runtime.allocate(size) {
            Ok(addr) => addr as *mut std::ffi::c_void,
            Err(e) => {
                eprintln!("GC allocation failed: {:?}", e);
                std::ptr::null_mut()
            },
        }
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn fax_gc_alloc_zeroed(size: usize) -> *mut std::ffi::c_void {
    let ptr = fax_gc_alloc(size);
    if !ptr.is_null() {
        unsafe {
            std::ptr::write_bytes(ptr, 0, size);
        }
    }
    ptr
}

#[no_mangle]
pub extern "C" fn fax_gc_register_root(ptr: *mut std::ffi::c_void) -> bool {
    if let Some(runtime) = RUNTIME.get() {
        if let Err(e) = runtime.gc().register_root(ptr as usize) {
            eprintln!("Failed to register root: {:?}", e);
            return false;
        }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn fax_gc_unregister_root(ptr: *mut std::ffi::c_void) -> bool {
    if let Some(runtime) = RUNTIME.get() {
        if let Err(e) = runtime.gc().unregister_root(ptr as usize) {
            eprintln!("Failed to unregister root: {:?}", e);
            return false;
        }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn fax_gc_collect() {
    if let Some(runtime) = RUNTIME.get() {
        let _ = runtime.request_gc(fgc::GcGeneration::Full);
    }
}

#[no_mangle]
pub extern "C" fn fax_gc_collect_young() {
    if let Some(runtime) = RUNTIME.get() {
        let _ = runtime.request_gc(fgc::GcGeneration::Young);
    }
}

#[no_mangle]
pub extern "C" fn fax_gc_shutdown() {
    if let Some(runtime) = RUNTIME.get() {
        let _ = runtime.stop();
    }
    INITIALIZED.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn fax_string_len(ptr: *const u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let mut len = 0usize;
        let mut current = ptr;
        while *current != 0 {
            len += 1;
            current = current.add(1);
        }
        len
    }
}

#[no_mangle]
pub extern "C" fn fax_string_concat(s1: *const u8, s2: *const u8) -> *mut u8 {
    if s1.is_null() || s2.is_null() {
        return std::ptr::null_mut();
    }

    let len1 = fax_string_len(s1);
    let len2 = fax_string_len(s2);
    let total_len = len1 + len2;

    let ptr = fax_gc_alloc(total_len + 1) as *mut u8;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(s1, ptr, len1);
        std::ptr::copy_nonoverlapping(s2, ptr.add(len1), len2);
        *ptr.add(total_len) = 0;
    }
    ptr
}

#[no_mangle]
pub extern "C" fn fax_string_eq(s1: *const u8, s2: *const u8) -> bool {
    if s1.is_null() || s2.is_null() {
        return s1.is_null() && s2.is_null();
    }

    let len1 = fax_string_len(s1);
    let len2 = fax_string_len(s2);

    if len1 != len2 {
        return false;
    }

    unsafe { std::slice::from_raw_parts(s1, len1) == std::slice::from_raw_parts(s2, len2) }
}

#[no_mangle]
pub extern "C" fn fax_string_cmp(s1: *const u8, s2: *const u8) -> i32 {
    if s1.is_null() {
        return if s2.is_null() { 0 } else { -1 };
    }
    if s2.is_null() {
        return 1;
    }

    let len1 = fax_string_len(s1);
    let len2 = fax_string_len(s2);
    let min_len = len1.min(len2);

    unsafe {
        let slice1 = std::slice::from_raw_parts(s1, min_len);
        let slice2 = std::slice::from_raw_parts(s2, min_len);

        for i in 0..min_len {
            if slice1[i] != slice2[i] {
                return if slice1[i] < slice2[i] { -1 } else { 1 };
            }
        }
    }

    if len1 < len2 {
        -1
    } else if len1 > len2 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn fax_array_len(ptr: *const u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let metadata = ptr.sub(std::mem::size_of::<usize>()) as *const usize;
        *metadata
    }
}

#[no_mangle]
pub extern "C" fn fax_array_get(ptr: *const u8, index: usize) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_array_len(ptr);
    if index >= len {
        return std::ptr::null_mut();
    }
    let elem_size = std::mem::size_of::<u64>();
    unsafe { ptr.add(index * elem_size) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn fax_array_set(ptr: *mut u8, index: usize, value: u64) -> bool {
    if ptr.is_null() {
        return false;
    }
    let len = fax_array_len(ptr);
    if index >= len {
        return false;
    }
    let elem_size = std::mem::size_of::<u64>();
    unsafe {
        let elem_ptr = ptr.add(index * elem_size) as *mut u64;
        *elem_ptr = value;
    }
    true
}

#[no_mangle]
pub extern "C" fn fax_panic(message: *const u8) {
    if message.is_null() {
        eprintln!("Fax: panic - unknown error");
    } else {
        let len = fax_string_len(message);
        unsafe {
            let slice = std::slice::from_raw_parts(message, len);
            let msg = String::from_utf8_lossy(slice);
            eprintln!("Fax: panic - {}", msg);
        }
    }
    std::process::exit(1);
}

#[no_mangle]
pub extern "C" fn fax_println(ptr: *const u8) {
    if ptr.is_null() {
        println!();
        return;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let msg = String::from_utf8_lossy(slice);
        println!("{}", msg);
    }
}

#[no_mangle]
pub extern "C" fn fax_print(ptr: *const u8) {
    if ptr.is_null() {
        return;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let msg = String::from_utf8_lossy(slice);
        print!("{}", msg);
    }
}

#[no_mangle]
pub extern "C" fn fax_bool_to_string(value: bool) -> *mut u8 {
    let s = if value { "true" } else { "false" };
    fax_string_from_str(s)
}

#[no_mangle]
pub extern "C" fn fax_char_to_string(c: u32) -> *mut u8 {
    let ch = char::from_u32(c).unwrap_or('\u{FFFD}');
    let mut buf = [0u8; 4];
    let encoded = ch.encode_utf8(&mut buf);
    fax_string_from_str(encoded)
}

fn fax_string_from_str(s: &str) -> *mut u8 {
    let len = s.len();
    let ptr = fax_gc_alloc(len + 1) as *mut u8;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
        *ptr.add(len) = 0;
    }
    ptr
}

#[no_mangle]
pub extern "C" fn fax_int_to_string(value: i64) -> *mut u8 {
    fax_string_from_str(&value.to_string())
}

#[no_mangle]
pub extern "C" fn fax_float_to_string(value: f64) -> *mut u8 {
    fax_string_from_str(&value.to_string())
}

#[no_mangle]
pub extern "C" fn fax_string_to_int(ptr: *const u8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = match std::str::from_utf8(slice) {
            Ok(st) => st,
            Err(_) => return 0,
        };
        s.parse().unwrap_or(0)
    }
}

#[no_mangle]
pub extern "C" fn fax_string_to_float(ptr: *const u8) -> f64 {
    if ptr.is_null() {
        return 0.0;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = match std::str::from_utf8(slice) {
            Ok(st) => st,
            Err(_) => return 0.0,
        };
        s.parse().unwrap_or(0.0)
    }
}

#[no_mangle]
pub extern "C" fn fax_int32_to_string(value: i32) -> *mut u8 {
    fax_string_from_str(&value.to_string())
}

#[no_mangle]
pub extern "C" fn fax_string_to_int32(ptr: *const u8) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = match std::str::from_utf8(slice) {
            Ok(st) => st,
            Err(_) => return 0,
        };
        s.parse().unwrap_or(0)
    }
}

#[no_mangle]
pub extern "C" fn fax_uint32_to_string(value: u32) -> *mut u8 {
    fax_string_from_str(&value.to_string())
}

#[no_mangle]
pub extern "C" fn fax_string_to_uint32(ptr: *const u8) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = match std::str::from_utf8(slice) {
            Ok(st) => st,
            Err(_) => return 0,
        };
        s.parse().unwrap_or(0)
    }
}

#[no_mangle]
pub extern "C" fn fax_array_create(size: usize, _element_size: usize) -> *mut u8 {
    let total_size = size * std::mem::size_of::<u64>() + std::mem::size_of::<usize>();
    let ptr = fax_gc_alloc_zeroed(total_size) as *mut u8;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let metadata_ptr = ptr.sub(std::mem::size_of::<usize>()) as *mut usize;
        *metadata_ptr = size;
    }
    ptr
}

#[no_mangle]
pub extern "C" fn fax_array_clone(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_array_len(ptr);
    let elem_size = std::mem::size_of::<u64>();
    let total_size = len * elem_size;
    let new_ptr = fax_gc_alloc_zeroed(total_size + std::mem::size_of::<usize>()) as *mut u8;
    if new_ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        // Copy metadata
        let src_meta = ptr.sub(std::mem::size_of::<usize>());
        let dst_meta = new_ptr.sub(std::mem::size_of::<usize>());
        std::ptr::copy_nonoverlapping(src_meta, dst_meta, std::mem::size_of::<usize>());

        // Copy data
        std::ptr::copy_nonoverlapping(ptr, new_ptr, total_size);
    }
    new_ptr
}

#[no_mangle]
pub extern "C" fn fax_string_clone(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    fax_string_from_str(unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    })
}

#[no_mangle]
pub extern "C" fn fax_string_slice(ptr: *const u8, start: usize, end: usize) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    let start = start.min(len);
    let end = end.min(len);
    if start >= end {
        return fax_string_from_str("");
    }
    let slice_len = end - start;
    unsafe {
        fax_string_from_str(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            ptr.add(start),
            slice_len,
        )))
    }
}

#[no_mangle]
pub extern "C" fn fax_string_contains(ptr: *const u8, substr: *const u8) -> bool {
    if ptr.is_null() || substr.is_null() {
        return false;
    }
    let len = fax_string_len(ptr);
    let sub_len = fax_string_len(substr);
    if sub_len > len {
        return false;
    }
    unsafe {
        let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len));
        let sub = std::str::from_utf8_unchecked(std::slice::from_raw_parts(substr, sub_len));
        s.contains(sub)
    }
}

#[no_mangle]
pub extern "C" fn fax_string_starts_with(ptr: *const u8, prefix: *const u8) -> bool {
    if ptr.is_null() || prefix.is_null() {
        return false;
    }
    let len = fax_string_len(ptr);
    let pre_len = fax_string_len(prefix);
    if pre_len > len {
        return false;
    }
    unsafe {
        let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len));
        let pre = std::str::from_utf8_unchecked(std::slice::from_raw_parts(prefix, pre_len));
        s.starts_with(pre)
    }
}

#[no_mangle]
pub extern "C" fn fax_string_ends_with(ptr: *const u8, suffix: *const u8) -> bool {
    if ptr.is_null() || suffix.is_null() {
        return false;
    }
    let len = fax_string_len(ptr);
    let suf_len = fax_string_len(suffix);
    if suf_len > len {
        return false;
    }
    unsafe {
        let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len));
        let suf = std::str::from_utf8_unchecked(std::slice::from_raw_parts(suffix, suf_len));
        s.ends_with(suf)
    }
}

#[no_mangle]
pub extern "C" fn fax_string_replace(ptr: *const u8, old: *const u8, new: *const u8) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };

    if old.is_null() {
        return fax_string_clone(ptr);
    }

    let old_str = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(old, fax_string_len(old)))
    };

    let new_str = if new.is_null() {
        ""
    } else {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(new, fax_string_len(new)))
        }
    };

    fax_string_from_str(&s.replace(old_str, new_str))
}

#[no_mangle]
pub extern "C" fn fax_string_trim(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    fax_string_from_str(s.trim())
}

#[no_mangle]
pub extern "C" fn fax_string_to_uppercase(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    fax_string_from_str(&s.to_uppercase())
}

#[no_mangle]
pub extern "C" fn fax_string_to_lowercase(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    fax_string_from_str(&s.to_lowercase())
}

#[no_mangle]
pub extern "C" fn fax_string_split(ptr: *const u8, delim: *const u8) -> *mut u8 {
    if ptr.is_null() || delim.is_null() {
        return std::ptr::null_mut();
    }
    let len = fax_string_len(ptr);
    let delim_len = fax_string_len(delim);
    if delim_len == 0 {
        return fax_string_clone(ptr);
    }
    let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    let d = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(delim, delim_len)) };

    if let Some(pos) = s.find(d) {
        fax_string_from_str(&s[..pos])
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn fax_assert(cond: bool, msg: *const u8) {
    if !cond {
        fax_panic(msg);
    }
}

#[no_mangle]
pub extern "C" fn fax_debug_println(ptr: *const u8, type_tag: i32) {
    if ptr.is_null() {
        println!("[DEBUG] (null) type_tag={}", type_tag);
        return;
    }
    let len = fax_string_len(ptr);
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let msg = String::from_utf8_lossy(slice);
        println!("[DEBUG] {} type_tag={}", msg, type_tag);
    }
}

#[no_mangle]
pub extern "C" fn fax_f64_math_sqrt(value: f64) -> f64 {
    value.sqrt()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

#[no_mangle]
pub extern "C" fn fax_f64_math_sin(value: f64) -> f64 {
    value.sin()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_cos(value: f64) -> f64 {
    value.cos()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_floor(value: f64) -> f64 {
    value.floor()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_ceil(value: f64) -> f64 {
    value.ceil()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_round(value: f64) -> f64 {
    value.round()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_abs(value: f64) -> f64 {
    value.abs()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_log(value: f64) -> f64 {
    value.ln()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_log10(value: f64) -> f64 {
    value.log10()
}

#[no_mangle]
pub extern "C" fn fax_f64_math_exp(value: f64) -> f64 {
    value.exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_string(s: &str) -> *mut u8 {
        fax_string_from_str(s)
    }

    #[test]
    fn test_string_len() {
        let s = make_test_string("hello");
        assert_eq!(fax_string_len(s), 5);
    }

    #[test]
    fn test_string_concat() {
        let s1 = make_test_string("hello");
        let s2 = make_test_string(" world");
        let result = fax_string_concat(s1, s2);
        assert_eq!(fax_string_len(result), 11);
    }

    #[test]
    fn test_string_eq() {
        let s1 = make_test_string("test");
        let s2 = make_test_string("test");
        let s3 = make_test_string("other");
        assert!(fax_string_eq(s1, s2));
        assert!(!fax_string_eq(s1, s3));
    }

    #[test]
    fn test_string_cmp() {
        let s1 = make_test_string("apple");
        let s2 = make_test_string("banana");
        let s3 = make_test_string("apple");
        assert!(fax_string_cmp(s1, s2) < 0);
        assert!(fax_string_cmp(s2, s1) > 0);
        assert_eq!(fax_string_cmp(s1, s3), 0);
    }

    #[test]
    fn test_bool_to_string() {
        let t = fax_bool_to_string(true);
        let f = fax_bool_to_string(false);
        assert_eq!(fax_string_len(t), 4);
        assert_eq!(fax_string_len(f), 5);
    }

    #[test]
    fn test_int_to_string() {
        let s = fax_int_to_string(42);
        let len = fax_string_len(s);
        assert!(len > 0);
    }

    #[test]
    fn test_string_to_int() {
        let s = make_test_string("123");
        assert_eq!(fax_string_to_int(s), 123);
        let s2 = make_test_string("-456");
        assert_eq!(fax_string_to_int(s2), -456);
    }

    #[test]
    fn test_string_clone() {
        let s = make_test_string("original");
        let cloned = fax_string_clone(s);
        assert!(fax_string_eq(s, cloned));
    }

    #[test]
    fn test_string_slice() {
        let s = make_test_string("hello world");
        let slice = fax_string_slice(s, 0, 5);
        assert_eq!(fax_string_len(slice), 5);
    }

    #[test]
    fn test_string_contains() {
        let s = make_test_string("hello world");
        let sub = make_test_string("world");
        let not_sub = make_test_string("foo");
        assert!(fax_string_contains(s, sub));
        assert!(!fax_string_contains(s, not_sub));
    }

    #[test]
    fn test_string_starts_with() {
        let s = make_test_string("hello world");
        let prefix = make_test_string("hello");
        let not_prefix = make_test_string("world");
        assert!(fax_string_starts_with(s, prefix));
        assert!(!fax_string_starts_with(s, not_prefix));
    }

    #[test]
    fn test_string_ends_with() {
        let s = make_test_string("hello world");
        let suffix = make_test_string("world");
        let not_suffix = make_test_string("hello");
        assert!(fax_string_ends_with(s, suffix));
        assert!(!fax_string_ends_with(s, not_suffix));
    }

    #[test]
    fn test_string_trim() {
        let s = make_test_string("  hello  ");
        let trimmed = fax_string_len(s);
        assert_eq!(trimmed, 9);
    }

    #[test]
    fn test_string_to_uppercase() {
        let s = make_test_string("hello");
        let upper = fax_string_to_uppercase(s);
        let content = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(upper, fax_string_len(upper)))
        };
        assert_eq!(content, "HELLO");
    }

    #[test]
    fn test_string_to_lowercase() {
        let s = make_test_string("HELLO");
        let lower = fax_string_to_lowercase(s);
        let content = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(lower, fax_string_len(lower)))
        };
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_string_replace() {
        let s = make_test_string("hello world");
        let old = make_test_string("world");
        let new = make_test_string("rust");
        let result = fax_string_replace(s, old, new);
        let content = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                result,
                fax_string_len(result),
            ))
        };
        assert_eq!(content, "hello rust");
    }

    #[test]
    fn test_float_math_sqrt() {
        assert!((fax_f64_math_sqrt(4.0) - 2.0).abs() < 1e-10);
        assert!((fax_f64_math_sqrt(9.0) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_math_pow() {
        assert!((fax_f64_math_pow(2.0, 3.0) - 8.0).abs() < 1e-10);
        assert!((fax_f64_math_pow(4.0, 0.5) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_math_trig() {
        let pi = std::f64::consts::PI;
        assert!((fax_f64_math_sin(0.0) - 0.0).abs() < 1e-10);
        assert!((fax_f64_math_cos(0.0) - 1.0).abs() < 1e-10);
        assert!((fax_f64_math_sin(pi) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_math_rounding() {
        assert!((fax_f64_math_floor(3.7) - 3.0).abs() < 1e-10);
        assert!((fax_f64_math_ceil(3.2) - 4.0).abs() < 1e-10);
        assert!((fax_f64_math_round(3.5) - 4.0).abs() < 1e-10);
        assert!((fax_f64_math_round(3.4) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_math_abs() {
        assert!((fax_f64_math_abs(-5.0) - 5.0).abs() < 1e-10);
        assert!((fax_f64_math_abs(5.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_math_log() {
        assert!((fax_f64_math_log(1.0) - 0.0).abs() < 1e-10);
        assert!((fax_f64_math_log(std::f64::consts::E) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_math_exp() {
        assert!((fax_f64_math_exp(0.0) - 1.0).abs() < 1e-10);
        assert!((fax_f64_math_exp(1.0) - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_array_create() {
        let arr = fax_array_create(5, 8);
        assert_eq!(fax_array_len(arr), 5);
    }

    #[test]
    fn test_array_get_set() {
        let arr = fax_array_create(5, 8);
        fax_array_set(arr, 0, 42);
        let val = fax_array_get(arr, 0) as *mut u64;
        unsafe {
            assert_eq!(*val, 42);
        }
    }

    #[test]
    fn test_char_to_string() {
        let s = fax_char_to_string(65); // 'A'
        assert!(fax_string_len(s) > 0);
    }

    #[test]
    fn test_float_to_string() {
        let s = fax_float_to_string(3.14);
        assert!(fax_string_len(s) > 0);
    }

    #[test]
    fn test_string_to_float() {
        let s = make_test_string("3.14");
        let val = fax_string_to_float(s);
        assert!((val - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_string_contains_negative() {
        let s = make_test_string("hello");
        let not_sub = make_test_string("xyz");
        assert!(!fax_string_contains(s, not_sub));
    }

    #[test]
    fn test_string_replace_no_match() {
        let s = make_test_string("hello");
        let old = make_test_string("xyz");
        let new = make_test_string("abc");
        let result = fax_string_replace(s, old, new);
        assert!(fax_string_eq(s, result));
    }

    #[test]
    fn test_array_out_of_bounds() {
        let arr = fax_array_create(3, 8);
        let val = fax_array_get(arr, 10);
        assert!(val.is_null());
        let result = fax_array_set(arr, 10, 42);
        assert!(!result);
    }

    #[test]
    fn test_array_clone() {
        let arr = fax_array_create(3, 8);
        fax_array_set(arr, 0, 1);
        fax_array_set(arr, 1, 2);
        fax_array_set(arr, 2, 3);

        let cloned = fax_array_clone(arr);
        assert_eq!(fax_array_len(arr), fax_array_len(cloned));
    }

    #[test]
    fn test_multiple_concats() {
        let s1 = make_test_string("a");
        let s2 = make_test_string("b");
        let s3 = make_test_string("c");

        let ab = fax_string_concat(s1, s2);
        let abc = fax_string_concat(ab, s3);
        assert!(!abc.is_null());
        assert_eq!(fax_string_len(abc), 3);
    }
}
