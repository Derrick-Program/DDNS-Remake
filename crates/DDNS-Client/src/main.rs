use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
    /// 設定檔管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
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
