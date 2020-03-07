extern crate glob;
extern crate http;
extern crate serde_json;
extern crate url;

mod config;
mod interpreter;
mod test_case;
mod variables;

use colored::*;
use config::Config;
use glob::glob;
use http::{HeaderMap, HeaderValue, Method};
use hyper::body::{to_bytes, Body};
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::{Client as HyperClient, Request};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use interpreter::Interpreter;
use json_comments::StripComments;
use serde_json::{from_reader, from_str, Value};
use std::fs::read_to_string;
use test_case::TestCase;
use typed_headers::Credentials;
use url::Url;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

enum Client {
    Proxy(HyperClient<ProxyConnector<HttpConnector>>),
    Http(HyperClient<HttpConnector>),
}

impl Client {
    pub fn request(&self, req: Request<Body>) -> ResponseFuture {
        match self {
            Client::Proxy(client) => client.request(req),
            Client::Http(client) => client.request(req),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file();

    let mut interpreter = Interpreter::new(config.pre_variables);

    let base_url = match config.base_url {
        Some(base) => base,
        None => String::from(""),
    };

    let default_method = match config.default.method {
        Some(method) => method,
        None => Method::GET,
    };

    let default_headers = match config.default.headers {
        Some(headers) => headers,
        None => HeaderMap::new(),
    };

    let client = match config.proxy {
        Some(proxy_config) => {
            let uri_str = &proxy_config.uri;
            let proxy_uri = uri_str.parse().unwrap();
            let mut proxy = Proxy::new(Intercept::All, proxy_uri);

            if let Some(username) = proxy_config.username {
                if let Some(password) = proxy_config.password {
                    proxy.set_authorization(Credentials::basic(&username, &password).unwrap());
                } else {
                    panic!("Please provide a password for proxy.")
                }
            }

            let connector = HttpConnector::new();
            let proxy_connector = ProxyConnector::from_proxy(connector, proxy).unwrap();

            let client = HyperClient::builder().build(proxy_connector);

            Client::Proxy(client)
        }
        None => Client::Http(HyperClient::new()),
    };

    let mut failed = false;

    for path in glob(&config.files).unwrap() {
        let path = path.unwrap();
        let name = String::from(path.to_str().unwrap());

        println!("{} : {}", "STARTED".cyan(), name);

        let test_case_str = read_to_string(path).unwrap();

        // Stripping comments in the json file.
        let test_case_reader = StripComments::new(test_case_str.as_bytes());
        let test_case_json: Value = from_reader(test_case_reader).unwrap();
        let test_case = TestCase::new(name.clone(), test_case_json.as_object().unwrap().to_owned());

        let req_url = match Url::parse(&base_url) {
            Ok(url) => url.join(&test_case.url()).unwrap(),
            Err(_) => Url::parse(&test_case.url()).unwrap(),
        };

        let body = Value::Object(
            interpreter.parse_request_body(test_case.request().body().unwrap_or_default()),
        );

        let method = match test_case.method() {
            Some(method) => method,
            None => default_method.clone(),
        };

        let mut headers = match test_case.request().headers() {
            Some(headers) => headers,
            None => HeaderMap::new(),
        };

        // Appending default headers
        let mut default_headers_iter = default_headers.iter();

        while let Some((k, v)) = default_headers_iter.next() {
            headers.append(k, v.to_owned());
        }

        let mut request_builder = Request::builder();

        request_builder = request_builder.method(method);
        request_builder = request_builder.uri(req_url.as_str());

        let mut headers_iter = headers.iter();

        // Formatting all headers
        while let Some((k, v)) = headers_iter.next() {
            let value = v.to_str().unwrap();
            let value = interpreter.request_header(value);
            request_builder = request_builder.header(k, HeaderValue::from_str(&value).unwrap());
        }

        let request = request_builder
            .body(Body::from(format!("{}", body)))
            .unwrap();

        let response = client.request(request).await?;

        // Clearing interpreter assertion count
        interpreter.reset_counter();

        let mut passed = match test_case.response().status() {
            Some(test_status) => {
                let res_status = response.status().as_u16() as i64;
                let status_matched = test_status == res_status;

                interpreter.log_assertion(
                    status_matched,
                    &format!("HTTP_STATUS_{}", test_status),
                    &format!("HTTP_STATUS_{}", res_status),
                );

                status_matched
            }
            None => true,
        };

        let response_bytes = to_bytes(response.into_body()).await?;
        let response_string = String::from_utf8(response_bytes.to_vec()).unwrap();
        if let Some(test_response) = test_case.response().body() {
            let response_json: std::result::Result<Value, serde_json::Error> =
                from_str(&response_string);

            match response_json {
                Ok(response_value) => {
                    interpreter.log_assertion(true, "FORMAT_JSON", "FORMAT_JSON");
                    let res_body_passed = interpreter.parse_response_body(
                        test_response,
                        response_value.as_object().unwrap().to_owned(),
                    );

                    if !res_body_passed {
                        passed = false;
                    }
                }
                Err(_) => {
                    interpreter.log_assertion(false, "FORMAT_JSON", "FORMAT_NOT_JSON");
                    passed = false;
                }
            }
        }

        println!(
            "{} : Name: {}, Assertions: {}, Fails: {}, TotAssertions: {}, TotFails: {}",
            if passed {
                "PASSED TEST CASE".green()
            } else {
                "FAILED TEST CASE".red()
            },
            test_case.name(),
            interpreter.cur_asserts,
            interpreter.cur_fails,
            interpreter.tot_asserts,
            interpreter.tot_fails
        );

        if !passed {
            failed = true;
        }
    }

    if failed {
        Err(Box::from("Some test cases failed."))
    } else {
        Ok(())
    }
}
