use crate::command::utils::verify_client_token;
use salvo::prelude::*;
use std::sync::Arc;

// TODO(S2-1): 目前用 X-Device-ID header 識別裝置（方案 A）。
//   待 PATCH route 改為 /dns_records/{deviceid} 後，應改從 path 取得 device_id（方案 B），移除 X-Device-ID 依賴。
#[handler]
pub async fn token_validator(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let token = match req
        .header::<String>("authorization")
        .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_owned()))
        .filter(|t| t.starts_with("ddns_tok_"))
    {
        Some(t) => t,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
            ctrl.skip_rest();
            return;
        }
    };

    let device_id = match req.header::<String>("x-device-id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
            ctrl.skip_rest();
            return;
        }
    };

    let app_state = match depot.obtain::<Arc<crate::command::AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR).render("Internal Server Error");
            ctrl.skip_rest();
            return;
        }
    };

    let mut db = app_state.db_service.clone();
    let device = match db.find_by_device_identifier(&device_id) {
        Ok(Some(d)) => d,
        _ => {
            res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
            ctrl.skip_rest();
            return;
        }
    };

    if !verify_client_token(&device.token_hash, &token) {
        res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
        ctrl.skip_rest();
        return;
    }

    ctrl.call_next(req, depot, res).await;
}