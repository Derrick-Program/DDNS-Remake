use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use salvo::oapi::swagger_ui::Url;
use salvo::prelude::*;
use salvo::server::ServerHandle;
use tokio::signal;
use tracing::debug;

use crate::apis;
use crate::command::AppState;


async fn listen_shutdown_signal(handle: ServerHandle) {
    // Wait Shutdown Signal
    let ctrl_c = async {
        // Handle Ctrl+C signal
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        // Handle SIGTERM on Unix systems

        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(windows)]
    let terminate = async {
        // Handle Ctrl+C on Windows (alternative implementation)
        signal::windows::ctrl_c().expect("failed to install signal handler").recv().await;
    };

    tokio::select! {
        _ = ctrl_c => println!("ctrl_c signal received"),
        _ = terminate => println!("terminate signal received"),
    };

    handle.stop_graceful(None);
}

pub async fn start_server(state: Arc<AppState>, sl: SocketAddr) -> Result<()> {
    let acceptor = TcpListener::new(sl).bind().await;
    let v1_routers = apis::v1::routers();
    let mut router = Router::with_path("api").hoop(salvo::affix_state::inject(state));
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
    debug!("{router:?}");
    let server = Server::new(acceptor);
    let handle = server.handle();
    tokio::spawn(listen_shutdown_signal(handle));
    server.serve(router).await;
    Ok(())
}
