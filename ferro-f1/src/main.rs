mod event;
mod cli;

use clap::Parser;
use cli::Cli;
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt::init();
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    init_tracing();

    let cli = Cli::parse();
    
    tracing::info!("ferro-f1 starting, live={}, mock={}", cli.live, cli.mock);
    
    Ok(())
}