use std::sync::Arc;

use crate::command::AppState;
use crate::middlewares::token_validator::token_validator;
use salvo::oapi::swagger_ui::Url;
use salvo::prelude::*;
use tracing::debug;
pub mod v1;

pub fn all_routers(state: Arc<AppState>) -> Router {
    let v1_routers = v1::routers();
    let mut router = Router::with_path("api").hoop(salvo::affix_state::inject(state)).hoop(token_validator);
    if cfg!(debug_assertions) {
        let doc_v1 = OpenApi::new("API V1", "1.0").merge_router(&v1_routers);
        router = router.unshift(doc_v1.into_router("/docs/v1/openapi.json")).unshift(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .urls(vec![
                    (Url::with_primary("api doc 1", "/docs/v1/openapi.json", true)),
                    // (Url::new("api doc 2", "/api-docs/openapi2.json")),
                ])
                .into_router("/swagger-ui"),
        );
    }
    router = router.push(v1_routers);
    router
}
