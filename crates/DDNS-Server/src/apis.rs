use crate::middleware::token_validator;
use salvo::prelude::*;
use tracing::debug;
// web -> handler -> service -> db -> model
pub mod v1 {
    use std::sync::Arc;

    use crate::error::{AppError, AppResult};

    use super::*;

    use ddns_core::{CommonResponse, GetDnsRecordsResponse, UpdateDnsRecordRequest, WebDomain};
    use salvo::oapi::extract::{JsonBody, PathParam};
    use uuid::Uuid;

    pub fn routers() -> Router {
        //TODO: 這裡的路由需要區分使用者調用與設備調用
        //TODO: 之後再擴充使用者可以登入的話，才需要驗證使用者的password
        //TODO: 目前先實作設備調用的API，所以需要帶上設備的token
        Router::with_path("v1").hoop(token_validator).push(
            Router::with_path("dns_records")
                .push(Router::with_path("{host_uuid}").get(self::get_dns_records))
                .push(Router::with_path("{record_id}").patch(self::update_dns_record)),
        )
    }

    //TODO: 添加API，獲取DNS紀錄，更新DNS紀錄

    /// 獲取裝置可以更新的DNS紀錄列表，包含hostname和當前IP等資訊
    #[endpoint]
    pub async fn get_dns_records(
        depod: &mut Depot,
        host_uuid: PathParam<Uuid>,
    ) -> AppResult<Json<GetDnsRecordsResponse>> {
        debug!("Received request to get DNS records");
        let app_state = depod
            .obtain::<Arc<crate::server::AppState>>()
            .map_err(|_| anyhow::anyhow!("Failed to obtain AppState from Depot"))?;
        let mut db_service = app_state.db_service.clone();
        let h_id = host_uuid.into_inner().to_string();
        let dev_data =
            db_service.find_by_device_identifier(&h_id)?.ok_or(AppError::DeviceNotFound)?;
        let domains = db_service.find_active_domains_by_device_id(dev_data.id)?;
        Ok(Json(GetDnsRecordsResponse {
            domains: domains.into_iter().map(WebDomain::from).collect(),
        }))
    }

    //TODO: 更新的時候需要檢查DNS name 是否與用戶綁定，並且檢查IP格式是否正確
    #[endpoint]
    pub async fn update_dns_record(
        res: &mut Response,
        record_id: PathParam<Uuid>,
        data: JsonBody<UpdateDnsRecordRequest>,
    ) {
        debug!("Received request to update DNS record with ID: {}", record_id);
        debug!("Request data.ip: {:?}", data.ip);
        // let data = data.into_inner();
        res.status_code(StatusCode::NOT_IMPLEMENTED)
            .render(Json(CommonResponse { message: "Not Implemented".into() }));
    }
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
            .json(&json!({ "ip": "1.1.1.1" }))
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::UNAUTHORIZED);
    }

    // #[tokio::test]
    // async fn test_update_dns_record_not_implemented() {
    //     let router = v1::routers();
    //     let record_id = Uuid::new_v4();

    //     // 模擬帶有 Token 的請求 (假設你的 validator 檢查某個 header)
    //     let mut res =
    //         TestClient::patch(format!("http://127.0.0.1:5800/v1/dns_records/{}", record_id))
    //             .insert_header("Authorization", "Bearer valid_token")
    //             .json(&json!({ "ip": "1.1.1.1" }))
    //             .send(router)
    //             .await;

    //     // 目前你的實作回傳 NOT_IMPLEMENTED
    //     assert_eq!(res.status_code.unwrap(), StatusCode::NOT_IMPLEMENTED);

    //     let body: CommonResponse = res.parse_json().await.unwrap();
    //     assert_eq!(body.message, "Not Implemented");
    // }
}
