use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3004));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path().to_owned();

    println!("Forming request: {}", did_to_url(path.as_str()));
    let response = Response::new(Body::from(did_to_url(path.as_str())));

    Ok(response)
}

fn did_to_url(did: &str) -> String {
    println!("Segments: {:?}", did);

    let segments: Vec<&str> = did.split(':').collect();

    println!("Segments: {:?}", segments);

    // Check if the DID is of the expected format
    if segments.len() < 4 || segments[0] != "did" || segments[1] != "web" {
        return String::from("Invalid DID format");
    }

    // Extract the domain and path segments
    let domain = segments[2];
    let path_segments = &segments[3..];

    // Join the path segments with slashes
    let path = path_segments.join("/");

    // Construct the URL
    let url = format!("https://{}{}/did.json", domain, path);

    url
}
