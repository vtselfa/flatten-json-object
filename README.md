[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/vtselfa/flatten-json-object/blob/master/LICENSE.md)
## Tiny Rust library for flattening JSON objects

Given a JSON object it produces another one with all the nested objects and arrays flattened.
The string used to separate the concatenated keys, and the way the arrays are
formatted can be configured.

### Notes
- Empty arrays and objects are ignored. The empty key `""` and the JSON `null` value can
  be used without problems and are preserved.
- Having two or more keys that end being the same after flattening the object returns an error.
- The JSON value passed to be flattened must be an object. It can contain any valid JSON,
  though.

### Usage

*In your Cargo.toml*

```
[dependencies]
flatten-json-object = "0.1.0"
```

### Example
```rust
use flatten_json_object::ArrayFormatting;
use flatten_json_object::Flattener;
use serde_json::json;

let obj = json!({
    "a": {
        "b": [1, 2.0, "c", null, true, {}, []],
        "" : "my_key_is_empty"
    },
    "" : "my_key_is_also_empty"
});
assert_eq!(
    Flattener::new()
        .set_key_separator(".")
        .set_array_formatting(ArrayFormatting::Surrounded {
            start: "[".to_string(),
            end: "]".to_string()
        })
        .flatten(&obj)?,
    json!({
        "a.b[0]": 1,
        "a.b[1]": 2.0,
        "a.b[2]": "c",
        "a.b[3]": null,
        "a.b[4]": true,
        "a.": "my_key_is_empty",
        "": "my_key_is_also_empty",
    })
);
```

A trivial example that reads `JSON` from `stdin` and outputs the converted flat `JSON` to `stdout`
can be found in [examples/from_stdin.rs](https://github.com/vtselfa/flatten-json-object/blob/master/examples/from_stdin.rs).
To run it execute `cargo run --example from_stdin`.

