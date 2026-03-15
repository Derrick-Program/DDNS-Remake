use salvo_oapi::ToSchema;
use serde::Serialize;
use std::net::Ipv4Addr;
use serde::Deserialize;

#[derive(Serialize, ToSchema, Debug)]
pub struct CommonResponse {
    pub message: String,
}

// 進入Server的module，定義從外部接收的資料結構
#[derive(Deserialize, ToSchema, Debug)]
pub struct UpdateDnsRecordRequest {
    #[serde(rename = "Ip")]
    pub ip: Ipv4Addr,
}

#[derive(Debug, ToSchema, Serialize)]
pub struct WebDomain{
    pub hostname: String,
    pub current_ip: Option<String>,
}


#[derive(Debug, ToSchema, Serialize)]
pub struct GetDnsRecordsResponse {
    pub domains: Vec<WebDomain>,
}
