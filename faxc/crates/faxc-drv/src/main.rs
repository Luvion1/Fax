use faxc_drv::{main, Config, Session};

fn main() {
    if let Err(e) = main() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
