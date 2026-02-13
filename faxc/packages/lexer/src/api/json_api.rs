use serde_json;
use std::fs;

pub fn tokenize_json(input_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(input_path)?;
    let mut tokenizer = crate::lexer::tokenizer::Tokenizer::new(&source);
    let tokens = tokenizer.tokenize()?;
    let json_output = serde_json::to_string(&tokens)?;
    Ok(json_output)
}
