//! Tiny Rust library for flattening JSON objects

use serde_json::value::Map;
use serde_json::value::Value;

mod error;

pub enum ArrayFormatting {
    /// Uses only the key separator. Example:  {"a": ["b"]} => {"a.0": "b"}
    Plain,

    /// Does not use the key sepparator. Instead, the position in the array is surrounded with the provided `start` and `end` strings.
    /// Example: If `start` is "[" and `end` is "]" then {"a": ["b"]} => {"a[0]": "b"}
    Surrounded { start: String, end: String },
}

pub struct Flattener {
    key_separator: String,
    array_formatting: ArrayFormatting,
}

impl Flattener {
    /// Creates a JSON object flattener with the dafault configuration.
    pub fn new() -> Self {
        Flattener {
            array_formatting: ArrayFormatting::Plain,
            key_separator: ".".to_string(),
        }
    }

    /// Changes the string used to separate keys in the resulting flattened object.
    pub fn set_key_separator(mut self, key_separator: &str) -> Self {
        self.key_separator = key_separator.to_string();
        self
    }

    /// Changes the way arrays are formatted. By default the position in the array is treated as a
    /// normal key, but with this fuction we can change this behaviour.
    pub fn set_array_formatting(mut self, array_formatting: ArrayFormatting) -> Self {
        self.array_formatting = array_formatting;
        self
    }

    /// Flattens the provided JSON object (`current`).
    ///
    /// It will return an error if flattening the object would make two keys to be the same,
    /// overwriting a value. It will alre return an error if the JSON value passed it's not an object.
    pub fn flatten(&self, to_flatten: &Value) -> Result<Value, error::Error> {
        let mut flat = Map::<String, Value>::new();
        self.flatten_value(to_flatten, "".to_owned(), 0, &mut flat)
            .map(|_x| Value::Object(flat))
    }

    /// Flattens the passed JSON value (`current`), whose path is `parent_key` and its 0-based depth is `depth`.
    /// The result is stored in the JSON object `flattened`.
    fn flatten_value(
        &self,
        current: &Value,
        parent_key: String,
        depth: u32,
        flattened: &mut Map<String, Value>,
    ) -> Result<(), error::Error> {
        if depth == 0 && !current.is_object() {
            return Err(error::Error::FirstLevelMustBeAnObject);
        }

        if let Some(current) = current.as_object() {
            self.flatten_object(current, parent_key, depth, flattened)?;
        } else if let Some(current) = current.as_array() {
            self.flatten_array(current, parent_key, depth, flattened)?;
        } else {
            if flattened.contains_key(&parent_key) {
                return Err(error::Error::KeyWillBeOverwritten(parent_key));
            }
            flattened.insert(parent_key, current.clone());
        }
        Ok(())
    }

    /// Flattens the passed object (`current`), whose path is `parent_key` and its 0-based depth is `depth`.
    /// The result is stored in the JSON object `flattened`.
    fn flatten_object(
        &self,
        current: &Map<String, Value>,
        parent_key: String,
        depth: u32,
        flattened: &mut Map<String, Value>, // Were we accumulate the resulting flattened json
    ) -> Result<(), error::Error> {
        for (k, v) in current.iter() {
            let parent_key = if depth > 0 {
                format!("{}{}{}", parent_key, self.key_separator, k)
            } else {
                k.to_string()
            };
            self.flatten_value(v, parent_key, depth + 1, flattened)?;
        }
        Ok(())
    }

    /// Flattens the passed array (`current`), whose path is `parent_key` and its 0-based depth is `depth`.
    /// The result is stored in the JSON object `flattened`.
    fn flatten_array(
        &self,
        current: &[Value],
        parent_key: String,
        depth: u32,
        flattened: &mut Map<String, Value>, // Were we accumulate the resulting flattened json
    ) -> Result<(), error::Error> {
        for (i, obj) in current.iter().enumerate() {
            let parent_key = if depth > 0 {
                match self.array_formatting {
                    ArrayFormatting::Plain => format!("{}{}{}", parent_key, self.key_separator, i),
                    ArrayFormatting::Surrounded { ref start, ref end } => format!("{}{}{}{}", parent_key, start, i, end),
                }
            } else {
                format!("{}", i)
            };
            self.flatten_value(obj, parent_key, depth + 1, flattened)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Flattener;
    use crate::error::Error;
    use serde_json::json;

    #[test]
    fn single_key_value() {
        let obj = json!({"key": "value"});
        assert_eq!(obj, Flattener::new().flatten(&obj).unwrap());
    }

    #[test]
    fn object_with_plain_values() {
        let obj = json!({"int": 1, "float": 2.0, "str": "a", "bool": true, "null": null});
        assert_eq!(obj, Flattener::new().flatten(&obj).unwrap());
    }

    #[test]
    fn array_with_plain_values() {
        let obj = json!({"a": [1, 2.0, "b", null, true]});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"a.0": 1, "a.1": 2.0, "a.2": "b", "a.3": null, "a.4": true})
        );
    }

    #[test]
    fn multi_key_value() {
        let obj = json!({"key1": "value1", "key2": "value2"});
        assert_eq!(obj, Flattener::new().flatten(&obj).unwrap());
    }

    #[test]
    fn nested_single_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key": "value"}});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"key": "value", "nested_key.key": "value"}),
        );
    }

    #[test]
    fn nested_multiple_key_value() {
        let obj = json!({"key": "value", "nested_key": {"key1": "value1", "key2": "value2"}});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"key": "value", "nested_key.key1": "value1", "nested_key.key2": "value2"}),
        );
    }

    #[test]
    fn nested_obj_array() {
        let obj = json!({"key": ["value1", {"key": "value2"}]});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
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
            Flattener::new().flatten(&obj).unwrap(),
            json!({"simple_key": "simple_value", "key.0": "value1", "key.1.key": "value2",
                "key.2.nested_array.0": "nested1", "key.2.nested_array.1": "nested2",
                "key.2.nested_array.2.0": "nested3", "key.2.nested_array.2.1": "nested4"}),
        );
    }

    #[test]
    fn overlapping_after_flattening_array() {
        let obj = json!({"key": ["value1", "value2"], "key.0": "Oopsy"});
        let res = Flattener::new().flatten(&obj);
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
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn empty_object() {
        let obj = json!({"key": {}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn empty_complex_object() {
        let obj = json!({"key": {"key2": {}}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn nested_object_with_only_empty_array() {
        let obj = json!({"key": {"key2": []}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    #[test]
    fn nested_object_with_empty_array_and_string() {
        let obj = json!({"key": {"key2": [], "key3": "a"}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({"key.key3": "a"}));
    }

    #[test]
    fn nested_object_with_empty_object_and_string() {
        let obj = json!({"key": {"key2": {}, "key3": "a"}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({"key.key3": "a"}));
    }

    #[test]
    fn empty_string_as_key() {
        let obj = json!({"key": {"": "a"}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({"key.": "a"}));
    }

    #[test]
    fn empty_string_as_key_multiple_times() {
        let obj = json!({"key": {"": {"": {"": "a"}}}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({"key...": "a"}));
    }

    #[test]
    fn first_level_must_be_an_object() {
        let integer = json!(3);
        let string = json!("");
        let boolean = json!(false);
        let null = json!(null);
        let array = json!([1, 2, 3]);

        for j in [integer, string, boolean, null, array] {
            assert!(Flattener::new().flatten(&j).is_err());
        }
    }

    #[test]
    fn complex_array() {
        let obj = json!({"a": [1, [2, [3, 4], 5], 6]});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"a.0": 1, "a.2": 6, "a.1.0": 2, "a.1.1.0": 3, "a.1.1.1": 4, "a.1.2": 5}),
        );
    }
}
