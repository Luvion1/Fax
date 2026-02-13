use serde_json::Value;
use std::fs;

pub fn optimize_json(input_path: &str, level: u8) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input_path)?;
    let ast: Value = serde_json::from_str(&content)?;

    let mut optimizer = crate::optimizer::Optimizer::new(level);
    let optimized_ast = optimizer.run(ast);

    let result = serde_json::to_string(&optimized_ast)?;
    Ok(result)
}
