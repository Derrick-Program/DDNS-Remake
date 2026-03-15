use ddns_core::CommonResponse;
use salvo::http::StatusCode;
use salvo::oapi::EndpointOutRegister;
use salvo::{oapi, prelude::*};
use thiserror::Error;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[allow(unused)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("資料庫錯誤: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("連接池錯誤: {0}")]
    PoolError(#[from] diesel::r2d2::Error),
    #[error("Device not found")]
    DeviceNotFound,
    #[error("驗證失敗")]
    AuthenticationError,
    #[error("授權失敗")]
    AuthorizationError,
    #[error("資源未找到")]
    NotFound,
    #[error("無效的輸入: {0}")]
    InvalidInput(String),
    #[error("內部伺服器錯誤: {0}")]
    InternalServerError(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            AppError::DatabaseError(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(CommonResponse { message: format!("Database error: {}", e) }));
            }
            AppError::PoolError(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(CommonResponse {
                    message: format!("Connection pool error: {}", e),
                }));
            }
            AppError::DeviceNotFound => {
                res.status_code(StatusCode::NOT_FOUND);
                res.render(Json(CommonResponse { message: self.to_string() }));
            }
            AppError::AuthenticationError => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(CommonResponse { message: self.to_string() }));
            }
            AppError::AuthorizationError => {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(CommonResponse { message: self.to_string() }));
            }
            AppError::NotFound => {
                res.status_code(StatusCode::NOT_FOUND);
                res.render(Json(CommonResponse { message: self.to_string() }));
            }
            AppError::InvalidInput(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(CommonResponse { message: format!("Invalid input: {}", e) }));
            }
            AppError::InternalServerError(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(CommonResponse {
                    message: format!("Internal server error: {}", e),
                }));
            }
            AppError::Anyhow(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(CommonResponse { message: e.to_string() }));
            }
        }
    }
}

impl EndpointOutRegister for AppError {
    fn register(components: &mut oapi::Components, operation: &mut oapi::Operation) {
        let error_content = StatusError::to_schema(components);

        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            oapi::Response::new("Bad Request")
                .add_content("application/json", error_content.clone()),
        );
        operation.responses.insert(
            StatusCode::UNAUTHORIZED.as_str(),
            oapi::Response::new("Unauthorized")
                .add_content("application/json", error_content.clone()),
        );
        operation.responses.insert(
            StatusCode::FORBIDDEN.as_str(),
            oapi::Response::new("Forbidden").add_content("application/json", error_content.clone()),
        );
        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            oapi::Response::new("Not Found").add_content("application/json", error_content.clone()),
        );
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            oapi::Response::new("Internal Server Error")
                .add_content("application/json", error_content.clone()),
        );
    }
}
