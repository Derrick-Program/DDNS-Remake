use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr};
#[allow(unused)]
pub async fn get_public_ip() -> Result<Ipv4Addr> {
    match public_ip_address::perform_lookup(None).await {
        Ok(resp) => {
            if let IpAddr::V4(ip) = resp.ip {
                Ok(ip)
            } else {
                Err(anyhow::anyhow!("Received an unexpected IP type"))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
    }
}