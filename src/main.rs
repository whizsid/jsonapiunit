extern crate http;
extern crate serde_hjson;
extern crate glob;

mod variables;
mod config;
mod interpreter;

use config::Config;
use glob::glob;
use std::fs::read_to_string;
use interpreter::Interpreter;
use serde_hjson::Value;

fn main() {
    let config = Config::from_file();
    let mut interpreter = Interpreter::new(config.default);

    println!("Test: {}",interpreter.response_value(
        serde_hjson::from_str("\"{{ print : integer &&1==2}}\"").unwrap(),
        serde_hjson::from_str("\"aas\"").unwrap()
    ));
}
