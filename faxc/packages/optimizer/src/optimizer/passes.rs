use crate::optimizer::Optimizer;
use serde_json::{json, Value};

impl Optimizer {
    pub fn constant_folding_pass(&self, root: &Value) -> Value {
        self.fold_constants(root)
    }

    pub fn dead_code_elimination_pass(&self, root: &Value) -> Value {
        self.eliminate_dead_code(root)
    }

    pub fn function_elimination_pass(&self, root: &Value) -> Value {
        self.eliminate_unused_functions(root)
    }

    fn get_int_value(&self, v: &Value) -> Option<i64> {
        if let Some(s) = v.as_str() {
            return s.parse::<i64>().ok();
        }
        if let Some(obj) = v.as_object() {
            if obj.get("type").and_then(|t| t.as_str()) == Some("NumberLiteral") {
                return obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok());
            }
        }
        None
    }

    fn fold_constants(&self, v: &Value) -> Value {
        match v {
            Value::Object(obj) => {
                let processed_obj = self.process_object_fields(obj);

                if self.is_foldable_expression(&processed_obj) {
                    if let Some(result) = self.attempt_binary_fold(&processed_obj) {
                        return result;
                    }
                    if let Some(result) = self.attempt_comparison_fold(&processed_obj) {
                        return result;
                    }
                }

                Value::Object(processed_obj)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|val| self.fold_constants(val)).collect())
            }
            _ => v.clone(),
        }
    }

    fn process_object_fields(
        &self,
        obj: &serde_json::Map<String, Value>,
    ) -> serde_json::Map<String, Value> {
        let mut new_obj = serde_json::Map::new();
        for (key, val) in obj {
            new_obj.insert(key.clone(), self.fold_constants(val));
        }
        new_obj
    }

    fn is_foldable_expression(&self, obj: &serde_json::Map<String, Value>) -> bool {
        obj.get("type").and_then(|t| t.as_str()) == Some("BinaryExpression")
            || obj.get("type").and_then(|t| t.as_str()) == Some("ComparisonExpression")
    }

    fn attempt_binary_fold(&self, obj: &serde_json::Map<String, Value>) -> Option<Value> {
        if let (Some(left), Some(right), Some(op)) =
            (obj.get("left"), obj.get("right"), obj.get("op"))
        {
            let left_opt = self.fold_constants(left);
            let right_opt = self.fold_constants(right);

            if let (Some(li), Some(ri)) = (
                self.get_int_value(&left_opt),
                self.get_int_value(&right_opt),
            ) {
                let op_str = op.as_str().unwrap_or("");

                // Try Binary folding
                let bin_result = match op_str {
                    "add" => Some(li + ri),
                    "sub" => Some(li - ri),
                    "mul" => Some(li * ri),
                    "sdiv" if ri != 0 => Some(li / ri),
                    _ => None,
                };

                if let Some(res_val) = bin_result {
                    return Some(self.create_number_literal_node(res_val, obj));
                }
            }
        }
        None
    }

    fn attempt_comparison_fold(&self, obj: &serde_json::Map<String, Value>) -> Option<Value> {
        if let (Some(left), Some(right), Some(op)) =
            (obj.get("left"), obj.get("right"), obj.get("op"))
        {
            let left_opt = self.fold_constants(left);
            let right_opt = self.fold_constants(right);

            if let (Some(li), Some(ri)) = (
                self.get_int_value(&left_opt),
                self.get_int_value(&right_opt),
            ) {
                let op_str = op.as_str().unwrap_or("");

                // Try Comparison folding
                let cmp_result = match op_str {
                    "eq" => Some(li == ri),
                    "ne" => Some(li != ri),
                    "slt" => Some(li < ri),
                    "sgt" => Some(li > ri),
                    "sle" => Some(li <= ri),
                    "sge" => Some(li >= ri),
                    _ => None,
                };

                if let Some(res_val) = cmp_result {
                    return Some(self.create_boolean_literal_node(res_val, obj));
                }
            }
        }
        None
    }

    fn create_number_literal_node(
        &self,
        value: i64,
        original_obj: &serde_json::Map<String, Value>,
    ) -> Value {
        let mut res_node = serde_json::Map::new();
        res_node.insert("type".to_string(), json!("NumberLiteral"));
        res_node.insert("value".to_string(), json!(value.to_string()));
        if let Some(loc) = original_obj.get("loc") {
            res_node.insert("loc".to_string(), loc.clone());
        }
        Value::Object(res_node)
    }

    fn create_boolean_literal_node(
        &self,
        value: bool,
        original_obj: &serde_json::Map<String, Value>,
    ) -> Value {
        let mut res_node = serde_json::Map::new();
        res_node.insert("type".to_string(), json!("Boolean"));
        res_node.insert(
            "value".to_string(),
            json!(if value { "true" } else { "false" }),
        );
        if let Some(loc) = original_obj.get("loc") {
            res_node.insert("loc".to_string(), loc.clone());
        }
        Value::Object(res_node)
    }

    fn eliminate_dead_code(&self, v: &Value) -> Value {
        match v {
            Value::Object(obj) => {
                // Handle IfStatement elimination
                if obj.get("type").and_then(|t| t.as_str()) == Some("IfStatement") {
                    if let Some(condition) = obj.get("condition").and_then(|c| c.as_object()) {
                        if condition.get("type").and_then(|t| t.as_str()) == Some("Boolean") {
                            if let Some(value) = condition.get("value").and_then(|v| v.as_str()) {
                                let branch_key = if value == "true" {
                                    "then_branch"
                                } else {
                                    "else_branch"
                                };

                                if let Some(branch) = obj.get(branch_key) {
                                    return self.eliminate_dead_code(branch);
                                }
                            }
                        }
                    }
                }

                let mut new_obj = serde_json::Map::new();
                for (key, val) in obj {
                    new_obj.insert(key.clone(), self.eliminate_dead_code(val));
                }

                Value::Object(new_obj)
            }
            Value::Array(arr) => {
                let mut new_arr = Vec::new();
                for val in arr {
                    let opt_val = self.eliminate_dead_code(val);
                    if let Some(inner_arr) = opt_val.as_array() {
                        for item in inner_arr {
                            new_arr.push(item.clone());
                        }
                    } else {
                        new_arr.push(opt_val);
                    }
                }
                Value::Array(new_arr)
            }
            _ => v.clone(),
        }
    }

    fn eliminate_unused_functions(&self, v: &Value) -> Value {
        match v {
            Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();

                // Handle Program to remove unused functions
                if obj.get("type").and_then(|t| t.as_str()) == Some("Program") {
                    if let Some(body) = obj.get("body").and_then(|b| b.as_array()) {
                        let filtered_body: Vec<Value> = body
                            .iter()
                            .filter(|item| {
                                if item.get("type").and_then(|t| t.as_str())
                                    == Some("FunctionDeclaration")
                                {
                                    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                        return self.calls.contains(name);
                                    }
                                }
                                true
                            })
                            .map(|val| self.eliminate_unused_functions(val))
                            .collect();

                        let mut result_obj = obj.clone();
                        result_obj.insert("body".to_string(), Value::Array(filtered_body));
                        return Value::Object(result_obj);
                    }
                }

                for (key, val) in obj {
                    new_obj.insert(key.clone(), self.eliminate_unused_functions(val));
                }

                Value::Object(new_obj)
            }
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|val| self.eliminate_unused_functions(val))
                    .collect(),
            ),
            _ => v.clone(),
        }
    }
}
