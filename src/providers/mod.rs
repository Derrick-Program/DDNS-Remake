pub mod cloudflare;
use anyhow::Result;
use std::net::Ipv4Addr;
use async_trait::async_trait;

use crate::providers::cloudflare::CloudflareProvider;

pub type DnsRecord = (String, String, Ipv4Addr);
pub type ZoneInfo = (String, String);

#[async_trait]
pub trait ZoneHandler {
    async fn list_zones(&self, zone_name: Option<&str>) -> Result<Vec<ZoneInfo>>;
}

#[async_trait]
pub trait RecordHandler: ZoneHandler {
    async fn list_records(&self, zone_id: &str, dns_name: Option<&str>) -> Result<Vec<DnsRecord>>;

    async fn create_record(
        &self,
        zone_id: &str,
        new_dns_name: &str,
        new_ip: Ipv4Addr,
        proxied: Option<bool>,
    ) -> Result<DnsRecord>;

    async fn update_record(
        &self,
        zone_id: &str,
        dns_name: &str,
        record_id: &str,
        new_ip: Ipv4Addr,
        proxied: Option<bool>,
    ) -> Result<DnsRecord>;

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<bool>;
}
pub trait DnsProvider: ZoneHandler + RecordHandler + Send + Sync {}
impl<T: ZoneHandler + RecordHandler + Send + Sync> DnsProvider for T {}

pub enum ProviderType {
    Cloudflare,
}

pub struct DnsFactory;
impl DnsFactory {
    pub fn create(provider_type: ProviderType, api_key: &str) -> Box<dyn DnsProvider> {
        match provider_type {
            ProviderType::Cloudflare => Box::new(CloudflareProvider::new(api_key)),
        }
    }
}
