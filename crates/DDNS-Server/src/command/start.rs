use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
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
    let router = apis::all_routers(state); 
    debug!("{router:?}");
    let server = Server::new(acceptor);
    let handle = server.handle();
    tokio::spawn(listen_shutdown_signal(handle));
    server.serve(router).await;
    Ok(())
}