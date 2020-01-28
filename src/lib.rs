#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use serde_json::value::Value;
use serde_json::map::Map;

error_chain! {
foreign_links {
        Json(::serde_json::Error);
    }
}

pub fn flatten(nested_value: &Value, flat_value: &mut Value, parent_key: Option<String>, infer_type: bool) -> Result<()> {
    // if object
    if let Some(nested_dict) = nested_value.as_object() {
        flatten_object(flat_value, &parent_key, nested_dict, infer_type)?;
    } else if let Some(v_array) = nested_value.as_array() {
        let new_k = parent_key.unwrap_or_else(||String::from(""));
        flatten_array(flat_value, &new_k, v_array, infer_type)?;
    } else {
        error!("Expected object, found something else: {:?}", nested_value)
    }
    Ok(())
}


fn flatten_object(flat_value: &mut Value, parent_key: &Option<String>, nested_dict: &Map<String, Value>, infer_type: bool) -> Result<()> {
    for (k, v) in nested_dict.iter() {
        let new_k = match parent_key {
            Some(ref key) => format!("{}.{}", key, k),
            None => k.clone()
        };
        // if nested value is object recurse with parent_key
        if let Some(obj) = v.as_object() {
            flatten_object(flat_value, &Some(new_k), obj, infer_type)?;
            // if array
        } else if let Some(v_array) = v.as_array() {
            // if array is not empty
            if !v_array.is_empty() {
                // traverse array
                flatten_array(flat_value, &new_k, v_array, infer_type)?;
                // if array is empty insert empty array into flat_value
            } else if let Some(value) = flat_value.as_object_mut() {
                let empty: Vec<Value> = vec!();
                value.insert(k.to_string(), json!(empty));
            }
            // if no object or array insert value into the flat_value we're building
        } else if let Some(value) = flat_value.as_object_mut() {
            infer_type_and_insert(v, new_k, value, infer_type)?;
        }
    }
    Ok(())
}

fn infer_type_and_insert(v: &Value, new_k: String, value: &mut Map<String, Value>, infer_type: bool) -> Result<()> {
    let new_val;
    if infer_type {
        if let Some(string) = v.as_str() {
            new_val = match string.parse::<i64>() {
                Ok(i) => serde_json::to_value(i)?,
                Err(_) => match string.parse::<f64>() {
                    Ok(f) => serde_json::to_value(f)?,
                    Err(_) => match string.parse::<bool>() {
                        Ok(b) => serde_json::to_value(b)?,
                        Err(_) => serde_json::to_value(string)?
                    }
                }
            };
        } else {
            new_val = v.clone();
        }
    } else {
        new_val = v.clone();
    };
    value.insert(new_k, new_val);
    Ok(())
}

fn flatten_array(flat_value: &mut Value, new_k: &str, v_array: &[Value], infer_type: bool) -> Result<()> {
    for (i, obj) in v_array.iter().enumerate() {
        let array_key = format!("{}.{}", new_k, i);
        // if element is object or array recurse
        if obj.is_object() | obj.is_array() {
            flatten(obj, flat_value, Some(array_key), infer_type)?;
            // else insert value in the flat_value we're building
        } else if let Some(value) = flat_value.as_object_mut() {
            infer_type_and_insert(obj, array_key, value, infer_type)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::flatten;

    #[test]
    fn single_key_value() {
        let obj = json!({"key": "value"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(obj, json!(flat));
    }

    #[test]
    fn single_int_value() {
        let obj = json!({"key": 1});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key": 1}), json!(flat));
    }

    #[test]
    fn single_int_as_str_value() {
        let obj = json!({"key": "1"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key": 1}), json!(flat));
    }

    #[test]
    fn single_int_as_str_no_infer_type_value() {
        let obj = json!({"key": "1"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, false).unwrap();
        assert_eq!(json!({"key": "1"}), json!(flat));
    }

    #[test]
    fn single_float_as_str_value() {
        let obj = json!({"key": "1.0"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key": 1.}), json!(flat));
    }

    #[test]
    fn single_bool_as_str_value() {
        let obj = json!({"key": "true"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key": true}), json!(flat));
    }

    #[test]
    fn multi_key_value() {
        let obj = json!({"key1": "value1", "key2": "value2"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(obj, json!(flat));
    }

    #[test]
    fn nested_single_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key":"value"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key": "value", "nested_key.key": "value"}), json!(flat));
    }


    #[test]
    fn nested_multiple_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key1":"value1", "key2": "value2"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key": "value", "nested_key.key1": "value1", "nested_key.key2": "value2"}), json!(flat));
    }

    #[test]
    fn top_level_array() {
        let obj = json!(["value1", "value2"]);

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({".0": "value1", ".1": "value2"}), json!(flat));
    }

    #[test]
    fn nested_array() {
        let obj = json!({"key": ["value1", "value2"]});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key.0": "value1", "key.1": "value2"}), json!(flat));
    }

    #[test]
    fn nested_obj_array() {
        let obj = json!({"key": ["value1", {"key": "value2"}]});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"key.0": "value1", "key.1.key": "value2"}), json!(flat));
    }

    #[test]
    fn complex_nested_struct() {
        let obj = json!({"simple_key": "simple_value", "key": ["value1", {"key": "value2"}, {"nested_array": ["nested1", "nested2", ["nested3", "nested4"]]}]});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true).unwrap();
        assert_eq!(json!({"simple_key": "simple_value", "key.0": "value1", "key.1.key": "value2", "key.2.nested_array.0": "nested1", "key.2.nested_array.1": "nested2", "key.2.nested_array.2.0": "nested3", "key.2.nested_array.2.1": "nested4"}), json!(flat));
    }
}
