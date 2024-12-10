use axum::{
    body::{self, Bytes},
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};
use sysinfo::System;
use hyper::Server;
use tokio::test;
use crate::{create_router, AppMetrics}; // Import the create_router and AppMetrics from your module

// A test helper function to set up a test server and make requests
async fn get_response(path: &str) -> Result<hyper::Response<hyper::Body>, Box<dyn std::error::Error>> {
    // Set up the router
    let app = create_router();

    // Create a mock request for the given path
    let req = Request::builder()
        .uri(path)
        .body(body::Body::empty())?;

    // Create a test server
    let server = Server::bind(&"127.0.0.1:0".parse()?)
        .serve(app.into_make_service());

    // Execute the request
    let client = hyper::Client::new();
    let res = client.request(req).await?;

    Ok(res)
}

#[tokio::test]
async fn test_health_check() {
    // Make a request to the health check endpoint
    let response = get_response("/health").await.unwrap();

    // Assert that the response status is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Read the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert that the response body is "Server is healthy"
    assert_eq!(body_str, "Server is healthy");
}

#[tokio::test]
async fn test_system_metrics() {
    // Make a request to the system metrics endpoint
    let response = get_response("/health/metrics").await.unwrap();

    // Assert that the response status is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Read the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Check if the body contains memory and CPU usage information
    assert!(body_str.contains("Memory:"));
    assert!(body_str.contains("CPU usage:"));
}

#[tokio::test]
async fn test_system_metrics_locked_state() {
    // Set up a custom AppMetrics state
    let app_metrics = AppMetrics::new();
    let metrics = Arc::new(Mutex::new(app_metrics));
    let app = Router::new().route("/health/metrics", get(system_metrics)).with_state(metrics);

    // Make a request to the system metrics endpoint
    let response = get_response("/health/metrics").await.unwrap();

    // Assert that the response status is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Read the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Check if the body contains memory and CPU usage information
    assert!(body_str.contains("Memory:"));
    assert!(body_str.contains("CPU usage:"));
}
