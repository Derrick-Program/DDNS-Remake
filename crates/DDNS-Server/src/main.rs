mod providers;
use anyhow::Result;
use salvo::oapi::swagger_ui::Url;
// use tracing::{debug, error, info};
// use std::collections::HashMap;
use salvo::prelude::*;
use salvo::{oapi::extract::*, server::ServerHandle};
use serde::{Deserialize, Serialize};

use tokio::signal;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

// use crate::providers::{DnsFactory, ProviderType};
#[tokio::main]
async fn main() -> Result<()> {
    let stdout_layer = fmt::layer().with_target(true).with_thread_ids(true);
    let filter_layer =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info")).unwrap();

    tracing_subscriber::registry().with(filter_layer).with(stdout_layer).init();
    // let cf_token = "ad74vvoDGK0M3aE5Lzgzy8aZDZsXvaYvHP5p0Hfn";
    // let zone_name = "duacodie.com";
    // info!("Starting DDNS Client");
    // debug!("Using Cloudflare Token: {}", cf_token);
    // let cf_h = DnsFactory::create(ProviderType::Cloudflare, cf_token);
    // let zones = cf_h.list_zones(Some(zone_name)).await?;
    // if zones.is_empty() {
    //     error!("No zones found");
    //     return Ok(());
    // }
    // let zt: HashMap<String, String> = zones.into_iter().map(|(id, name)| (name, id)).collect();
    // debug!("Zone map created: {:#?}", zt);
    // match zt.get(zone_name) {
    //     Some(zone_id) => {
    //         debug!("Zone ID for '{}': {}", zone_name, zone_id);
    //         let records = cf_h.list_records(zone_id, None).await?;
    //         debug!("DNS Records for '{}': {:#?}", zone_name, records);
    //         //TODO: fn (zone_id)
    //     }
    //     None => error!("Zone '{}' not found.", zone_name),
    // }
    let acceptor = TcpListener::new("0.0.0.0:8698").bind().await;
    let v1_routers = Router::with_path("v1")
        .push(Router::with_path("hello").get(hello))
        .push(Router::with_path("dns_records").post(create_user));
    let doc_v1 = OpenApi::new("API V1", "1.0").merge_router(&v1_routers);
    let router = Router::new()
        .push(v1_routers)
        .unshift(doc_v1.into_router("/docs/v1/openapi.json"))
        .unshift(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .urls(vec![
                    (Url::with_primary("api doc 1", "/docs/v1/openapi.json", true)),
                    // (Url::new("api doc 2", "/api-docs/openapi2.json")),
                ])
                .into_router("/swagger-ui"),
        );
    println!("{router:?}");
    let server = Server::new(acceptor);
    let handle = server.handle();
    tokio::spawn(listen_shutdown_signal(handle));
    server.serve(router).await;
    Ok(())
}

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

#[handler]
async fn hello() -> &'static str {
    "Hello, World!"
}

#[handler]
async fn get_records() -> &'static str {
    "DNS Records"
}

#[derive(Deserialize, ToSchema, Debug)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Serialize, ToSchema, Debug)]
struct UserResponse {
    id: u64,
    username: String,
    status: String,
}

#[endpoint]
async fn create_user(new_user: JsonBody<CreateUserRequest>) -> Json<UserResponse> {
    let user = new_user.into_inner();
    info!("Creating user: {:#?}", user);
    Json(UserResponse { id: 1, username: user.username, status: "created".to_string() })
}
