use std::{net::SocketAddr, sync::Arc};

use crate::{
    cli::{Cli, Commands, ConfigSubcommands, ServerSubcommands},
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

#[allow(unused)]
pub async fn handle(cli: Cli, ctx: &Arc<AppState>) -> Result<CommandResult> {
    match &cli.command {
        Commands::Exit => Ok(CommandResult::Exit),

        Commands::Config(config_args) => {
            match &config_args.action {
                ConfigSubcommands::Generate { force, format } => {
                    // println!("正在產生 {} 格式的設定檔 (強制覆蓋: {})", format, force);
                    unimplemented!("ConfigSubcommands::Generate 還未實作");
                }
                ConfigSubcommands::Check => {
                    // println!("正在檢查設定檔：{}", cli.config);
                    unimplemented!("ConfigSubcommands::Check 還未實作");
                }
                ConfigSubcommands::Get { key } => {
                    // println!("正在獲取設定檔中的值，key: {}", key);
                    unimplemented!("ConfigSubcommands::Get 還未實作");
                }
                ConfigSubcommands::Set { key, value } => {
                    // println!("正在設定設定檔中的值，key: {}, value: {}", key, value);
                    unimplemented!("ConfigSubcommands::Set 還未實作");
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
        Commands::Server(server_args) => {
            match &server_args.action {
                ServerSubcommands::GenerateApiKey { username, output } => {
                    println!("正在產生新的 API Key...");
                    unimplemented!("ServerSubcommands::GenerateApiKey 還未實作");
                }
                ServerSubcommands::ListUsers => {
                    println!("正在列出所有使用者...");
                    unimplemented!("ServerSubcommands::ListUsers 還未實作");
                }
                ServerSubcommands::AddUser { username, password } => {
                    println!("正在新增使用者，username: {}, password: {:#?}", username, password);
                    unimplemented!("ServerSubcommands::AddUser 還未實作");
                }
                ServerSubcommands::RemoveUser { username } => {
                    println!("正在移除使用者，username: {}", username);
                    unimplemented!("ServerSubcommands::RemoveUser 還未實作");
                }
                ServerSubcommands::ListDevices => {
                    println!("正在列出所有裝置...");
                    unimplemented!("ServerSubcommands::ListDevices 還未實作");
                }
                ServerSubcommands::AddDomain { device_name, domain_name } => {
                    println!(
                        "正在新增裝置綁定的域名，device_name: {}, domain_name: {}",
                        device_name, domain_name
                    );
                    unimplemented!("ServerSubcommands::AddDomain 還未實作");
                }
                ServerSubcommands::RemoveDomain { domain_name } => {
                    println!("正在移除裝置綁定的域名，domain_name: {}", domain_name);
                    unimplemented!("ServerSubcommands::RemoveDomain 還未實作");
                }
                ServerSubcommands::ListDomains => {
                    println!("正在列出所有裝置綁定的域名...");
                    unimplemented!("ServerSubcommands::ListDomains 還未實作");
                }
                ServerSubcommands::AddDevice { device_name, owner_username } => {
                    println!(
                        "正在新增裝置，device_name: {}, owner_username: {}",
                        device_name, owner_username
                    );
                    unimplemented!("ServerSubcommands::AddDevice 還未實作");
                }
                ServerSubcommands::RemoveDevice { device_name } => {
                    println!("正在移除裝置，device_name: {}", device_name);
                    unimplemented!("ServerSubcommands::RemoveDevice 還未實作");
                }
            }
            Ok(CommandResult::Continue)
        }
    }
}
