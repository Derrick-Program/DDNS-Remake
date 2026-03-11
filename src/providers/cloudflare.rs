use crate::{
    providers::{RecordHandler, ZoneHandler,ZoneInfo,DnsRecord},
};
use anyhow::Result;
use cloudflare::framework::Environment::Production;
use cloudflare::framework::auth::Credentials::UserAuthToken;
use cloudflare::framework::client::{ClientConfig, async_api::Client as AsyncClient};
use std::net::Ipv4Addr;
pub struct CloudflareProvider {
    client: AsyncClient,
}

impl CloudflareProvider {
    pub fn new(api_key: &str) -> Self {
        let client = AsyncClient::new(
            UserAuthToken { token: api_key.into() },
            ClientConfig::default(),
            Production,
        )
        .inspect_err(|e| eprintln!("Error creating Cloudflare client: {}", e))
        .unwrap();
        Self { client }
    }
}

#[async_trait::async_trait]
impl ZoneHandler for CloudflareProvider {
    async fn list_zones(&self, zone_name: Option<&str>) -> Result<Vec<ZoneInfo>> {
        todo!()
    }
}

#[async_trait::async_trait]
impl RecordHandler for CloudflareProvider {
    async fn list_records(&self, zone_id: &str, dns_name: Option<&str>) -> Result<Vec<DnsRecord>> {
        todo!()
    }

    async fn update_record(
        &self,
        zone_id: &str,
        dns_name: &str,
        record_id: &str,
        new_ip: Ipv4Addr,
        proxied: Option<bool>,
    ) -> Result<DnsRecord> {
        todo!()
    }

    async fn create_record(
        &self,
        zone_id: &str,
        new_dns_name: &str,
        new_ip: Ipv4Addr,
        proxied: Option<bool>,
    ) -> Result<DnsRecord> {
        unimplemented!("Create record is not implemented yet.")
    }

    async fn delete_record(&self, _zone_id: &str, _record_id: &str) -> Result<bool> {
        unimplemented!("Delete record is not implemented yet.")
    }
}
