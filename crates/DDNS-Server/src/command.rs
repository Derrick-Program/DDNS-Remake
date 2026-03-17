use std::{net::SocketAddr, sync::Arc};

use crate::{
    cli::{Cli, Commands, ConfigSubcommands},
    server,
};
use anyhow::Result;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub enum CommandResult {
    Continue,
    Exit,
}

#[allow(unused)]
pub struct AppState {
    pub db_service: crate::db::DbService,
}

pub async fn handle(cli: Cli, ctx: &Arc<AppState>) -> Result<CommandResult> {
    let stdout_layer = fmt::layer().with_target(true).with_thread_ids(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cli.verbosity.to_string()));
    tracing_subscriber::registry().with(filter_layer).with(stdout_layer).init();
    match &cli.command {
        Commands::Exit => Ok(CommandResult::Exit),

        Commands::Config(config_args) => {
            match &config_args.action {
                ConfigSubcommands::Generate { force, format } => {
                    println!("正在產生 {} 格式的設定檔 (強制覆蓋: {})", format, force);
                }
                ConfigSubcommands::Check => {
                    println!("正在檢查設定檔：{}", cli.config);
                }
            }
            Ok(CommandResult::Continue)
        }

        Commands::Start { port, host } => {
            info!("Starting DDNS Server");
            let sl = SocketAddr::new((*host).into(), *port);
            server::start_server(ctx.clone(), sl).await?;
            Ok(CommandResult::Continue)
        }

        Commands::Database(db_args) => {
            println!("資料庫相關操作: {:#?}", db_args);
            Ok(CommandResult::Continue)
        }
    }
}
