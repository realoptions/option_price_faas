use hex_literal::hex;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use sha2::{Digest, Sha256};
use std::str;
pub struct ApiKey(String);

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    let result = Sha256::new().chain(key).finalize();
    result.as_slice() == hex!("9bdd78714a3f3076ffacce2672c546f2b38635db97de0c72a8b5aa248df4dbcd")
}

#[derive(Debug)]
pub enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-RapidAPI-Proxy-Secret").collect();
        match keys.len() {
            0 => request::Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => request::Outcome::Success(ApiKey(keys[0].to_string())),
            1 => request::Outcome::Error((Status::Forbidden, ApiKeyError::Invalid)),
            _ => request::Outcome::Error((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}
