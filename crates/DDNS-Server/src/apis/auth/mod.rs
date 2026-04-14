use crate::command::utils::verify_client_token;
use crate::error::{AppError, AppResult};
use crate::middlewares::user::{JwtClaims, get_secret, jwt_middleware};
use ddns_core::{CommonResponse, LoginRequest, RegisterDeviceRequest, TokenResponse};
use jsonwebtoken::{EncodingKey, Header, encode};
use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use salvo::{Router, oapi::endpoint};
use std::sync::Arc;
use tracing::debug;

pub fn routers() -> Router {
    let public_routes = Router::new().push(Router::with_path("login").post(login));
    let protected_routes = Router::with_hoop(jwt_middleware())
        .push(Router::with_path("profile").get(get_profile))
        .push(Router::with_path("is_login").get(is_login))
        .push(
            Router::with_path("devices")
                .post(register_device)
                .push(Router::with_path("{device_name}").delete(delete_device)),
        );
    Router::with_path("auth").push(public_routes).push(protected_routes)
}

#[endpoint]
pub async fn login(
    depot: &mut Depot,
    data: JsonBody<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    let login_data = data.into_inner();
    debug!("Received login request for username: {}", login_data.username);

    let app_state = depot
        .obtain::<Arc<crate::command::AppState>>()
        .map_err(|_| anyhow::anyhow!("Failed to obtain AppState from Depot"))?;
    let mut db_service = app_state.db_service.clone();

    let user = db_service
        .find_user_by_username(&login_data.username)?
        .ok_or(AppError::AuthenticationError)?;

    if !verify_client_token(&user.password_hash, &login_data.password) {
        return Err(AppError::AuthenticationError);
    }

    let exp = chrono::Utc::now().timestamp() + 300; // token 5分鐘後過期
    let claims = JwtClaims { uid: user.id, username: user.username, exp };
    let token =
        encode(&Header::default(), &claims, &EncodingKey::from_secret(get_secret())).unwrap();
    Ok(Json(TokenResponse { token }))
}

#[endpoint]
async fn get_profile(depot: &mut Depot) -> String {
    if let Some(data) = depot.jwt_auth_data::<JwtClaims>() {
        format!("驗證成功！你的 ID 是 {}，使用者名稱是 {}", data.claims.uid, data.claims.username)
    } else {
        "未預期的錯誤".to_string()
    }
}

#[endpoint]
pub async fn is_login(depot: &mut Depot) -> Json<CommonResponse> {
    let is_logged_in = depot.jwt_auth_data::<JwtClaims>().is_some();
    Json(CommonResponse {
        message: if is_logged_in { "已登入".to_string() } else { "未登入".to_string() },
    })
}

#[endpoint]
pub async fn register_device(
    _depot: &mut Depot,
    data: JsonBody<RegisterDeviceRequest>,
) -> AppResult<Json<TokenResponse>> {
    tracing::debug!("Received request to register device with name: {}", data.device_name);
    Err(AppError::NotImplemented("register_device 還未實作".into()))
}

#[endpoint]
pub async fn delete_device(
    _depot: &mut Depot,
    device_name: PathParam<String>,
) -> AppResult<Json<CommonResponse>> {
    tracing::debug!("Received request to delete device with name: {}", device_name);
    Err(AppError::NotImplemented("delete_device 還未實作".into()))
}
