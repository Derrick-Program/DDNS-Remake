use std::{net::SocketAddr, sync::Arc};

use crate::{
    cli::{Cli, Commands, ConfigSubcommands},
    server,
};
use anyhow::Result;
use tracing::info;

pub enum CommandResult {
    Continue,
    Exit,
}

#[allow(unused)]
pub struct AppState {
    pub db_service: crate::db::DbService,
}

pub async fn handle(cli: Cli, ctx: &Arc<AppState>) -> Result<CommandResult> {
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
                ConfigSubcommands::Get { key } => {
                    println!("正在獲取設定檔中的值，key: {}", key);
                }
                ConfigSubcommands::Set { key, value } => {
                    println!("正在設定設定檔中的值，key: {}, value: {}", key, value);
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
