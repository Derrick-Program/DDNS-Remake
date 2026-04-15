use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use ddns_client::client::{fetch_domains, login, register_device};
use ddns_client::config::ClientConfig;
use ddns_client::{daemon, get_public_ip};

#[derive(Parser)]
#[command(name = "ddns-client", about = "DDNS Client daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 啟動 DDNS daemon（持續偵測 IP 並更新 DNS）
    Run,
    /// 一次性偵測目前 public IP（不更新 DNS）
    Check,
    /// 認證管理
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// 設定檔管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// 登入伺服器並自動將 device_token 寫入設定檔
    Login {
        /// 伺服器位址（未指定時從設定檔讀取）
        #[arg(long, short)]
        server: Option<String>,
        /// 裝置名稱（預設使用主機名稱）
        #[arg(long, short = 'n')]
        device_name: Option<String>,
    },
    /// 重新選擇此裝置要更新的域名（從伺服器取得最新清單）
    SelectDomains,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// 產生預設設定檔
    Init,
    /// 顯示設定檔路徑
    Path,
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdout_layer = fmt::layer().with_target(true).with_thread_ids(true);
    let filter_layer =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info")).unwrap();
    tracing_subscriber::registry().with(filter_layer).with(stdout_layer).init();

    let cli = Cli::parse();

    match cli.command {
        Command::Run => {
            let config = ClientConfig::load()?;
            daemon::run(config).await?;
        }
        Command::Check => {
            let ip = get_public_ip().await?;
            info!("目前 public IP：{ip}");
            println!("{ip}");
        }
        Command::Auth { action } => match action {
            AuthAction::Login { server, device_name } => {
                handle_login(server, device_name).await?;
            }
            AuthAction::SelectDomains => {
                handle_select_domains().await?;
            }
        },
        Command::Config { action } => match action {
            ConfigAction::Init => {
                let config = ClientConfig::default();
                config.save()?;
                let path = ClientConfig::config_path();
                println!("已產生預設設定檔：{}", path.display());
                println!("請編輯 server_url 與 device_token 後再執行 run");
            }
            ConfigAction::Path => {
                println!("{}", ClientConfig::config_path().display());
            }
        },
    }

    Ok(())
}

async fn handle_select_domains() -> Result<()> {
    let mut config = ClientConfig::load().context("請先執行 auth login 完成裝置註冊")?;

    let device_id = ddns_core::get_device_id()
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string();

    println!("正在從 {} 取得域名清單 ...", config.server_url);
    let available_domains = fetch_domains(&config.server_url, &config.device_token, &device_id).await?;

    let selected_domains = if available_domains.is_empty() {
        println!("此裝置尚無已設定的域名，請由伺服器管理員新增後再執行此指令。");
        return Ok(());
    } else {
        let items: Vec<(&str, &str, bool)> = available_domains
            .iter()
            .map(|h| (h.as_str(), h.as_str(), config.domains.contains(h)))
            .collect();

        cliclack::multiselect("選擇要讓此裝置更新的域名（空白鍵選取，Enter 確認）：")
            .items(&items)
            .interact()?
            .into_iter()
            .map(|s: &str| s.to_string())
            .collect()
    };

    config.domains = selected_domains;
    config.save()?;

    let path = ClientConfig::config_path();
    println!("已將域名設定寫入：{}", path.display());

    Ok(())
}

async fn handle_login(server_arg: Option<String>, device_name_arg: Option<String>) -> Result<()> {
    let server_url = match server_arg {
        Some(url) => url,
        None => {
            ClientConfig::load()
                .context("請用 --server 指定伺服器位址，或先執行 config init 建立設定檔")?
                .server_url
        }
    };

    let device_name = device_name_arg
        .or_else(|| hostname::get().ok()?.into_string().ok())
        .unwrap_or_else(|| "ddns-device".to_string());

    let device_id = ddns_core::get_device_id()
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string();

    let username = {
        print!("使用者名稱：");
        use std::io::{Write, stdin, stdout};
        stdout().flush()?;
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        buf.trim().to_string()
    };
    let password = rpassword::prompt_password("密碼：").context("無法讀取密碼")?;

    println!("正在登入 {server_url} ...");
    let jwt = login(&server_url, &username, &password).await?;
    println!("登入成功");

    println!("正在註冊裝置 \"{device_name}\" (device_id: {device_id}) ...");
    let api_key = register_device(&server_url, &jwt, &device_name, &device_id).await?;
    println!("裝置註冊成功");

    // 取得此裝置可更新的域名清單
    let available_domains = fetch_domains(&server_url, &api_key, &device_id).await?;

    let selected_domains = if available_domains.is_empty() {
        println!("此裝置尚無已設定的域名，稍後可由伺服器管理員新增後重新設定。");
        Vec::new()
    } else {
        let items: Vec<(&str, &str, bool)> = available_domains
            .iter()
            .map(|h| (h.as_str(), h.as_str(), true))
            .collect();

        cliclack::multiselect("選擇要讓此裝置更新的域名（空白鍵選取，Enter 確認）：")
            .items(&items)
            .interact()?
            .into_iter()
            .map(|s: &str| s.to_string())
            .collect()
    };

    let mut config = ClientConfig::load().unwrap_or_default();
    config.server_url = server_url;
    config.device_token = api_key;
    config.domains = selected_domains;
    config.save()?;

    let path = ClientConfig::config_path();
    println!("已將設定寫入：{}", path.display());

    Ok(())
}
