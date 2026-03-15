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
    #[error("裝置未找到")]
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
        let status = match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DeviceNotFound => StatusCode::NOT_FOUND,
            AppError::AuthenticationError => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        res.status_code(status);
        res.render(Json(CommonResponse { message: self.to_string() }));
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
