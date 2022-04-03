extern crate flatten_json;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use serde_json::Value;

use flatten_json::flatten;
use std::io::{self, Write};

error_chain! {
foreign_links {
        Json(::serde_json::Error);
        Io(::std::io::Error);
        Flatten(::flatten_json::Error);
    }
}

fn main() {
    let mut input = String::new();

    while let Ok(n) = io::stdin().read_line(&mut input) {
        if n > 0 {
            let v: Value = match serde_json::from_str(&input) {
                Ok(value) => value,
                Err(e) => {
                    error!("{}", &input);
                    panic!("{}", e);
                }
            };
            process_line(&v).unwrap();
            String::clear(&mut input);
        } else {
            info!("Reached end of stdin...");
            break;
        }
    }
}

fn process_line(value: &Value) -> Result<()> {
    let mut flat_value: Value = json!({});
    flatten(value, &mut flat_value, None, true, None)?;
    io::stdout().write_all(serde_json::to_string(&flat_value)?.as_bytes())?;
    io::stdout().write_all(b"\n")?;
    Ok(())
}
