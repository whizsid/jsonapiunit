use super::variables::Variables;

use http::HeaderMap;
use http::header::HeaderName;
use http::header::HeaderValue;
use http::Method;
use std::fs::read_to_string;
use serde_json::{Value,Map};
use serde_json::from_reader;
use json_comments::StripComments;

pub struct DefaultConfig {
    pub method: Option<Method>,
    pub headers: Option<HeaderMap>
}

pub struct Config {
    pub base_url: Option<String>,
    pub pre_variables: Option< Variables>,
    pub default: DefaultConfig,
    pub files: String
}

impl Config {
    /// # Creating new empty configuration
    /// 
    /// All fields are empty except files.
    pub fn new()->Config {
        Config {
            base_url: None,
            pre_variables: None,
            default: DefaultConfig {
                method: None,
                headers: None
            },
            files: String::from("apiTest/*.jsonc")
        }
    }

    /// # Loading configurations from file
    /// 
    /// This function will read the `apiunit.jsonc` file and
    /// creating the new config struct
    pub fn from_file() -> Config {
        let file = read_to_string("apiunit.jsonc");

        match file {
            Ok(file_str)=>{
                let file_without_comment = StripComments::new(file_str.as_bytes());
                let config_val:Value = from_reader(file_without_comment).unwrap();
                let config_json: &Map<String, Value> = config_val.as_object().unwrap();

                let mut config = Config::new();

                config.base_url = match config_json.get("baseUrl") {
                    Some(base_url)=>{
                         Some(String::from(base_url.as_str().unwrap()))
                    }
                    None=>{None}
                };

                match config_json.get("files") {
                    Some(file_pattern)=>{
                        config.files = String::from(file_pattern.as_str().unwrap());
                    }
                    None=>{}
                };

                config.pre_variables = match config_json.get("preVariables") {
                    Some(pre_variables)=>{
                        let mut variables = Variables::new();

                        
                        for (k,v) in pre_variables.as_object().unwrap() {
                            variables.add(k,v.to_owned());
                        }

                        Some(variables)
                    }
                    None =>{None}
                };

                config.default = match config_json.get("default") {
                    Some (default)=>{
                        let default_obj = default.as_object().unwrap();

                        let mut default_config = DefaultConfig {
                            method: None,
                            headers: None
                        };

                        default_config.method = match default_obj.get("method") {
                            Some(method)=>{
                                let method_str = method.as_str().unwrap().to_uppercase();

                                Some(Method::from_bytes(& method_str.into_bytes()).unwrap())
                            }
                            None=>{None}
                        };

                        default_config.headers = match default_obj.get("headers") {
                            Some(headers) =>{
                                let headers_obj = headers.as_object().unwrap();

                                let mut headers_map = HeaderMap::new();

                                for (key,val) in headers_obj {
                                    headers_map.append(
                                        HeaderName::from_bytes(
                                            String::from(key)
                                                .into_bytes()
                                                .as_ref()
                                        ).unwrap(), 
                                        HeaderValue::from_bytes( 
                                            &val
                                                .as_str()
                                                .unwrap()
                                                .as_bytes()
                                        ).unwrap()
                                    );
                                }

                                Some(headers_map)
                            }
                            None =>{None}
                        };

                        default_config
                    }
                    None =>{ DefaultConfig {
                        method: None,
                        headers: None
                    }}
                };

                config
            }
            Err(_)=>{
                Config::new()
            }
        }
    }
}