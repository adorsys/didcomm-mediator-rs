use sample_pop_server::didgen;

/// Program entry
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable tracing
    tracing_subscriber::fmt::init();

    // Run didgen logic
    didgen::didgen()?;
    Ok(())
}
