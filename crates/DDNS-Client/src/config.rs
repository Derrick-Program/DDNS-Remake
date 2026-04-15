use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    pub device_token: String,
    #[serde(default = "default_check_interval")]
    pub check_interval_secs: u64,
}

fn default_check_interval() -> u64 {
    60
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:8698".to_string(),
            device_token: "your-api-token-here".to_string(),
            check_interval_secs: 60,
        }
    }
}

impl ClientConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("duacodie")
            .join("ddns-client")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("無法讀取設定檔：{}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("設定檔格式錯誤：{}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("無法建立設定目錄：{}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("序列化設定失敗")?;
        std::fs::write(&path, content)
            .with_context(|| format!("無法寫入設定檔：{}", path.display()))
    }
}
