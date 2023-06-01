mod jsight;

use std::convert::Infallible;
use std::net::SocketAddr;
use chrono::Local;
use http_body_util::{Full, BodyExt};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Method};
use tokio::net::TcpListener;

// Server HTTP-request processing function
async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {

    let api_spec_path = "/opt/app/src/my-api-spec.jst";
    let method = get_current_method(&req);
    let uri = req.uri().to_string();
    let request_headers = get_http_headers(req.headers());
    let request_body = req.collect().await.unwrap().to_bytes();

    println!("{} {} {}", Local::now().format("%Y-%m-%d %H:%M:%S"), method, uri);

    _ = jsight::clear_cache(); // COMMENT THIS FUNCTION ON PROD!!! Only for development purpuses.

    // Validate request

    let request_validation_result = jsight::validate_http_request(
        api_spec_path,
        &method,
        &uri,
        &request_headers,
        &request_body
    );

    match request_validation_result {
        Ok(()) => {}
        Err(error) => {
            let json_error = jsight::serialize_error("json", error).unwrap();
            println!("Request does not match API spec {}: {}", api_spec_path, json_error);
            return Ok(Response::builder().status(400).body(Full::new(Bytes::from(json_error))).unwrap());
        }
    }

    // Prepare stub response

    let response = Response::builder()
        .status(200)
        .body(Full::new(Bytes::from("\"OK\"")))
        .unwrap();

    // Validate response
    let response_headers     = get_http_headers(response.headers());
    let response_status_code = response.status().as_u16();
    let response_body        = response.collect().await.unwrap().to_bytes();

    let response_validation_result = jsight::validate_http_response(
        api_spec_path,
        &method,
        &uri,
        response_status_code.into(),
        &response_headers,
        &response_body
    );

    match response_validation_result {
        Ok(()) => {}
        Err(error) => {
            let json_error = jsight::serialize_error("json", error).unwrap();
            println!("Response does not match API spec {}: {}", api_spec_path, json_error);
            return Ok(Response::builder().status(500).body(Full::new(Bytes::from(json_error))).unwrap());
        }
    }

    Ok(Response::builder().status(response_status_code).body(Full::new(Bytes::from(response_body))).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // initialize jsight
    jsight::init("/opt/lib/libjsight.so").unwrap();
    let stat = jsight::stat().unwrap();
    println!("JSight stat: {}", stat);

    // initialize server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(handle_request))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn get_current_method(req: &Request<Incoming>) -> String {
    match req.method() {
        &Method::GET    => "GET"   .to_string(),
        &Method::POST   => "POST"  .to_string(),
        &Method::PATCH  => "PATCH" .to_string(),
        &Method::PUT    => "PUT"   .to_string(),
        &Method::DELETE => "DELETE".to_string(),
        // Add more HTTP methods as needed
        _ => "UNKNOWN".to_string(),
    }
}

fn get_http_headers(hyper_headers: &hyper::HeaderMap) -> http::HeaderMap {
    let mut http_headers = http::HeaderMap::new();
    hyper_headers.iter().for_each(|(k, v)| {
        http_headers.append(k, v.clone());
    });
    return http_headers;
}