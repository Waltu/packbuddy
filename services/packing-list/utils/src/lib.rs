use lambda_http::{http::StatusCode, Body, Response};

pub fn response(status_code: StatusCode, body: String) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}
