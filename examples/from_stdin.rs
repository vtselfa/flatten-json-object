use flatten_json_object::flatten;
use log::{error, info};
use serde_json::Value;
use std::io::{self, Write};

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

fn process_line(value: &Value) -> Result<(), anyhow::Error> {
    let flat_value = flatten(value)?;
    io::stdout().write_all(serde_json::to_string(&flat_value)?.as_bytes())?;
    io::stdout().write_all(b"\n")?;
    Ok(())
}
