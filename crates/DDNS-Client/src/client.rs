use anyhow::{Context, Result};
use ddns_core::{GetDnsRecordsResponse, LoginRequest, RegisterDeviceRequest, UpdateDnsRecordRequest};
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
        let http = Client::builder().build().context("無法建立 HTTP client")?;
        Ok(Self { http, config })
    }

    /// 呼叫 PATCH /api/v1/dns_records/{device_id} 更新 DNS 記錄
    pub async fn update_dns(&self, ip: Ipv4Addr) -> Result<()> {
        let device_id = ddns_core::get_device_id().map_err(|e| anyhow::anyhow!(e))?;

        let url = format!(
            "{}/api/v1/dns_records/{}",
            self.config.server_url.trim_end_matches('/'),
            device_id
        );

        let body = UpdateDnsRecordRequest { ip, domains: self.config.domains.clone() };

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

/// 登入並取得 JWT，不需要現有 config
pub async fn login(server_url: &str, username: &str, password: &str) -> Result<String> {
    let http = Client::new();
    let url = format!("{}/api/auth/login", server_url.trim_end_matches('/'));

    let body = LoginRequest { username: username.to_string(), password: password.to_string() };

    let resp = http
        .post(&url)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("無法連線至伺服器：{url}"))?;

    let status = resp.status();
    if status.is_success() {
        let token_resp: ddns_core::TokenResponse =
            resp.json().await.context("解析 login 回應失敗")?;
        Ok(token_resp.token)
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(anyhow::anyhow!("登入失敗 {status}：{body}"))
    }
}

/// 用 JWT 註冊裝置並取得 api_key
pub async fn register_device(
    server_url: &str,
    jwt: &str,
    device_name: &str,
    device_id: &str,
) -> Result<String> {
    let http = Client::new();
    let url = format!("{}/api/auth/devices", server_url.trim_end_matches('/'));

    let body = RegisterDeviceRequest {
        device_name: device_name.to_string(),
        device_id: device_id.to_string(),
    };

    let resp = http
        .post(&url)
        .bearer_auth(jwt)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("裝置註冊請求失敗：{url}"))?;

    let status = resp.status();
    if status.is_success() {
        let reg_resp: ddns_core::RegisterDeviceResponse =
            resp.json().await.context("解析 register_device 回應失敗")?;
        Ok(reg_resp.api_key)
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(anyhow::anyhow!("裝置註冊失敗 {status}：{body}"))
    }
}

/// 取得裝置可更新的域名清單
pub async fn fetch_domains(
    server_url: &str,
    api_key: &str,
    device_id: &str,
) -> Result<Vec<String>> {
    let http = Client::new();
    let url = format!("{}/api/v1/dns_records/{}", server_url.trim_end_matches('/'), device_id);

    let resp = http
        .get(&url)
        .bearer_auth(api_key)
        .send()
        .await
        .with_context(|| format!("取得域名清單失敗：{url}"))?;

    let status = resp.status();
    if status.is_success() {
        let data: GetDnsRecordsResponse =
            resp.json().await.context("解析域名清單回應失敗")?;
        Ok(data.domains.into_iter().map(|d| d.hostname).collect())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(anyhow::anyhow!("取得域名清單失敗 {status}：{body}"))
    }
}
