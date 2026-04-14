use crate::command::utils::verify_client_token;
use salvo::prelude::*;
use std::sync::Arc;

// 方案 B：device 識別透過 path param `{deviceid}` 取得，無需 X-Device-ID header。
// 路由必須包含 `{deviceid}` path param，例如 /api/v1/dns_records/{deviceid}
#[handler]
pub async fn token_validator(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // 1. 驗證 Bearer token 格式
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

    // 2. 從 path param 取得 device_id（UUID v5 字串）
    let device_id = match req.param::<String>("deviceid") {
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

    // 3. 查詢 DB 確認裝置存在
    let mut db = app_state.db_service.clone();
    let device = match db.find_by_device_identifier(&device_id) {
        Ok(Some(d)) => d,
        _ => {
            res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
            ctrl.skip_rest();
            return;
        }
    };

    // 4. Argon2 驗證 token hash
    if !verify_client_token(&device.token_hash, &token) {
        res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
        ctrl.skip_rest();
        return;
    }

    ctrl.call_next(req, depot, res).await;
}
