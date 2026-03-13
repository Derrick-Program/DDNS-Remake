use std::net::Ipv4Addr;

use salvo::oapi::ToSchema;
use serde::Deserialize;
// 進入Server的module，定義從外部接收的資料結構
#[derive(Deserialize, ToSchema, Debug)]
pub struct UpdateDnsRecordRequest {
    #[serde(rename = "Ip")]
    pub ip: Ipv4Addr,
}
