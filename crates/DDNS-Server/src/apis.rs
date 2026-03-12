use ddns_core::{r#in::CreateUserRequest, out::UserResponse};
use salvo::prelude::*;
use salvo::{oapi::extract::*};
use tracing::info;
#[endpoint]
pub async fn create_user(new_user: JsonBody<CreateUserRequest>) -> Json<UserResponse> {
    let user = new_user.into_inner();
    info!("Creating user: {:#?}", user);
    Json(UserResponse { id: 1, username: user.username, status: "created".to_string() })
}

#[endpoint]
pub async fn hello() -> &'static str {
    "Hello, World!"
}
//TODO: 添加API，獲取DNS紀錄，更新DNS紀錄，每個路由需要帶Auth token