//! Validates a settings document given as JSON on standard input and prints the field errors
//! the API would return, or the document it would store.
//!
//! ```sh
//! echo '{"version":1,"country":"JP","guest":{"name":"カフェの無料ワイファイ","mode":"open_to_all","enabled":true},"management":{"name":"Enkrnett-3F2A"}}' \
//!   | cargo run -q -p enkrnett-model --example validate_settings
//! ```

use std::io::Read;

use enkrnett_model::settings::{Settings, validate};

fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("read stdin");
    let settings: Settings = match serde_json::from_str(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("not a settings document: {e}");
            std::process::exit(2);
        }
    };
    let errors = validate(&settings);
    if errors.is_empty() {
        println!("{}", serde_json::to_string_pretty(&settings).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&errors).unwrap());
        std::process::exit(1);
    }
}
