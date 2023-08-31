use sample_pop_server::didgen;
use std::error::Error;

/// Program entry
fn main() -> Result<(), Box<dyn Error>> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable tracing
    tracing_subscriber::fmt::init();

    // Run didgen logic
    didgen::didgen().map(|_| ())
}
