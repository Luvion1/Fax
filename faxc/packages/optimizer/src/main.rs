use fax_opt::api::json_api;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file> [--opt-level=<level>]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let mut level = 2; // Default to intermediate

    for arg in &args {
        if arg.starts_with("--opt-level=") {
            level = match arg.split('=').nth(1).unwrap() {
                "none" => 0,
                "basic" => 1,
                "intermediate" => 2,
                "advanced" => 3,
                "aggressive" => 4,
                _ => 2,
            };
        }
    }

    match json_api::optimize_json(input_path, level) {
        Ok(json_output) => {
            println!("{}", json_output);
            Ok(())
        }
        Err(e) => {
            eprintln!("Optimizer error: {}", e);
            std::process::exit(1)
        }
    }
}
