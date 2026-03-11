use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use anyhow::Result;
use ddns_client::get_public_ip;
#[tokio::main]
async fn main() -> Result<()> {
    let stdout_layer = fmt::layer().with_target(true).with_thread_ids(true);
    let filter_layer =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info")).unwrap();

    tracing_subscriber::registry().with(filter_layer).with(stdout_layer).init();
    info!("Starting DDNS Client...");
    debug!("Client IP: {}", get_public_ip().await?);
    Ok(())
}
