use http::header::{HeaderName, HeaderValue};
use http::{HeaderMap, Method};
use serde_json::{Map, Value};

pub struct TestCase {
    json: Map<String, Value>,
    name: String,
}

impl TestCase {
    pub fn new(name: String, json: Map<String, Value>) -> TestCase {
        TestCase { name, json }
    }

    pub fn url(&self) -> String {
        String::from(self.json.get("url").unwrap().as_str().unwrap())
    }

    pub fn method(&self) -> Option<Method> {
        match self.json.get("method") {
            Some(method_val) => {
                Some(Method::from_bytes(method_val.as_str().unwrap().as_bytes()).unwrap())
            }
            None => None,
        }
    }

    pub fn request(&self) -> Request {
        match self.json.get("request") {
            Some(request) => Request::new(request.as_object().unwrap().to_owned()),
            None => panic!("Not containing request"),
        }
    }

    pub fn response(&self) -> Response {
        match self.json.get("response") {
            Some(response) => Response::new(response.as_object().unwrap().to_owned()),
            None => panic!("Not containing response"),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct Request {
    json: Map<String, Value>,
}

impl Request {
    pub fn new(json: Map<String, Value>) -> Request {
        Request { json }
    }

    pub fn body(&self) -> Option<Map<String, Value>> {
        match self.json.get("body") {
            Some(body) => Some(body.as_object().unwrap().to_owned()),
            None => None,
        }
    }

    pub fn headers(&self) -> Option<HeaderMap> {
        match self.json.get("headers") {
            Some(headers) => {
                let headers_obj = headers.as_object().unwrap();

                let mut headers_map = HeaderMap::new();

                for (key, val) in headers_obj {
                    headers_map.append(
                        HeaderName::from_bytes(String::from(key).into_bytes().as_ref()).unwrap(),
                        HeaderValue::from_bytes(&val.as_str().unwrap().as_bytes()).unwrap(),
                    );
                }

                Some(headers_map)
            }
            None => None,
        }
    }
}

pub struct Response {
    json: Map<String, Value>,
}

impl Response {
    pub fn new(json: Map<String, Value>) -> Response {
        Response { json }
    }

    pub fn body(&self) -> Option<Map<String, Value>> {
        match self.json.get("body") {
            Some(body) => Some(body.as_object().unwrap().to_owned()),
            None => None,
        }
    }

    pub fn status(&self) -> Option<i64> {
        match self.json.get("status") {
            Some(status) => Some(status.as_i64().unwrap()),
            None => None,
        }
    }
}
