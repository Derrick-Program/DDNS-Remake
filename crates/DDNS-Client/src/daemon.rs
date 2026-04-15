use std::net::Ipv4Addr;
use std::time::Duration;

use anyhow::Result;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::client::DdnsClient;
use crate::config::ClientConfig;
use crate::get_public_ip;

const MAX_RETRY_DELAY_SECS: u64 = 300;
const BASE_RETRY_DELAY_SECS: u64 = 5;

pub async fn run(config: ClientConfig) -> Result<()> {
    let interval = config.check_interval_secs;
    let client = DdnsClient::new(config)?;
    let mut last_ip: Option<Ipv4Addr> = None;

    info!("DDNS daemon 已啟動，每 {interval} 秒偵測一次 IP");

    loop {
        match get_public_ip().await {
            Ok(ip) => {
                if last_ip != Some(ip) {
                    info!("IP 變更偵測：{:?} → {ip}", last_ip);
                    update_with_retry(&client, ip).await;
                    last_ip = Some(ip);
                } else {
                    info!("IP 未變更：{ip}，跳過更新");
                }
            }
            Err(e) => {
                warn!("無法取得 public IP：{e}");
            }
        }

        sleep(Duration::from_secs(interval)).await;
    }
}

async fn update_with_retry(client: &DdnsClient, ip: Ipv4Addr) {
    let mut attempt = 0u32;

    loop {
        match client.update_dns(ip).await {
            Ok(()) => return,
            Err(e) => {
                attempt += 1;
                let delay = (BASE_RETRY_DELAY_SECS * 2u64.pow(attempt - 1)).min(MAX_RETRY_DELAY_SECS);
                error!("DNS 更新失敗（第 {attempt} 次）：{e}，{delay} 秒後重試");
                sleep(Duration::from_secs(delay)).await;
            }
        }
    }
}
