use anyhow::{Context, Result};
use ddns_core::UpdateDnsRecordRequest;
use reqwest::Client;
use std::net::Ipv4Addr;
use tracing::info;

use crate::config::ClientConfig;

pub struct DdnsClient {
    http: Client,
    config: ClientConfig,
}

impl DdnsClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let http = Client::builder()
            .build()
            .context("無法建立 HTTP client")?;
        Ok(Self { http, config })
    }

    /// 呼叫 PATCH /api/v1/dns_records/{device_id} 更新 DNS 記錄
    pub async fn update_dns(&self, ip: Ipv4Addr) -> Result<()> {
        let device_id = ddns_core::get_device_id()
            .map_err(|e| anyhow::anyhow!(e))?;

        let url = format!(
            "{}/api/v1/dns_records/{}",
            self.config.server_url.trim_end_matches('/'),
            device_id
        );

        let body = UpdateDnsRecordRequest { ip };

        let resp = self
            .http
            .patch(&url)
            .bearer_auth(&self.config.device_token)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("HTTP 請求失敗：{url}"))?;

        let status = resp.status();
        if status.is_success() {
            info!("DNS 更新成功：{ip} (HTTP {status})");
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("伺服器回傳錯誤 {status}：{body}"))
        }
    }
}
