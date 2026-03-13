use crate::middleware::token_validator;
use salvo::prelude::*;
use tracing::debug;
pub mod v1 {
    use super::*;

    use ddns_core::r#in::UpdateDnsRecordRequest;
    use salvo::oapi::extract::{JsonBody, PathParam};
    use uuid::Uuid;

    pub fn routers() -> Router {
        Router::with_path("v1").hoop(token_validator).push(
            Router::with_path("dns_records")
                .get(self::get_dns_records)
                .push(Router::with_path("{record_id}").patch(self::update_dns_record)),
        )
    }

    //TODO: 添加API，獲取DNS紀錄，更新DNS紀錄

    #[endpoint]
    pub async fn get_dns_records() -> &'static str {
        "Get DNS Records - Not Implemented"
    }

    //TODO: 更新的時候需要檢查DNS name 是否與用戶綁定，並且檢查IP格式是否正確
    #[endpoint]
    pub async fn update_dns_record(
        record_id: PathParam<Uuid>,
        data: JsonBody<UpdateDnsRecordRequest>,
    ) -> &'static str {
        debug!("Received request to update DNS record with ID: {}", record_id);
        debug!("Request data.ip: {:?}", data.ip);
        // let data = data.into_inner();

        "Update DNS Record - Not Implemented"
    }
}
