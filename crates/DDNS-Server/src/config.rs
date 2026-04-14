use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub cloudflare: CloudflareConfig,
    #[serde(default)]
    pub zones: Vec<ZoneConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZoneConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
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

/// 獲取預設設定檔路徑（符合 XDG 規範）
pub fn default_config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    base.join("duacodie").join("ddns").join("config.toml")
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        config.apply_env_overrides();
        Ok(config)
    }

    pub fn load_or_default(path: &str) -> Self {
        match Self::load(path) {
            Ok(config) => config,
            Err(_) => {
                let mut config = Self::default();
                config.apply_env_overrides();
                config
            }
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("DDNS_SERVER_HOST") {
            self.server.host = v;
        }

        if let Some(port) =
            std::env::var("DDNS_SERVER_PORT").ok().and_then(|v| v.parse::<u16>().ok())
        {
            self.server.port = port;
        }

        if let Ok(v) = std::env::var("CLOUDFLARE_API_KEY") {
            self.cloudflare.api_key = v;
        }

        if self.zones.is_empty()
            && let Ok(v) = std::env::var("DDNS_ZONES")
        {
            self.zones = v
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|name| ZoneConfig { name: name.to_string(), zone_id: None })
                .collect();
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        if let Some(parent) = std::path::Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
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
                    "未知的設定鍵: '{}'。可用鍵: server.host, server.port, cloudflare.api_key",
                    key
                ));
            }
        }
        Ok(())
    }

    pub fn add_zone(&mut self, name: &str, zone_id: Option<String>) -> bool {
        if self.zones.iter().any(|z| z.name == name) {
            return false;
        }
        self.zones.push(ZoneConfig { name: name.to_string(), zone_id });
        true
    }

    pub fn remove_zone(&mut self, name: &str) -> bool {
        let before = self.zones.len();
        self.zones.retain(|z| z.name != name);
        self.zones.len() < before
    }

    pub fn check(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.cloudflare.api_key.is_empty() {
            warnings.push("cloudflare.api_key 未設定".to_string());
        }
        if self.zones.is_empty() {
            warnings.push(
                "zones 未設定，無 DNS zone 可管理（提示：可用 config zone-add 新增）".to_string(),
            );
        }
        warnings
    }
}
