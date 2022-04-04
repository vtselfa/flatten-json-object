use serde_json::value::Map;
use serde_json::value::Value;

mod error;

pub fn flatten(
    to_flatten: &Value, // The part of the json that we are iterating
) -> Result<Value, error::Error> {
    let mut flat = Map::<String, Value>::new();
    flatten_value(to_flatten, "".to_owned(), 0, &mut flat).map(|_x| Value::Object(flat))
}

fn flatten_value(
    current: &Value,                    // The part of the json that we are iterating
    parent_key: String, // The concatenated keys of the ancestors of the current value
    depth: u32,         // The depth were we are, starting from 0
    flattened: &mut Map<String, Value>, // Were we accumulate the resulting flattened json
) -> Result<(), error::Error> {
    if depth == 0 && !current.is_object() {
        return Err(error::Error::FirstLevelMustBeAnObject);
    }

    if let Some(current) = current.as_object() {
        flatten_object(current, parent_key, depth, flattened)?;
    } else if let Some(current) = current.as_array() {
        flatten_array(current, parent_key, depth, flattened)?;
    } else {
        if flattened.contains_key(&parent_key) {
            return Err(error::Error::KeyWillBeOverwritten(parent_key));
        }
        flattened.insert(parent_key, current.clone());
    }
    Ok(())
}

fn flatten_object(
    current: &Map<String, Value>,
    parent_key: String, // The concatenated keys of the ancestors of the current value
    depth: u32,         // The depth were we are, starting from 0
    flattened: &mut Map<String, Value>, // Were we accumulate the resulting flattened json
) -> Result<(), error::Error> {
    for (k, v) in current.iter() {
        let parent_key = if depth > 0 {
            format!("{}.{}", parent_key, k)
        } else {
            k.to_string()
        };
        flatten_value(v, parent_key, depth + 1, flattened)?;
    }
    Ok(())
}

fn flatten_array(
    current: &[Value],
    parent_key: String, // The concatenated keys of the ancestors of the current value
    depth: u32,         // The depth were we are, starting from 0
    flattened: &mut Map<String, Value>, // Were we accumulate the resulting flattened json
) -> Result<(), error::Error> {
    for (i, obj) in current.iter().enumerate() {
        let parent_key = if depth > 0 {
            format!("{}.{}", parent_key, i)
        } else {
            format!("{}", i)
        };
        flatten_value(obj, parent_key, depth + 1, flattened)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::flatten;
    use crate::error::Error;
    use serde_json::json;

    #[test]
    fn single_key_value() {
        let obj = json!({"key": "value"});
        assert_eq!(obj, flatten(&obj).unwrap());
    }

    #[test]
    fn object_with_plain_values() {
        let obj = json!({"int": 1, "float": 2.0, "str": "a", "bool": true, "null": null});
        assert_eq!(obj, flatten(&obj).unwrap());
    }

    #[test]
    fn array_with_plain_values() {
        let obj = json!({"a": [1, 2.0, "b", null, true]});
        assert_eq!(
            flatten(&obj).unwrap(),
            json!({"a.0": 1, "a.1": 2.0, "a.2": "b", "a.3": null, "a.4": true})
        );
    }

    #[test]
    fn multi_key_value() {
        let obj = json!({"key1": "value1", "key2": "value2"});
        assert_eq!(obj, flatten(&obj).unwrap());
    }

    #[test]
    fn nested_single_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key": "value"}});
        assert_eq!(
            flatten(&obj).unwrap(),
            json!({"key": "value", "nested_key.key": "value"}),
        );
    }

    #[test]
    fn nested_multiple_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key1": "value1", "key2": "value2"}});
        assert_eq!(
            flatten(&obj).unwrap(),
            json!({"key": "value", "nested_key.key1": "value1", "nested_key.key2": "value2"}),
        );
    }

    #[test]
    fn nested_obj_array() {
        let obj = json!({"key": ["value1", {"key": "value2"}]});
        assert_eq!(
            flatten(&obj).unwrap(),
            json!({"key.0": "value1", "key.1.key": "value2"}),
        );
    }

    #[test]
    fn complex_nested_struct() {
        let obj = json!({
            "simple_key": "simple_value",
            "key": [
                "value1",
                {"key": "value2"},
                {"nested_array": [
                    "nested1",
                    "nested2",
                    ["nested3", "nested4"]
                ]}
            ]
        });
        assert_eq!(
            flatten(&obj).unwrap(),
            json!({"simple_key": "simple_value", "key.0": "value1", "key.1.key": "value2",
                "key.2.nested_array.0": "nested1", "key.2.nested_array.1": "nested2",
                "key.2.nested_array.2.0": "nested3", "key.2.nested_array.2.1": "nested4"}),
        );
    }

    #[test]
    fn overlapping_after_flattening_array() {
        let obj = json!({"key": ["value1", "value2"], "key.0": "Oopsy"});
        let res = flatten(&obj);
        assert!(res.is_err());
        match res {
            Err(Error::KeyWillBeOverwritten(key)) => assert_eq!(key, "key.0"),
            Ok(_) => assert!(false, "This should have failed"),
            _ => assert!(false, "Wrong kind of error"),
        }
    }

    #[test]
    fn empty_array() {
        let obj = json!({"key": []});
        assert_eq!(flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn empty_object() {
        let obj = json!({"key": {}});
        assert_eq!(flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn empty_complex_object() {
        let obj = json!({"key": {"key2": {}}});
        assert_eq!(flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn nested_object_with_only_empty_array() {
        let obj = json!({"key": {"key2": []}});
        assert_eq!(flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn nested_object_with_empty_array_and_string() {
        let obj = json!({"key": {"key2": [], "key3": "a"}});
        assert_eq!(flatten(&obj).unwrap(), json!({"key.key3": "a"}));
    }

    #[test]
    fn nested_object_with_empty_object_and_string() {
        let obj = json!({"key": {"key2": {}, "key3": "a"}});
        assert_eq!(flatten(&obj).unwrap(), json!({"key.key3": "a"}));
    }

    #[test]
    fn empty_string_as_key() {
        let obj = json!({"key": {"": "a"}});
        assert_eq!(flatten(&obj).unwrap(), json!({"key.": "a"}));
    }

    #[test]
    fn empty_string_as_key_multiple_times() {
        let obj = json!({"key": {"": {"": {"": "a"}}}});
        assert_eq!(flatten(&obj).unwrap(), json!({"key...": "a"}));
    }

    #[test]
    fn first_level_must_be_an_object() {
        let integer = json!(3);
        let string = json!("");
        let boolean = json!(false);
        let null = json!(null);
        let array = json!([1, 2, 3]);

        for j in [integer, string, boolean, null, array] {
            assert!(flatten(&j).is_err());
        }
    }

    #[test]
    fn complex_array() {
        let obj = json!({"a": [1, [2, [3, 4], 5], 6]});
        assert_eq!(
            flatten(&obj).unwrap(),
            json!({"a.0": 1, "a.2": 6, "a.1.0": 2, "a.1.1.0": 3, "a.1.1.1": 4, "a.1.2": 5}),
        );
    }
}
