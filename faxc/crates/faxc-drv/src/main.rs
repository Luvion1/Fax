fn main() {
    if let Err(e) = faxc_drv::main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
