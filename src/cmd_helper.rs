use serde_json::{Value, Map};
use std::collections::HashMap;

// Translate tellraw JSON with substitutions
pub fn translate_tellraw(
    jsonc: &mut Value,
    selectors_sub: &HashMap<String, String>,
    scores_sub: &HashMap<String, HashMap<String, i32>>,
) -> Value {
    if let Some(rawtext) = jsonc.get_mut("rawtext") {
        if let Some(rawtext_array) = rawtext.as_array_mut() {
            for element in rawtext_array.iter_mut() {
                if let Some(score) = element.get("score") {
                    if let Some(score_obj) = score.as_object() {
                        if let (Some(name), Some(objective)) = (
                            score_obj.get("name").and_then(|v| v.as_str()),
                            score_obj.get("objective").and_then(|v| v.as_str()),
                        ) {
                            let replacement = if let Some(scb_data) = scores_sub.get(objective) {
                                if let Some(&value) = scb_data.get(name) {
                                    Value::String(value.to_string())
                                } else {
                                    Value::String(String::new())
                                }
                            } else {
                                Value::String(String::new())
                            };
                            
                            *element = Value::Object(Map::from_iter(vec![
                                ("text".to_string(), replacement)
                            ].into_iter()));
                        }
                    }
                } else if let Some(selector) = element.get("selector") {
                    if let Some(selector_str) = selector.as_str() {
                        let replacement = selectors_sub.get(selector_str)
                            .map(|s| Value::String(s.clone()))
                            .unwrap_or_else(|| Value::String(String::new()));
                        
                        *element = Value::Object(Map::from_iter(vec![
                            ("text".to_string(), replacement)
                        ].into_iter()));
                    }
                }
            }
        }
    }
    
    jsonc.clone()
}