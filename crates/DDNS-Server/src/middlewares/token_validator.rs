use crate::command::utils::verify_client_token;
use crate::error::{AppError, AppResult};
use salvo::prelude::*;
use std::sync::Arc;

#[handler]
pub async fn token_validator(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if let Err(e) = validate(req, depot).await {
        e.write(req, depot, res).await;
        ctrl.skip_rest();
        return;
    }
    ctrl.call_next(req, depot, res).await;
}

async fn validate(req: &mut Request, depot: &mut Depot) -> AppResult<()> {
    let token = req
        .header::<String>("x-device-key")
        .filter(|t| t.starts_with("ddns_tok_"))
        .ok_or(AppError::AuthenticationError)?;
    tracing::debug!("Received token: {}", token);

    let device_id = req.param::<String>("deviceid").ok_or(AppError::AuthenticationError)?;
    tracing::debug!("Extracted device_id: {}", device_id);

    let app_state = depot
        .obtain::<Arc<crate::command::AppState>>()
        .map_err(|_| AppError::InternalServerError("AppState not found".into()))?;

    let mut db = app_state.db_service.clone();
    let device = db.find_by_device_identifier(&device_id)?.ok_or(AppError::AuthenticationError)?;

    if !verify_client_token(&device.token_hash, &token) {
        return Err(AppError::AuthenticationError);
    }

    Ok(())
}
