use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub cloudflare: CloudflareConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { host: "127.0.0.1".to_string(), port: 8698 }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CloudflareConfig {
    #[serde(default)]
    pub api_key: String,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &str) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "server.host" => Some(self.server.host.clone()),
            "server.port" => Some(self.server.port.to_string()),
            "cloudflare.api_key" => Some(self.cloudflare.api_key.clone()),
            _ => None,
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "server.host" => self.server.host = value.to_string(),
            "server.port" => {
                self.server.port = value
                    .parse::<u16>()
                    .map_err(|_| anyhow::anyhow!("server.port 須為有效埠號 (0-65535)"))?;
            }
            "cloudflare.api_key" => self.cloudflare.api_key = value.to_string(),
            _ => {
                return Err(anyhow::anyhow!(
                    "未知的設定鍵: '{}'. 可用鍵: server.host, server.port, cloudflare.api_key",
                    key
                ))
            }
        }
        Ok(())
    }

    /// 檢查設定完整性，回傳警告訊息列表
    pub fn check(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.cloudflare.api_key.is_empty() {
            warnings.push("cloudflare.api_key 未設定".to_string());
        }
        warnings
    }
}
