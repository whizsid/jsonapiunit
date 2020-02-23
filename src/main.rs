extern crate http;
extern crate serde_hjson;

mod variables;
mod config;

use variables::Variables;
use config::Config;

fn main() {
    let config = Config::from_file();

    match config.default.headers {
        Some(headers)=>{

            for (k,v) in headers {
                println!("{}:{}",k.unwrap(),v.to_str().unwrap());

            }
        }

        None=>{}
    };

    println!("Hello, world!");
}
