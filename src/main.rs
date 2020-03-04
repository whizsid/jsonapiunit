extern crate http;
extern crate glob;
extern crate url;
extern crate serde_json;

mod variables;
mod config;
mod interpreter;
mod test_case;

use colored::*;
use config::Config;
use glob::glob;
use http::Method;
use http::HeaderMap;
use http::HeaderValue;
use hyper::Client;
use hyper::body::Body;
use hyper::body::to_bytes;
use hyper::Request;
use std::fs::read_to_string;
use interpreter::Interpreter;
use serde_json::from_reader;
use serde_json::from_str;
use serde_json::Value;
use json_comments::StripComments;
use test_case::TestCase;
use url::Url;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() ->  Result<()> {
    let config = Config::from_file();

    let mut interpreter = Interpreter::new(config.pre_variables);

    let base_url = match config.base_url {
        Some(base)=>{
            base
        }
        None=>{
            String::from("")
        }
    };

    let default_method = match config.default.method {
        Some(method)=>{
            method
        }
        None=>{
            Method::GET
        }
    };

    let default_headers = match config.default.headers {
        Some(headers)=>{headers}
        None=>{HeaderMap::new()}
    };

    let client = Client::new();
    let mut failed = false;

    for path in glob(&config.files).unwrap() {
        let path = path.unwrap();
        let name = String::from(path.to_str().unwrap());

        println!("{} : {}","STARTED".cyan(),name);

        let test_case_str = read_to_string(path).unwrap();

        // Stripping comments in the json file.
        let test_case_reader = StripComments::new(test_case_str.as_bytes());
        let test_case_json:Value = from_reader(test_case_reader).unwrap();
        let test_case = TestCase::new(name.clone(),test_case_json.as_object().unwrap().to_owned());

        let req_url = match Url::parse(&base_url) {
            Ok(url)=>{
                url.join(&test_case.url()).unwrap()
            }
            Err(_)=>{
                Url::parse(&test_case.url()).unwrap()
            }
        };

        let body = Value::Object(interpreter.parse_request_body(test_case.request().body().unwrap()));

        let method = match test_case.method() {
            Some(method)=>{
                method
            }
            None =>{
                default_method.clone()
            }
        };

        let mut headers = match test_case.request().headers() {
            Some(headers) => {headers}
            None=>{HeaderMap::new()}
        };

        // Appending default headers
        let mut default_headers_iter = default_headers.iter();

        while let Some((k,v)) = default_headers_iter.next() {
            headers.append(k, v.to_owned());
        }

        let mut request_builder = Request::builder();

        request_builder = request_builder.method(method);
        request_builder = request_builder.uri(req_url.as_str());

        let mut headers_iter = headers.iter();

        // Formatting all headers
        while let Some((k,v)) = headers_iter.next(){
            let value = v.to_str().unwrap();
            let value = interpreter.request_header(value);
            request_builder = request_builder.header(k, HeaderValue::from_str(&value).unwrap());
        }

        let request = request_builder.body(Body::from(format!("{}",body))).unwrap();

        let response = client.request(request).await?;

        let status_matched = match test_case.response().status() {
            Some(status)=>{
                status == (response.status().as_u16() as i64)
            }
            None=>{
                true
            }
        };

        let response_bytes = to_bytes( response.into_body()).await?;
        let response_string = String::from_utf8(response_bytes.to_vec()).unwrap();
    
        if let Some(test_response) = test_case.response().body() {

            let response_json:std::result::Result<Value, serde_json::Error> = from_str(&response_string);

            match response_json {
                Ok(response_value)=>{
                    
                    let passed = interpreter.parse_response_body(test_response, response_value.as_object().unwrap().to_owned());

                    if !status_matched {
                        failed = true;

                        println!("{} : Name: {}, Reason: Status code not matched.","FAILED TEST CASE".red(),test_case.name());
                    } else if ! passed  {
                        failed = true;

                        println!("{} : Name: {}, Reason: Some assertion(s) failed.","FAILED TEST CASE".red(),test_case.name());
                    } else {
                        println!("{} : Name: {}","PASSED TEST CASE".green(),test_case.name());
                    }
                }
                Err(_)=>{
                    failed = true;
                    println!("{} : Response is not a JSON.","FAILED TEST CASE".red());
                }
            }

        }
    };

    if failed {
        Err(Box::from("Some test cases failed."))
    } else {
        Ok(())
    }

}
