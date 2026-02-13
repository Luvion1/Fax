use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

pub mod passes;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    #[serde(rename = "type")]
    pub typ: String,
    name: Option<String>,
    value: Option<Value>,
    op: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

pub struct Optimizer {
    level: u8,
    pub calls: HashSet<String>,
}

impl Optimizer {
    pub fn new(level: u8) -> Self {
        let mut calls = HashSet::new();
        for f in ["main", "print", "collect_gc"] {
            calls.insert(f.to_string());
        }
        Self { level, calls }
    }

    pub fn run(&mut self, mut root: Value) -> Value {
        self.find_calls(&root);
        if self.level >= 1 {
            root = self.constant_folding_pass(&root);
        }
        if self.level >= 2 {
            root = self.dead_code_elimination_pass(&root);
        }
        if self.level >= 3 {
            root = self.function_elimination_pass(&root);
        }
        root
    }

    fn find_calls(&mut self, v: &Value) {
        if let Some(obj) = v.as_object() {
            if obj.get("type").and_then(|t| t.as_str()) == Some("CallExpression") {
                if let Some(n) = obj.get("name").and_then(|n| n.as_str()) {
                    self.calls.insert(n.to_string());
                }
            }
            for val in obj.values() {
                self.find_calls(val);
            }
        } else if let Some(arr) = v.as_array() {
            for val in arr {
                self.find_calls(val);
            }
        }
    }
}
