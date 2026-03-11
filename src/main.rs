mod core;
mod providers;
use anyhow::Result;
use tracing::{debug, error, info};
use std::collections::HashMap;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::providers::{DnsFactory, ProviderType};
#[tokio::main]
async fn main() -> Result<()> {
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(stdout_layer)
        .init();
    let cf_token = "ad74vvoDGK0M3aE5Lzgzy8aZDZsXvaYvHP5p0Hfn";
    let zone_name = "duacodie.com";
    info!("Starting DDNS Client");
    debug!("Using Cloudflare Token: {}", cf_token);
    let cf_h = DnsFactory::create(ProviderType::Cloudflare, cf_token);
    let zones = cf_h.list_zones(Some(zone_name)).await?;
    if zones.is_empty() {
        error!("No zones found");
        return Ok(());
    }
    let zt: HashMap<String, String> = zones.into_iter().map(|(id, name)| (name, id)).collect();
    debug!("Zone map created: {:#?}", zt);
    match zt.get(zone_name) {
        Some(zone_id) => {
            debug!("Zone ID for '{}': {}", zone_name, zone_id);
            let records = cf_h.list_records(zone_id, None).await?;
            debug!("DNS Records for '{}': {:#?}", zone_name, records);
            //TODO: fn (zone_id)
        }
        None => error!("Zone '{}' not found.", zone_name),
    }
    Ok(())
}
