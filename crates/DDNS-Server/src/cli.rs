use std::net::Ipv4Addr;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use nanoid::nanoid;

fn generate_api_key() -> String {
    let token = nanoid!(45);
    format!("ddns_tok_{}", token)
}

fn hash_token(token: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(token.as_bytes(), &salt).unwrap().to_string()
}

fn verify_client_token(db_hash: &str, provided_token: &str) -> bool {
    let parsed_hash = match PasswordHash::new(db_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };
    let argon2 = Argon2::default();
    argon2.verify_password(provided_token.as_bytes(), &parsed_hash).is_ok()
}

pub fn generate_and_print_api_key() {
    let api_key = generate_api_key();
    println!("Generated API Key: {}", api_key);
    let db_token = hash_token(&api_key);
    println!("Hashed API Key for DB storage: {}", db_token);
    let is_valid = verify_client_token(&db_token, &api_key);
    println!("Token verification result: {}", is_valid);
    let host_uuid = uuid::Uuid::new_v4();
    println!("Generated Host UUID: {}", host_uuid);
}

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
        /// 輸出的格式 (如: toml, json, yaml)
        #[arg(short, long, default_value = "toml")]
        format: String,
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
