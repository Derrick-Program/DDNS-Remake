use crate::command::utils::generate_api_key;
use crate::error::AppResult;
use crate::middlewares::user::{JwtClaims, get_secret, jwt_middleware};
use ddns_core::{CommonResponse, LoginRequest, TokenResponse};
use jsonwebtoken::{EncodingKey, Header, encode};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use salvo::{Router, oapi::endpoint};
use tracing::debug;

pub fn routers() -> Router {
    let public_routes = Router::new().push(Router::with_path("login").post(login));
    let protected_routes = Router::with_hoop(jwt_middleware())
        .push(Router::with_path("profile").get(get_profile))
        .push(Router::with_path("is_login").get(is_login))
        .push(Router::with_path("exchange").get(exchange_token));
    Router::with_path("auth").push(public_routes).push(protected_routes)
}

#[endpoint]
pub async fn login(data: JsonBody<LoginRequest>) -> Json<TokenResponse> {
    //TODO: 這裡應該要驗證使用者的帳密，並且從資料庫中取出對應的使用者資訊來生成JWT token
    let login_data = data.into_inner();
    debug!("Received login request for username: {}", login_data.username);
    let user_id = 9527; //TODO: 這裡應該從資料庫中取出對應的使用者ID
    let username = login_data.username;
    let exp = chrono::Utc::now().timestamp() + 300; // token 5分鐘後過期
    let claims = JwtClaims { uid: user_id, username: username.to_string(), exp };
    let token =
        encode(&Header::default(), &claims, &EncodingKey::from_secret(get_secret())).unwrap();
    Json(TokenResponse { token })
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
pub async fn exchange_token(depot: &mut Depot) -> AppResult<Json<TokenResponse>> {
    let is_logged_in = depot.jwt_auth_data::<JwtClaims>().is_some();
    if is_logged_in {
        let new_device_token = generate_api_key();
        Ok(Json(TokenResponse { token: new_device_token }))
    } else {
        Err(crate::error::AppError::AuthenticationError)
    }
}
