use serde_json::map::Map;
use serde_json::value::Value;

mod error;

pub fn flatten(
    nested_value: &Value,
    flat_value: &mut Value,
    parent_key: Option<String>,
    infer_type: bool,
    separator: Option<&str>,
) -> Result<(), error::Error> {
    // if object
    if let Some(nested_dict) = nested_value.as_object() {
        flatten_object(flat_value, &parent_key, nested_dict, infer_type, separator)?;
    } else if let Some(v_array) = nested_value.as_array() {
        let new_k = parent_key.unwrap_or_else(|| String::from(""));
        flatten_array(flat_value, &new_k, v_array, infer_type, separator)?;
    } else {
        log::error!("Expected object, found something else: {:?}", nested_value)
    }
    Ok(())
}

fn flatten_object(
    flat_value: &mut Value,
    parent_key: &Option<String>,
    nested_dict: &Map<String, Value>,
    infer_type: bool,
    separator: Option<&str>,
) -> Result<(), error::Error> {
    let sep = if let Some(sep) = separator { sep } else { "." };

    for (k, v) in nested_dict.iter() {
        let new_k = match parent_key {
            Some(ref key) => format!("{}{}{}", key, sep, k),
            None => k.clone(),
        };
        // if nested value is object recurse with parent_key
        if let Some(obj) = v.as_object() {
            flatten_object(flat_value, &Some(new_k), obj, infer_type, separator)?;
            // if array
        } else if let Some(v_array) = v.as_array() {
            // if array is not empty traverse array
            // if array is empty do not inset anything
            if !v_array.is_empty() {
                flatten_array(flat_value, &new_k, v_array, infer_type, separator)?;
            }
            // if no object or array insert value into the flat_value we're building
        } else if let Some(value) = flat_value.as_object_mut() {
            infer_type_and_insert(v, new_k, value, infer_type)?;
        }
    }
    Ok(())
}

fn infer_type_and_insert(
    v: &Value,
    new_k: String,
    value: &mut Map<String, Value>,
    infer_type: bool,
) -> Result<(), error::Error> {
    let new_val;
    if infer_type {
        if let Some(string) = v.as_str() {
            new_val = match string.parse::<i64>() {
                Ok(i) => serde_json::to_value(i)?,
                Err(_) => match string.parse::<f64>() {
                    Ok(f) => serde_json::to_value(f)?,
                    Err(_) => match string.parse::<bool>() {
                        Ok(b) => serde_json::to_value(b)?,
                        Err(_) => serde_json::to_value(string)?,
                    },
                },
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

fn flatten_array(
    flat_value: &mut Value,
    new_k: &str,
    v_array: &[Value],
    infer_type: bool,
    separator: Option<&str>,
) -> Result<(), error::Error> {
    for (i, obj) in v_array.iter().enumerate() {
        let array_key = format!("{}.{}", new_k, i);
        // if element is object or array recurse
        if obj.is_object() | obj.is_array() {
            flatten(obj, flat_value, Some(array_key), infer_type, separator)?;
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
    use serde_json::json;

    #[test]
    fn single_key_value() {
        let obj = json!({"key": "value"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(obj, json!(flat));
    }

    #[test]
    fn single_int_value() {
        let obj = json!({"key": 1});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key": 1}), json!(flat));
    }

    #[test]
    fn single_int_as_str_value() {
        let obj = json!({"key": "1"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key": 1}), json!(flat));
    }

    #[test]
    fn single_int_as_str_no_infer_type_value() {
        let obj = json!({"key": "1"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, false, None).unwrap();
        assert_eq!(json!({"key": "1"}), json!(flat));
    }

    #[test]
    fn single_float_as_str_value() {
        let obj = json!({"key": "1.0"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key": 1.}), json!(flat));
    }

    #[test]
    fn single_bool_as_str_value() {
        let obj = json!({"key": "true"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key": true}), json!(flat));
    }

    #[test]
    fn multi_key_value() {
        let obj = json!({"key1": "value1", "key2": "value2"});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(obj, json!(flat));
    }

    #[test]
    fn nested_single_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key":"value"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(
            json!({"key": "value", "nested_key.key": "value"}),
            json!(flat)
        );
    }

    #[test]
    fn nested_multiple_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key1":"value1", "key2": "value2"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(
            json!({"key": "value", "nested_key.key1": "value1", "nested_key.key2": "value2"}),
            json!(flat)
        );
    }

    #[test]
    fn top_level_array() {
        let obj = json!(["value1", "value2"]);

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({".0": "value1", ".1": "value2"}), json!(flat));
    }

    #[test]
    fn nested_array() {
        let obj = json!({"key": ["value1", "value2"]});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key.0": "value1", "key.1": "value2"}), json!(flat));
    }

    #[test]
    fn nested_obj_array() {
        let obj = json!({"key": ["value1", {"key": "value2"}]});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(
            json!({"key.0": "value1", "key.1.key": "value2"}),
            json!(flat)
        );
    }

    #[test]
    fn complex_nested_struct() {
        let obj = json!({"simple_key": "simple_value", "key": ["value1", {"key": "value2"}, {"nested_array": ["nested1", "nested2", ["nested3", "nested4"]]}]});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(
            json!({"simple_key": "simple_value", "key.0": "value1", "key.1.key": "value2", "key.2.nested_array.0": "nested1", "key.2.nested_array.1": "nested2", "key.2.nested_array.2.0": "nested3", "key.2.nested_array.2.1": "nested4"}),
            json!(flat)
        );
    }

    #[test]
    fn overlapping_after_flattening_array() {
        let obj = json!({"key": ["value1", "value2"], "key.0": "Oopsy"});

        let mut flat = json!({});
        let result = flatten(&obj, &mut flat, None, true, None);
        assert!(
            result.is_err(),
            "Flattening that JSON produces a collision so it should return Err"
        )
    }

    #[test]
    fn empty_array() {
        let obj = json!({"key": []});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({}), json!(flat));
    }

    #[test]
    fn empty_object() {
        let obj = json!({"key": {}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({}), json!(flat));
    }

    #[test]
    fn empty_complex_object() {
        let obj = json!({"key": {"key2": {}}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({}), json!(flat));
    }

    #[test]
    fn object_with_empty_array() {
        let obj = json!({"key": {"key2": []}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({}), json!(flat));
    }

    #[test]
    fn object_with_empty_array_and_string() {
        let obj = json!({"key": {"key2": [], "key3": "a"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key.key3": "a"}), json!(flat));
    }

    #[test]
    fn object_with_empty_object_and_string() {
        let obj = json!({"key": {"key2": {}, "key3": "a"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key.key3": "a"}), json!(flat));
    }

    #[test]
    fn empty_string_as_key() {
        let obj = json!({"key": {"": "a"}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key.": "a"}), json!(flat));
    }

    #[test]
    fn empty_string_as_key_multiple_times() {
        let obj = json!({"key": {"": {"": {"": "a"}}}});

        let mut flat = json!({});
        flatten(&obj, &mut flat, None, true, None).unwrap();
        assert_eq!(json!({"key...": "a"}), json!(flat));
    }

    #[test]
    fn flatten_plain_types() {
        let integer = json!(3);
        let string = json!("");
        let boolean = json!(false);

        for j in [integer, string, boolean] {
            let mut flat = json!({});
            flatten(&j, &mut flat, None, true, None).unwrap();
            assert_eq!(j, json!(flat));
        }
    }
}
