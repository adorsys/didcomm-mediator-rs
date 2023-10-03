#[cfg(feature = "plugin-didpop")]
use sample_pop_server::plugin::didpop::didgen;

/// Program entry
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "plugin-didpop")]
    {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        // Load dotenv-flow variables
        dotenv_flow::dotenv_flow().ok();

        // Enable logging
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .init();

        // Run didgen logic
        didgen::didgen()?;
        Ok(())
    }

    #[cfg(not(feature = "plugin-didpop"))]
    {
        Err("You must enable `plugin-didpop` to run this command.".into())
    }
}
