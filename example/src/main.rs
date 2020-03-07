#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket_contrib::json::{Json, JsonValue};

const AUTH_KEY: &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

#[derive(Serialize, Deserialize)]
pub struct LoginRequestBody {
    pub username: String,
    pub password: String,
}

#[post("/user/login", data = "<body>")]
fn login(body: Json<LoginRequestBody>) -> JsonValue {
    if body.username != "murali" {
        return json!({
            "status": false,
            "message": "Invalid username"
        });
    }

    json!({
        "status": true,
        "name":"Muththaiya Muralitharan",
        "username": "murali",
        "token": AUTH_KEY,
        "limit": 1200,
        "usage": 120
    })
}

struct ApiKey(String);

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    key == format!("Bearer {}", AUTH_KEY)
}

#[derive(Debug)]
enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => Outcome::Success(ApiKey(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}

#[post("/user/details")]
fn user_details(_api_key: ApiKey) -> JsonValue {
    json!({
        "status": true,
        "name":"Muththaiya Muralitharan",
        "username": "murali",
        "allowance": 12000,
        "totalExpences": 120
    })
}

#[get("/categories")]
fn categories(_api_key: ApiKey) -> JsonValue {
    json!({
        "status": true,
        "categories": [
            {
                "id": 1,
                "name": "Category 1",
                "minPrice": 1234.9
            },
            {
                "id": 2,
                "name": "Category 2",
                "minPrice": 12.23
            },
            {
                "id": 3,
                "name": "Category 3",
                "minPrice": 124.23
            }
        ]
    })
}

fn main() {
    rocket::ignite()
        .mount("/api", routes![login, user_details, categories])
        .launch();
}
