use std::os::raw::c_void;

#[repr(C)]
pub struct StackFrame {
    next: *mut StackFrame,
    roots: *mut *mut c_void,
    root_count: usize,
}

extern "C" {
    fn fax_gc_init();
    fn fax_gc_alloc(size: usize, ptr_map_ptr: *const usize, ptr_map_len: usize) -> *mut c_void;
    fn fax_gc_collect();
    fn fax_gc_push_frame(frame: *mut StackFrame);
    fn fax_gc_pop_frame();
}

unsafe fn create_rooted_frame(roots: &mut [*mut c_void]) -> StackFrame {
    StackFrame {
        next: std::ptr::null_mut(),
        roots: roots.as_mut_ptr(),
        root_count: roots.len(),
    }
}

fn main() {
    unsafe {
        println!("[Rust-Test] Starting GC tests with full debug logging...");
        fax_gc_init();

        println!("\n[Rust-Test] SCENARIO: Circular References (Unrooted)");
        {
            let ptr_map = [0usize];
            let a = fax_gc_alloc(8, ptr_map.as_ptr(), 1) as *mut *mut c_void;
            let b = fax_gc_alloc(8, ptr_map.as_ptr(), 1) as *mut *mut c_void;
            *a = b as *mut c_void;
            *b = a as *mut c_void;
            println!("[Rust-Test] Created A <-> B cycle. Requesting manual GC...");
            fax_gc_collect(); 
        }

        println!("\n[Rust-Test] SCENARIO: Deep Nesting (Linked List)");
        {
            let ptr_map = [0usize];
            let mut head: *mut c_void = std::ptr::null_mut();
            
            let mut roots = [head];
            let mut frame = create_rooted_frame(&mut roots);
            fax_gc_push_frame(&mut frame);

            println!("[Rust-Test] Allocating 6 nodes (will trigger automatic GC)...");
            for i in 0..6 {
                println!("[Rust-Test] Creating node {}", i);
                let node = fax_gc_alloc(16, ptr_map.as_ptr(), 1) as *mut *mut c_void;
                *node = roots[0]; 
                roots[0] = node as *mut c_void;
            }
            
            println!("[Rust-Test] Final manual GC collection...");
            fax_gc_collect(); 
            
            fax_gc_pop_frame();
            println!("[Rust-Test] Root removed. Objects should be freed in next cycle.");
            fax_gc_collect(); 
        }
        
        println!("\n[Rust-Test] All tests completed.");
    }
}