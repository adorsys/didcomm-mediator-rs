#[cfg(feature = "plugin-did_endpoint")]
use did_endpoint::didgen;

/// Program entry
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "plugin-did_endpoint")]
    {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        // Load dotenv-flow variables
        dotenv_flow::dotenv_flow().ok();

        // Enable logging
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .init();

        // Run didgen logic
        let storage_dirpath = std::env::var("STORAGE_DIRPATH")?;
        let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN")?;
        didgen::didgen(&storage_dirpath, &server_public_domain)?;
        Ok(())
    }

    #[cfg(not(feature = "plugin-did_endpoint"))]
    {
        Err("You must enable `plugin-did_endpoint` to run this command.".into())
    }
}
