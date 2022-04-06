//! Tiny Rust library for flattening JSON objects
//!
//! Given a JSON object it produces another one with all the nested objects and arrays flattened.
//! The string used to separate the concatenated keys, and the way the arrays are
//! formatted can be configured.
//!
//! ### Notes
//! - Empty arrays and objects are ignored by default, but it's configurable.
//! - The empty key `""` and the JSON `null` value can be used without problems and are preserved.
//! - Having two or more keys that end being the same after flattening the object returns an error.
//! - The JSON value passed to be flattened must be an object. The object can contain any valid JSON,
//!   though.
//!
//! ### Usage example
//!```rust
//!# use std::error::Error;
//!#
//!# fn main() -> Result<(), Box<dyn Error>> {
//!#
//! use flatten_json_object::ArrayFormatting;
//! use flatten_json_object::Flattener;
//! use serde_json::json;
//!
//! let obj = json!({
//!     "a": {
//!         "b": [1, 2.0, "c", null, true, {}, []],
//!         "" : "my_key_is_empty"
//!     },
//!     "" : "my_key_is_also_empty"
//! });
//! assert_eq!(
//!     Flattener::new()
//!         .set_key_separator(".")
//!         .set_array_formatting(ArrayFormatting::Surrounded {
//!             start: "[".to_string(),
//!             end: "]".to_string()
//!         })
//!         .set_preserve_empty_arrays(false)
//!         .set_preserve_empty_objects(false)
//!         .flatten(&obj)?,
//!     json!({
//!         "a.b[0]": 1,
//!         "a.b[1]": 2.0,
//!         "a.b[2]": "c",
//!         "a.b[3]": null,
//!         "a.b[4]": true,
//!         "a.": "my_key_is_empty",
//!         "": "my_key_is_also_empty",
//!     })
//! );
//!#
//!#     Ok(())
//!# }
//! ```

use serde_json::value::Map;
use serde_json::value::Value;

pub mod error;

/// Enum to specify how arrays are formatted.
pub enum ArrayFormatting {
    /// Uses only the key separator. Example:  `{"a": ["b"]}` => `{"a.0": "b"}`
    Plain,

    /// Does not use the key separator. Instead, the position in the array is surrounded with the
    /// provided `start` and `end` strings.
    /// Example: If `start` is `[` and `end` is `]` then `{"a": ["b"]}` => `{"a[0]": "b"}`
    Surrounded { start: String, end: String },
}

/// Basic struct of this crate. It contains the configuration. Instantiate it and use the method
/// `flatten` to flatten a JSON object.
pub struct Flattener {
    /// String used to separate the keys after the object it's flattened
    key_separator: String,
    /// Enum that indicates how arrays are formatted
    array_formatting: ArrayFormatting,
    /// If `true` all `[]` are preserved
    preserve_empty_arrays: bool,
    /// If `true` all `{}` are preserved
    preserve_empty_objects: bool,
}

impl Default for Flattener {
    fn default() -> Self {
        Self::new()
    }
}

impl Flattener {
    /// Creates a JSON object flattener with the default configuration.
    #[must_use]
    pub fn new() -> Self {
        Flattener {
            array_formatting: ArrayFormatting::Plain,
            key_separator: ".".to_string(),
            preserve_empty_arrays: false,
            preserve_empty_objects: false,
        }
    }

    /// Changes the string used to separate keys in the resulting flattened object.
    #[must_use]
    pub fn set_key_separator(mut self, key_separator: &str) -> Self {
        self.key_separator = key_separator.to_string();
        self
    }

    /// Changes the way arrays are formatted. By default the position in the array is treated as a
    /// normal key, but with this function we can change this behaviour.
    #[must_use]
    pub fn set_array_formatting(mut self, array_formatting: ArrayFormatting) -> Self {
        self.array_formatting = array_formatting;
        self
    }

    /// Changes the behaviour regarding empty arrays `[]`
    #[must_use]
    pub fn set_preserve_empty_arrays(mut self, value: bool) -> Self {
        self.preserve_empty_arrays = value;
        self
    }

    /// Changes the behaviour regarding empty objects `{}`
    #[must_use]
    pub fn set_preserve_empty_objects(mut self, value: bool) -> Self {
        self.preserve_empty_objects = value;
        self
    }

    /// Flattens the provided JSON object (`current`).
    ///
    /// It will return an error if flattening the object would make two keys to be the same,
    /// overwriting a value. It will alre return an error if the JSON value passed it's not an object.
    ///
    /// # Errors
    /// Will return `Err` if `to_flatten` it's not an object, or if flattening the object would
    /// result in two or more keys colliding.
    pub fn flatten(&self, to_flatten: &Value) -> Result<Value, error::Error> {
        let mut flat = Map::<String, Value>::new();
        self.flatten_value(to_flatten, "".to_owned(), 0, &mut flat)
            .map(|_x| Value::Object(flat))
    }

    /// Flattens the passed JSON value (`current`), whose path is `parent_key` and its 0-based
    /// depth is `depth`.  The result is stored in the JSON object `flattened`.
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
            if current.is_empty() && self.preserve_empty_objects {
                flattened.insert(parent_key, serde_json::json!({}));
            } else {
                self.flatten_object(current, &parent_key, depth, flattened)?;
            }
        } else if let Some(current) = current.as_array() {
            if current.is_empty() && self.preserve_empty_arrays {
                flattened.insert(parent_key, serde_json::json!([]));
            } else {
                self.flatten_array(current, &parent_key, depth, flattened)?;
            }
        } else {
            if flattened.contains_key(&parent_key) {
                return Err(error::Error::KeyWillBeOverwritten(parent_key));
            }
            flattened.insert(parent_key, current.clone());
        }
        Ok(())
    }

    /// Flattens the passed object (`current`), whose path is `parent_key` and its 0-based depth
    /// is `depth`.  The result is stored in the JSON object `flattened`.
    fn flatten_object(
        &self,
        current: &Map<String, Value>,
        parent_key: &str,
        depth: u32,
        flattened: &mut Map<String, Value>,
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

    /// Flattens the passed array (`current`), whose path is `parent_key` and its 0-based depth
    /// is `depth`.  The result is stored in the JSON object `flattened`.
    fn flatten_array(
        &self,
        current: &[Value],
        parent_key: &str,
        depth: u32,
        flattened: &mut Map<String, Value>,
    ) -> Result<(), error::Error> {
        for (i, obj) in current.iter().enumerate() {
            let parent_key = if depth > 0 {
                match self.array_formatting {
                    ArrayFormatting::Plain => format!("{}{}{}", parent_key, self.key_separator, i),
                    ArrayFormatting::Surrounded { ref start, ref end } => {
                        format!("{}{}{}{}", parent_key, start, i, end)
                    }
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
    use super::ArrayFormatting;
    use super::Flattener;
    use crate::error::Error;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("")]
    #[case(".")]
    #[case("-->")]
    fn object_with_plain_values(#[case] key_separator: &str) {
        let obj = json!({"int": 1, "float": 2.0, "str": "a", "bool": true, "null": null});
        assert_eq!(
            obj,
            Flattener::new()
                .set_key_separator(key_separator)
                .flatten(&obj)
                .unwrap()
        );
    }

    /// Ensures that when using `ArrayFormatting::Plain` both arrays and objects are formatted
    /// properly.
    #[rstest]
    #[case("")]
    #[case(".")]
    #[case("aaa")]
    fn array_formatting_plain(#[case] key_separator: &str) {
        let obj = json!({"s": {"a": [1, 2.0, "b", null, true]}});
        assert_eq!(
            Flattener::new()
                .set_key_separator(key_separator)
                .flatten(&obj)
                .unwrap(),
            json!({
                format!("s{k}a{k}0", k = key_separator): 1,
                format!("s{k}a{k}1", k = key_separator): 2.0,
                format!("s{k}a{k}2", k = key_separator): "b",
                format!("s{k}a{k}3", k = key_separator): null,
                format!("s{k}a{k}4", k = key_separator): true,
            })
        );
    }

    /// Ensures that when using `ArrayFormatting::Surrounded` both the array start and end
    /// decorations and the key separator work as expected.
    #[rstest]
    fn array_formatting_surrouded(
        #[values("", ".", "-->")] key_separator: &str,
        #[values("", "[", "{{")] array_fmt_start: &str,
        #[values("", "]", "}}")] array_fmt_end: &str,
    ) {
        let obj = json!({"s": {"a": [1, 2.0, "b", null, true]}});
        assert_eq!(
            Flattener::new()
                .set_key_separator(key_separator)
                .set_array_formatting(ArrayFormatting::Surrounded {
                    start: array_fmt_start.to_string(),
                    end: array_fmt_end.to_string()
                })
                .flatten(&obj)
                .unwrap(),
            json!({
                format!("s{}a{}0{}", key_separator, array_fmt_start, array_fmt_end): 1,
                format!("s{}a{}1{}", key_separator, array_fmt_start, array_fmt_end): 2.0,
                format!("s{}a{}2{}", key_separator, array_fmt_start, array_fmt_end): "b",
                format!("s{}a{}3{}", key_separator, array_fmt_start, array_fmt_end): null,
                format!("s{}a{}4{}", key_separator, array_fmt_start, array_fmt_end): true,
            })
        );
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
            Ok(_) => panic!("This should have failed"),
            _ => panic!("Wrong kind of error"),
        }
    }

    /// Ensure that empty arrays are not present in the result
    #[test]
    fn empty_array() {
        let obj = json!({"key": []});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    /// Ensure that empty objects are not present in the result
    #[test]
    fn empty_object() {
        let obj = json!({"key": {}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    /// Ensure that if all the end values of the JSON object are either `[]` or `{}` the flattened
    /// resulting object it's empty.
    #[test]
    fn empty_complex_object() {
        let obj = json!({"key": {"key2": {}, "key3": [[], {}, {"k": {}, "q": []}]}});
        assert_eq!(Flattener::new().flatten(&obj).unwrap(), json!({}));
    }

    /// Ensure that empty arrays are preserved if so configured
    #[test]
    fn empty_array_preserved() {
        let obj = json!({"key": [], "a": {}});
        assert_eq!(
            Flattener::new()
                .set_preserve_empty_arrays(true)
                .flatten(&obj)
                .unwrap(),
            json!({"key": []})
        );
    }

    /// Ensure that empty objects are preserved if so configured
    #[test]
    fn empty_object_preserved() {
        let obj = json!({"key": {}, "a": []});
        assert_eq!(
            Flattener::new()
                .set_preserve_empty_objects(true)
                .flatten(&obj)
                .unwrap(),
            json!({"key": {}})
        );
    }

    /// Ensure that all the end values of the JSON object that are either `[]` or `{}` are preserved
    /// if so configured
    #[test]
    fn empty_objects_and_arrays_preserved() {
        let obj = json!({
            "key": {
                "key2": {},
                "key3": [[], {}, {"k": {}, "q": []}]
            }
        });
        assert_eq!(
            Flattener::new()
                .set_preserve_empty_arrays(true)
                .set_preserve_empty_objects(true)
                .flatten(&obj)
                .unwrap(),
            json!({
                "key.key2": {},
                "key.key3.0": [],
                "key.key3.1": {},
                "key.key3.2.k": {},
                "key.key3.2.q": [],
            })
        );
    }

    #[test]
    fn nested_object_with_empty_array_and_string() {
        let obj = json!({"key": {"key2": [], "key3": "a"}});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"key.key3": "a"})
        );
    }

    #[test]
    fn nested_object_with_empty_object_and_string() {
        let obj = json!({"key": {"key2": {}, "key3": "a"}});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"key.key3": "a"})
        );
    }

    #[test]
    fn empty_string_as_key() {
        let obj = json!({"key": {"": "a"}});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"key.": "a"})
        );
    }

    #[test]
    fn empty_string_as_key_multiple_times() {
        let obj = json!({"key": {"": {"": {"": "a"}}}});
        assert_eq!(
            Flattener::new().flatten(&obj).unwrap(),
            json!({"key...": "a"})
        );
    }

    /// Flattening only makes sense for objects. Passing something else must return an informative
    /// error.
    #[test]
    fn first_level_must_be_an_object() {
        let integer = json!(3);
        let string = json!("");
        let boolean = json!(false);
        let null = json!(null);
        let array = json!([1, 2, 3]);

        for j in [integer, string, boolean, null, array] {
            let res = Flattener::new().flatten(&j);
            match res {
                Err(Error::FirstLevelMustBeAnObject) => return, // Good
                Ok(_) => panic!("This should have failed"),
                _ => panic!("Wrong kind of error"),
            }
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
