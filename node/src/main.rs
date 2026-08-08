
mod log;
mod state;
use std::collections::HashMap;
fn main() {
    let mut value: HashMap<String,  u16> = HashMap::new();
    let test = value.entry("angshu".to_string()).or_insert(0);
}
