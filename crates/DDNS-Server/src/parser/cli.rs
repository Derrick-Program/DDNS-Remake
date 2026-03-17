use std::net::Ipv4Addr;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "DDNS Server 管理工具", long_about = None, propagate_version = true)]
pub struct Cli {
    #[command(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
    #[arg(short, long, default_value = "config.toml")]
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
    /// 資料庫相關操作
    Database(DbArgs),
    
    ///Server相關操作
    Server(ServerArgs),
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
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
    },
    /// 檢查設定檔是否正確
    Check,
}

#[derive(Args, Debug)]
pub struct DbArgs {
    #[command(subcommand)]
    pub action: DbSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum DbSubcommands {}

#[derive(Args, Debug)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub action: ServerSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ServerSubcommands {
    ///產生API Key到選擇的地方（如: 輸出到終端、儲存到檔案）
    GenerateApiKey {
        #[arg(short, long)]
        username: String,
    },
    /// 列出所有使用者
    ListUsers,
    /// 新增使用者
    AddUser {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// 移除使用者
    RemoveUser {
        #[arg(short, long)]
        username: String,
    },
    /// 列出所有裝置
    ListDevices,
    /// 新增裝置
    AddDevice {
        #[arg(short, long)]
        device_name: String,
        #[arg(short, long)]
        owner_username: String,
    },
    /// 移除裝置
    RemoveDevice {
        #[arg(short, long)]
        device_name: String,
    },
    /// 列出所有裝置綁定的域名
    ListDomains,
    /// 新增裝置綁定的域名
    AddDomain {
        #[arg(short, long)]
        device_name: String,
        #[arg(short, long)]
        domain_name: String,
    },
    /// 移除裝置綁定的域名
    RemoveDomain {
        #[arg(short, long)]
        domain_name: String,
    },
}