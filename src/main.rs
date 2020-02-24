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

fn main() {
    let config = Config::from_file();
    let mut interpreter = Interpreter::new(config.default);



    println!("Test: {}, Test2:{}",interpreter.request_value("{{ > print : integer }}"),interpreter.request_value("{{print}}"));
}
