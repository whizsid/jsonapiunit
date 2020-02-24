use super::variables::Variables;
use super::variables::VariableType;

use http::HeaderMap;
use http::header::HeaderName;
use http::header::HeaderValue;
use http::Method;
use std::fs::read_to_string;
use serde_hjson::{Value,Map};

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
                let config_json: Map<String, Value> = serde_hjson::from_str(&file_str).unwrap();
                let mut config = Config::new();

                config.base_url = match config_json.get("baseUrl") {
                    Some(base_url)=>{
                         Some(base_url.to_string())
                    }
                    None=>{None}
                };

                match config_json.get("files") {
                    Some(file_pattern)=>{
                        config.files = file_pattern.to_string();
                    }
                    None=>{}
                };

                config.pre_variables = match config_json.get("preVariables") {
                    Some(pre_variables)=>{
                        let mut variables = Variables::new();

                        
                        for (k,v) in pre_variables.as_object().unwrap() {
                            variables.add(k,&v.to_string(),VariableType::Any);
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
                                let method_str = method.to_string().to_uppercase();

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
                                                .to_string()
                                                .into_bytes()
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