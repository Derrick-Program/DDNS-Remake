mod server;
mod start;
pub mod utils;
use std::{net::SocketAddr, path::Path, sync::Arc};

use crate::{
    command,
    parser::cli::{Cli, Commands, ConfigSubcommands, ServerSubcommands},
};
use comfy_table::{ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use anyhow::Result;
use tracing::{error, info};

pub enum CommandResult {
    Continue,
    Exit,
}

#[allow(unused)]
pub struct AppState {
    pub db_service: crate::db::DbService,
    pub config: crate::config::AppConfig,
    pub config_path: String,
}

#[allow(unused)]
pub async fn handle(cli: Cli, ctx: &Arc<AppState>) -> Result<CommandResult> {
    match &cli.command {
        Commands::Exit => Ok(CommandResult::Exit),

        Commands::Config(config_args) => {
            let path = cli.config.as_str();
            match &config_args.action {
                ConfigSubcommands::Generate { force } => {
                    if !force && Path::new(path).exists() {
                        error!("設定檔 '{}' 已存在，使用 --force 強制覆蓋", path);
                        return Ok(CommandResult::Continue);
                    }
                    let config = crate::config::AppConfig::default();
                    config.save(path)?;
                    info!("已產生預設設定檔: {}", path);
                }
                ConfigSubcommands::Check => {
                    match crate::config::AppConfig::load(path) {
                        Err(e) => error!("設定檔 '{}' 讀取失敗: {}", path, e),
                        Ok(config) => {
                            let issues = config.check();
                            if issues.is_empty() {
                                info!("設定檔 '{}' 驗證通過", path);
                            } else {
                                for issue in &issues {
                                    error!("警告: {}", issue);
                                }
                            }
                        }
                    }
                }
                ConfigSubcommands::Get { key } => {
                    match ctx.config.get_value(key) {
                        Some(v) => info!("{} = {}", key, v),
                        None => error!(
                            "未知的設定鍵: '{}'. 可用鍵: server.host, server.port, cloudflare.api_key",
                            key
                        ),
                    }
                }
                ConfigSubcommands::Set { key, value } => {
                    let mut config = crate::config::AppConfig::load_or_default(path);
                    config.set_value(key, value)?;
                    config.save(path)?;
                    info!("已設定 {} = {}", key, value);
                }
                ConfigSubcommands::ZoneList => {
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .set_header(vec!["Zone Name", "Zone ID"]);
                    for zone in &ctx.config.zones {
                        table.add_row(vec![
                            zone.name.clone(),
                            zone.zone_id.clone().unwrap_or_else(|| "(auto)".to_string()),
                        ]);
                    }
                    info!("已設定的 DNS Zone：\n{table}");
                }
                ConfigSubcommands::ZoneAdd { name, zone_id } => {
                    let mut config = crate::config::AppConfig::load_or_default(path);
                    if config.add_zone(name, zone_id.clone()) {
                        config.save(path)?;
                        info!("已新增 Zone '{}'", name);
                    } else {
                        error!("Zone '{}' 已存在", name);
                    }
                }
                ConfigSubcommands::ZoneRemove { name } => {
                    let mut config = crate::config::AppConfig::load_or_default(path);
                    if config.remove_zone(name) {
                        config.save(path)?;
                        info!("已移除 Zone '{}'", name);
                    } else {
                        error!("Zone '{}' 不存在", name);
                    }
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
                    command::server::add_domain(device_name, domain_name, ctx)?;
                }
                ServerSubcommands::RemoveDomain { domain_name } => {
                    command::server::remove_domain(domain_name, ctx)?;
                }
                ServerSubcommands::ListDomains { device_name } => {
                    command::server::list_domains(device_name.as_deref(), ctx)?;
                }
                ServerSubcommands::AddDevice { device_name, owner_username } => {
                    command::server::add_device(device_name, owner_username, ctx)?;
                }
                ServerSubcommands::RemoveDevice { device_name } => {
                    command::server::remove_device(device_name, ctx)?;
                }
            }
            Ok(CommandResult::Continue)
        }
    }
}
