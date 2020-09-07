use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use sha2::{Digest, Sha256};
use std::str;
pub struct ApiKey(String);

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    let result = Sha256::new().chain(key).finalize();
    println!("{}", str::from_utf8(result.as_slice()).unwrap());
    result.as_slice()
        == "3cfb19bbd129ebbaa28ea4368d7fe4329e193c4a5d108002b771104e35fcf172".as_bytes()
}

#[derive(Debug)]
pub enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-RapidAPI-Proxy-Secret").collect();
        println!("{}", keys[0]);
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => Outcome::Success(ApiKey(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}
