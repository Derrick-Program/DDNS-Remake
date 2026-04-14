use salvo_oapi::ToSchema;
use serde::Serialize;
use std::net::Ipv4Addr;
use serde::Deserialize;
use uuid::Uuid;

pub const DEVICE_NAMESPACE: Uuid = uuid::uuid!("c2139751-7d74-4ae1-b413-54c0155ea5aa");

pub fn get_device_id() -> Result<Uuid, String> {
    let machine_id = machine_uid::get()
        .map_err(|e| format!("無法取得 Machine ID: {:?}", e))?;
    Ok(Uuid::new_v5(&DEVICE_NAMESPACE, machine_id.as_bytes()))
}

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
    pub device_id: String,
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
