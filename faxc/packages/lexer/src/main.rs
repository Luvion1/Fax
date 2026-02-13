use std::env;

pub mod api;
pub mod lexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    match api::json_api::tokenize_json(input_path) {
        Ok(json_output) => {
            let mut val: serde_json::Value = serde_json::from_str(&json_output)?;
            if let Some(obj) = val.as_object_mut() {
                obj.insert("source_file".to_string(), serde_json::json!(input_path));
            } else {
                // If it's just an array, wrap it
                val = serde_json::json!({
                    "source_file": input_path,
                    "tokens": val
                });
            }
            println!("{}", serde_json::to_string(&val)?);
            Ok(())
        }
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1)
        }
    }
}
