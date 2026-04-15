use std::net::Ipv4Addr;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "DDNS Server 管理工具", long_about = None, propagate_version = true)]
pub struct Cli {
    #[command(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
    /// 設定檔路徑（也可透過 DDNS_CONFIG 環境變數指定）
    #[arg(short, long, env = "DDNS_CONFIG",
          default_value_t = crate::config::default_config_path().to_string_lossy().into_owned())]
    pub config: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 啟動伺服器
    Start {
        /// 指定監聽埠號
        #[arg(short, long, default_value_t = 8698)]
        port: u16,
        #[arg(short = 'H', long, default_value_t = Ipv4Addr::LOCALHOST)]
        host: Ipv4Addr,
    },
    /// 設定檔相關操作
    Config(ConfigArgs),
    ///Server相關操作
    Server(ServerArgs),
    /// 開啟互動式管理介面（TUI）
    Tui,
    /// 退出應用程式
    Exit,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    /// 產生預設設定檔
    Generate {
        /// 是否覆蓋現有檔案
        #[arg(short, long)]
        force: bool,
    },
    /// 讀取指定設定值
    Get {
        key: String,
    },
    /// 設定指定設定值
    Set {
        key: String,
        value: String,
    },
    /// 檢查設定檔是否正確
    Check,
    /// 列出所有已設定的 DNS Zone
    ZoneList,
    /// 新增一個 DNS Zone（如 duacodie.com）
    ZoneAdd {
        /// Zone 名稱，例如 duacodie.com
        name: String,
        /// 預先填入的 Cloudflare Zone ID（選填，留空則在更新時自動查詢）
        #[arg(long)]
        zone_id: Option<String>,
    },
    /// 移除一個 DNS Zone
    ZoneRemove {
        /// Zone 名稱
        name: String,
    },
}

#[derive(Args, Debug)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub action: ServerSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ServerSubcommands {
    ///產生API Key到選擇的地方（如: 輸出到終端、儲存到檔案）
    GenerateApiKey {
        #[arg(short = 'u', long)]
        username: String,
    },
    /// 產生當前機器的 UUID v5（用於測試裝置註冊）
    GenerateDeviceId,
    /// 列出所有使用者
    ListUsers,
    /// 新增使用者
    AddUser {
        #[arg(short = 'u', long)]
        username: String,
        #[arg(short = 'p', long)]
        password: Option<String>,
    },
    /// 移除使用者
    RemoveUser {
        #[arg(short = 'u', long)]
        username: String,
    },
    /// 列出所有裝置
    ListDevices,
    /// 新增裝置
    AddDevice {
        #[arg(short = 'd', long)]
        device_name: String,
        #[arg(short = 'o', long)]
        owner_username: String,
    },
    /// 移除裝置
    RemoveDevice {
        #[arg(short = 'd', long)]
        device_name: String,
    },
    /// 列出域名（不指定裝置則列出全部）
    ListDomains {
        /// 指定裝置名稱，只列出該裝置的域名
        #[arg(short = 'd', long)]
        device_name: Option<String>,
    },
    /// 新增裝置綁定的域名
    AddDomain {
        #[arg(short = 'd', long)]
        device_name: String,
        #[arg(short = 'n', long)]
        domain_name: String,
    },
    /// 移除裝置綁定的域名
    RemoveDomain {
        #[arg(short = 'n', long)]
        domain_name: String,
    },
}