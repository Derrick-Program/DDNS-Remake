use salvo::oapi::ToSchema;
use serde::Serialize;
// 從Server出去的module，定義從內部發送的資料結構
#[derive(Serialize, ToSchema, Debug)]
pub struct CommonResponse {
    pub message: String,
}
