use lambda_http::{Body, IntoResponse, Response};
use serde_json::json;
pub fn build_response(code: u16, body: &str) -> impl IntoResponse {
    Response::builder()
        .status(code)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Credentials", "true")
        .body::<Body>(body.into())
        .unwrap()
}

pub fn construct_error(e_message: &str) -> String {
    json!({ "err": e_message }).to_string()
}
