use crate::providers::{DnsRecord, RecordHandler, ZoneHandler, ZoneInfo};
use anyhow::Result;
use cloudflare::endpoints::dns::dns::{
    CreateDnsRecord, CreateDnsRecordParams, DeleteDnsRecord, DnsContent, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams
};
use cloudflare::endpoints::zones::zone::{ListZones, ListZonesParams, Status};
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
        let params = ListZonesParams {
            status: Some(Status::Active),
            name: zone_name.map(|s| s.to_string()),
            ..Default::default()
        };

        match self.client.request(&ListZones { params }).await {
            Ok(response) => {
                let zones: Vec<ZoneInfo> =
                    response.result.into_iter().map(|zone| (zone.id, zone.name)).collect();
                Ok(zones)
            }
            Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
        }
    }
}

#[async_trait::async_trait]
impl RecordHandler for CloudflareProvider {
    async fn list_records(&self, zone_id: &str, dns_name: Option<&str>) -> Result<Vec<DnsRecord>> {
        let params =
            ListDnsRecordsParams { name: dns_name.map(|s| s.to_string()), ..Default::default() };
        match self.client.request(&ListDnsRecords { zone_identifier: zone_id, params }).await {
            Ok(response) => {
                let records: Vec<DnsRecord> = response
                    .result
                    .into_iter()
                    .filter_map(|record| match record.content {
                        DnsContent::A { content } => Some((record.id, record.name, content)),
                        _ => None,
                    })
                    .collect();
                Ok(records)
            }
            Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
        }
    }

    async fn update_record(
        &self,
        zone_id: &str,
        dns_name: &str,
        record_id: &str,
        new_ip: Ipv4Addr,
        proxied: Option<bool>,
    ) -> Result<DnsRecord> {
        let params = UpdateDnsRecordParams {
            content: DnsContent::A { content: new_ip },
            proxied,
            ttl: None,
            name: dns_name,
        };
        let resp = self
            .client
            .request(&UpdateDnsRecord { zone_identifier: zone_id, identifier: record_id, params })
            .await?;

        let ip = match resp.result.content {
            DnsContent::A { content } => content,
            _ => return Err(anyhow::anyhow!("Record is not an A record")),
        };

        Ok((resp.result.id, resp.result.name, ip))
    }

    async fn create_record(
        &self,
        zone_id: &str,
        new_dns_name: &str,
        new_ip: Ipv4Addr,
        proxied: Option<bool>,
    ) -> Result<DnsRecord> {
        let params = CreateDnsRecordParams {
            ttl: None,
            priority: None,
            proxied,
            name: new_dns_name,
            content: DnsContent::A { content: new_ip },
        };
        let resp = self.client.request(&CreateDnsRecord { zone_identifier: zone_id, params }).await?;
        let ip = match resp.result.content {
            DnsContent::A { content } => content,
            _ => return Err(anyhow::anyhow!("Record is not an A record")),
        };
        Ok((resp.result.id, resp.result.name, ip))
    }

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<bool> {
        match self.client.request(&DeleteDnsRecord{zone_identifier: zone_id,identifier: record_id}).await {
            Ok(_) => {
                Ok(true)
            }
            Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
        }
    }
}
