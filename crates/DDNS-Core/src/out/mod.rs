use salvo::oapi::ToSchema;
use serde::Serialize;

#[derive(Serialize, ToSchema, Debug)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    pub status: String,
}