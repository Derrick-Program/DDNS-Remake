use salvo::oapi::ToSchema;
use serde::Serialize;
// 從Server出去的module，定義從內部發送的資料結構
#[derive(Serialize, ToSchema, Debug)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    pub status: String,
}