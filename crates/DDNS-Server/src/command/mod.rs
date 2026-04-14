mod server;
mod start;
pub mod utils;
use std::{net::SocketAddr, sync::Arc};

use crate::{
    command,
    parser::cli::{Cli, Commands, ConfigSubcommands, ServerSubcommands},
};
use anyhow::Result;
use tracing::{error, info};

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
                ConfigSubcommands::Generate { force } => {
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
            start::start_server(ctx.clone(), sl).await?;
            Ok(CommandResult::Continue)
        }

        Commands::Database(db_args) => {
            println!("資料庫相關操作: {:#?}", db_args);
            Ok(CommandResult::Continue)
        }
        Commands::Server(server_args) => {
            match &server_args.action {
                ServerSubcommands::GenerateApiKey { username } => {
                    //TODO: 要先檢查使用者是否存在，之後在將產出的deviceKey寫入device資料庫
                    //TODO: 這裡可能就是當測試時使用，因為正式的時候沒有需要手動產生
                    command::server::generate_api_key(username, ctx)?;
                }
                ServerSubcommands::GenerateDeviceId => {
                    match ddns_core::get_device_id() {
                        Ok(uuid) => info!("此機器的 Device UUID v5: {}", uuid),
                        Err(e) => error!("無法產生 Device UUID: {}", e),
                    }
                }
                ServerSubcommands::ListUsers => {
                    command::server::list_users(ctx)?;
                }
                ServerSubcommands::AddUser { username, password } => {
                    let password = match password {
                        Some(p) => p.to_string(),
                        None => {
                            print!("請輸入密碼: ");
                            use std::io::{self, Write};
                            io::stdout().flush()?;
                            rpassword::read_password()?
                        }
                    };
                    command::server::add_user(username, &password, ctx)?;
                }
                ServerSubcommands::RemoveUser { username } => {
                    command::server::remove_user(username, ctx)?;
                }
                ServerSubcommands::ListDevices => {
                    command::server::list_device(ctx)?;
                }
                ServerSubcommands::AddDomain { device_name, domain_name } => {
                    println!(
                        "正在新增裝置綁定的域名，device_name: {}, domain_name: {}",
                        device_name, domain_name
                    );
                    //TODO: 將輸入的裝置名稱及域名做檢查之後寫入
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
