use salvo::prelude::*;
#[handler]
pub async fn token_validator(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let auth_header = req.header::<String>("authorization");
    let is_valid = auth_header
        .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_owned()))
        .filter(|t| t.starts_with("ddns_tok_"))
        // .filter(|t| 這裡可以加入資料庫驗證邏輯)
        //TODO: 從資料庫中取出 hashed token，並使用 verify_client_token 進行驗證
        .is_some();

    if is_valid {
        ctrl.call_next(req, depot, res).await;
    } else {
        res.status_code(StatusCode::UNAUTHORIZED).render("Unauthorized");
        ctrl.skip_rest();
    }
}