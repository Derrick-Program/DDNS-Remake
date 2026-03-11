mod core;
mod providers;
use anyhow::Result;
use std::collections::HashMap;

use crate::providers::{DnsFactory, ProviderType};
#[tokio::main]
async fn main() -> Result<()> {
    let cf_token = "ad74vvoDGK0M3aE5Lzgzy8aZDZsXvaYvHP5p0Hfn";
    let zone_name = "duacodie.com";

    println!("Cloudflare Token: {}", cf_token);
    let cf_h = DnsFactory::create(ProviderType::Cloudflare, cf_token);
    let zones = cf_h.list_zones(Some(zone_name)).await?;
    if zones.is_empty() {
        println!("No zones found.");
        return Ok(());
    }
    let zt: HashMap<String, String> = zones.into_iter().map(|(id, name)| (name, id)).collect();
    println!("Zone Map: {:#?}", zt);
    match zt.get(zone_name) {
        Some(zone_id) => {
            println!("Zone ID for '{}': {}", zone_name, zone_id);
            let records = cf_h.list_records(zone_id, None).await?;
            println!("DNS Records for '{}': {:#?}", zone_name, records);
            //TODO: fn (zone_id)
        }
        None => println!("Zone '{}' not found.", zone_name),
    }
    Ok(())
}
