use super::*;
use crate::error::{AppError, AppResult};
use crate::providers::{DnsFactory, ProviderType};
use ddns_core::{CommonResponse, GetDnsRecordsResponse, UpdateDnsRecordRequest, WebDomain};
use salvo::oapi::extract::{JsonBody, PathParam};
use std::sync::Arc;
use uuid::Uuid;

pub fn routers() -> Router {
    Router::with_path("v1").hoop(token_validator).push(
        Router::with_path("dns_records").push(
            Router::with_path("{deviceid}")
                .get(self::get_dns_records)
                .patch(self::update_dns_record),
        ),
    )
}

/// 獲取裝置可以更新的DNS紀錄列表，包含hostname和當前IP等資訊
#[endpoint]
pub async fn get_dns_records(
    depod: &mut Depot,
    deviceid: PathParam<Uuid>,
) -> AppResult<Json<GetDnsRecordsResponse>> {
    debug!("Received request to get DNS records");
    let app_state = depod
        .obtain::<Arc<crate::command::AppState>>()
        .map_err(|_| anyhow::anyhow!("Failed to obtain AppState from Depot"))?;
    let mut db_service = app_state.db_service.clone();
    let h_id = deviceid.into_inner().to_string();
    debug!("Extracted deviceid: {}", h_id);
    let dev_data = db_service.find_by_device_identifier(&h_id)?.ok_or(AppError::DeviceNotFound)?;
    let domains = db_service.find_active_domains_by_device_id(dev_data.id)?;
    debug!("Found {} active domains for device_id {}", domains.len(), dev_data.id);
    Ok(Json(GetDnsRecordsResponse { domains: domains.into_iter().map(WebDomain::from).collect() }))
}

/// 更新裝置所有 active 域名的 IP，並同步更新 Cloudflare DNS 記錄
#[endpoint]
pub async fn update_dns_record(
    depod: &mut Depot,
    deviceid: PathParam<Uuid>,
    data: JsonBody<UpdateDnsRecordRequest>,
) -> AppResult<Json<CommonResponse>> {
    let new_ip = data.into_inner().ip;
    debug!("Received request to update DNS records for device: {}, new IP: {}", deviceid, new_ip);

    let app_state = depod
        .obtain::<Arc<crate::command::AppState>>()
        .map_err(|_| anyhow::anyhow!("Failed to obtain AppState from Depot"))?;

    let api_key = app_state.config.cloudflare.api_key.clone();
    if api_key.is_empty() {
        return Err(AppError::InternalServerError(
            "Cloudflare API key 未設定，請執行: config set cloudflare.api_key <key>".into(),
        ));
    }

    let mut db_service = app_state.db_service.clone();
    let device_id_str = deviceid.into_inner().to_string();

    let device =
        db_service.find_by_device_identifier(&device_id_str)?.ok_or(AppError::DeviceNotFound)?;

    let active_domains = db_service.find_active_domains_by_device_id(device.id)?;
    if active_domains.is_empty() {
        return Ok(Json(CommonResponse { message: "此裝置無活躍域名".into() }));
    }

    let cf = DnsFactory::create(ProviderType::Cloudflare, &api_key);

    // 取得所有 Zone，建立 zone_name -> zone_id 的對應表（依名稱長度降序，優先匹配最長）
    let zones = cf
        .list_zones(None)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Cloudflare 列出 Zone 失敗: {}", e)))?;

    let mut zone_map: Vec<(String, String)> =
        zones.into_iter().map(|(id, name)| (name, id)).collect();
    zone_map.sort_by(|a, b| b.0.len().cmp(&a.0.len())); // 最長優先 (sub-zone 優先於 parent-zone)

    let total = active_domains.len();
    let mut updated = 0usize;
    let mut error_msgs: Vec<String> = Vec::new();

    for domain in &active_domains {
        let hostname = &domain.hostname;

        // 找到最匹配的 Zone（hostname 為 zone_name 本身，或以 .zone_name 結尾）
        let zone_id = zone_map
            .iter()
            .find(|(zone_name, _)| {
                hostname == zone_name
                    || hostname.ends_with(&format!(".{}", zone_name))
            })
            .map(|(_, id)| id.clone());

        let zone_id = match zone_id {
            Some(id) => id,
            None => {
                error_msgs.push(format!("{}: 無對應的 Cloudflare Zone", hostname));
                continue;
            }
        };

        // 查詢該 hostname 在 Cloudflare 上的 DNS A record ID
        let records = match cf.list_records(&zone_id, Some(hostname)).await {
            Ok(r) => r,
            Err(e) => {
                error_msgs.push(format!("{}: 查詢 DNS 記錄失敗: {}", hostname, e));
                continue;
            }
        };

        let cf_record_id = match records.into_iter().find(|(_, name, _)| name == hostname) {
            Some((id, _, _)) => id,
            None => {
                error_msgs.push(format!("{}: Cloudflare 上無此 DNS A 記錄", hostname));
                continue;
            }
        };

        // 更新 Cloudflare DNS 記錄
        match cf.update_record(&zone_id, hostname, &cf_record_id, new_ip, None).await {
            Ok(_) => {
                db_service.update_domain_ip(domain.id, new_ip)?;
                updated += 1;
                debug!("Updated {} -> {}", hostname, new_ip);
            }
            Err(e) => {
                error_msgs.push(format!("{}: Cloudflare 更新失敗: {}", hostname, e));
            }
        }
    }

    let message = if error_msgs.is_empty() {
        format!("已更新 {}/{} 個域名 IP 為 {}", updated, total, new_ip)
    } else {
        format!(
            "已更新 {}/{} 個域名 IP 為 {}；錯誤: {}",
            updated,
            total,
            new_ip,
            error_msgs.join("; ")
        )
    };

    Ok(Json(CommonResponse { message }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use salvo::test::TestClient;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_update_dns_record_unauthorized() {
        let router = v1::routers();
        let record_id = Uuid::new_v4();
        let service = Service::new(router);

        let res = TestClient::patch(format!("http://127.0.0.1:8698/v1/dns_records/{}", record_id))
            .json(&json!({ "Ip": "1.1.1.1" }))
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::UNAUTHORIZED);
    }
}
