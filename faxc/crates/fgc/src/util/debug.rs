//! Debug Utilities
//!
//! Helper functions untuk debugging dan logging.

/// Debug formatter untuk memory addresses
pub fn format_address(address: usize) -> String {
    format!("0x{:016X}", address)
}

/// Debug dump memory region
pub fn dump_region(start: usize, size: usize, lines: usize) {
    println!("Memory dump at {}:", format_address(start));

    for i in 0..lines {
        let offset = i * 16;
        if offset >= size {
            break;
        }

        let addr = start + offset;
        print!("  {}: ", format_address(addr));

        for j in 0..16 {
            if offset + j >= size {
                break;
            }

            let byte = unsafe { *((addr + j) as *const u8) };
            print!("{:02X} ", byte);
        }

        println!();
    }
}

/// Trace macro untuk debugging
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "trace") {
            println!("[TRACE] {}", format!($($arg)*));
        }
    };
}

/// Debug macro untuk conditional debugging
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!("[DEBUG] {}", format!($($arg)*));
        }
    };
}
