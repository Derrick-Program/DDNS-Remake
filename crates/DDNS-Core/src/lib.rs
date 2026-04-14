use salvo_oapi::ToSchema;
use serde::Serialize;
use std::net::Ipv4Addr;
use serde::Deserialize;

#[derive(Serialize, ToSchema, Debug)]
pub struct CommonResponse {
    pub message: String,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct RegisterDeviceRequest {
    pub device_name: String,
    pub device_id: String, // UUID v5，由 client 以 machine-uid 產生
}

#[derive(Serialize, ToSchema, Debug)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct RegisterDeviceResponse {
    pub device_name: String,
    pub device_id: String,
    pub api_key: String,
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
