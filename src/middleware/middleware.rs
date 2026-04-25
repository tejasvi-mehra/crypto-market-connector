use axum::{body::Body, http::Request, middleware::Next, response::Response};

use chrono::Utc;
use std::time::Instant;

pub async fn request_log(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let start_time = Utc::now().format("%Y-%m-%d %H:%M:%S");
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = req.uri().path().to_string();
    let headers = format!("{:?}", req.headers());

    log::info!(
        "{} {} {} - Headers: {}, started at: {}",
        method,
        uri,
        path,
        headers,
        start_time
    );

    let response = next.run(req).await;

    let duration = start.elapsed();
    let end_time = Utc::now().format("%Y-%m-%d %H:%M:%S");
    log::info!(
        "{} {} {} - Headers: {}, completed at: {}, time taken: {} ms",
        method,
        uri,
        path,
        headers,
        end_time,
        duration.as_millis()
    );

    response
}
