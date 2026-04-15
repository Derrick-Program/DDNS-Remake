# DDNS Remake

以 Rust 實作的動態 DNS（DDNS）系統，包含 REST API 伺服器、客戶端守護程式與共用型別函式庫。

## 專案架構

本專案為 Cargo Workspace，包含三個 Crate：

| Crate | 說明 |
|-------|------|
| `crates/DDNS-Server` | REST API 伺服器（Salvo 框架 + Diesel/SQLite ORM） |
| `crates/DDNS-Client` | 客戶端守護程式，定期偵測公開 IP 並觸發 DNS 更新 |
| `crates/DDNS-Core` | 伺服器與客戶端共用的 DTO 型別定義 |

### 請求流程

```
HTTP Request → Salvo Router → Token Validator Middleware → Handler → DbService → SQLite
```

### 資料庫關聯

```
users (1) ──→ (N) devices (1) ──→ (N) domains
```

## 環境需求

- Rust 1.94.0（透過 `rust-toolchain.toml` 鎖定）
- [just](https://github.com/casey/just) 任務執行器
- SQLite

## 安裝

### Linux / macOS

```bash
curl -fsSL -H "Cache-Control: no-cache" -H "Pragma: no-cache" \
  "https://raw.githubusercontent.com/Derrick-Program/DDNS-Remake/main/deploy/install.sh" | sudo bash
```

或下載後執行：

```bash
curl -fsSL -H "Cache-Control: no-cache" -H "Pragma: no-cache" \
  "https://raw.githubusercontent.com/Derrick-Program/DDNS-Remake/main/deploy/install.sh" -o install.sh
chmod +x install.sh
sudo ./install.sh
```

安裝腳本啟動後會顯示互動式選單：

```
╔════════════════════════════════════╗
║   DDNS Remake Installer vX.Y.Z    ║
╚════════════════════════════════════╝

  [1] Install DDNS Server
  [2] Install DDNS Client
  [3] Uninstall DDNS Remake
  [4] Exit
```

安裝路徑：`/opt/duacodie/`  
設定檔：`/opt/duacodie/config/duacodie/`  
Service 使用者：`duacodie`（系統帳號，無登入權限）

> 若需指定版本：`DDNS_VERSION=v0.1.1 sudo ./install.sh`

**Server 安裝後**（Linux systemd）：

```bash
systemctl status duacodie-server
journalctl -u duacodie-server -f
```

**Client 安裝後**（Linux systemd）：

```bash
systemctl status duacodie-client
journalctl -u duacodie-client -f
```

**macOS**（launchd）：

```bash
# 查看服務狀態
sudo launchctl print system/com.duacodie.server
sudo launchctl print system/com.duacodie.client

# 或快速確認是否執行中
sudo launchctl list | grep duacodie

# 查看 log
tail -f /var/log/duacodie/ddns-server.log
tail -f /var/log/duacodie/ddns-client.log
```

### Windows

以系統管理員身份開啟 PowerShell，執行：

```powershell
irm -Headers @{"Cache-Control"="no-cache";"Pragma"="no-cache"} `
  "https://raw.githubusercontent.com/Derrick-Program/DDNS-Remake/main/deploy/install.ps1" | iex
```

或下載後執行：

```powershell
Invoke-WebRequest -Headers @{"Cache-Control"="no-cache";"Pragma"="no-cache"} `
  -Uri "https://raw.githubusercontent.com/Derrick-Program/DDNS-Remake/main/deploy/install.ps1" -OutFile install.ps1
.\install.ps1
```

安裝路徑：`C:\Program Files\DDNS\`  
Windows 服務名稱：`DuacodieServer` / `DuacodieClient`

```powershell
# 查看服務狀態
Get-Service DuacodieServer
Get-Service DuacodieClient
```

> 若需指定版本：`$env:DDNS_VERSION = "v0.1.1"; .\install.ps1`

### 解除安裝

**Linux / macOS**：

```bash
sudo ./install.sh
# 選擇 [3] Uninstall DDNS Remake
```

此操作會：
- 停止並移除 systemd / launchd 服務
- 刪除 `/opt/duacodie/` 及 `/var/log/duacodie/`（含資料庫與設定檔）

**Windows**：

```powershell
.\install.ps1
# 選擇 [3] Uninstall DDNS Remake
```

此操作會：
- 停止並刪除 `DuacodieServer` / `DuacodieClient` 服務
- 刪除 `C:\Program Files\DDNS\`
- 從系統 PATH 移除安裝路徑

---

## 從原始碼建置

### 1. 設定環境變數

複製 `.env.example` 或在根目錄建立 `.env`：

```bash
DATABASE_URL=ddns.db
```

### 2. 建置

```bash
just build-server          # Release 建置
just build-server --debug  # Debug 建置
just build-client          # 建置客戶端
```

### 3. 產生設定檔

```bash
./ddns-server config generate   # 產生預設 config.toml
./ddns-server config check       # 檢查設定是否完整
```

`config.toml` 範例：

```toml
[server]
host = "127.0.0.1"
port = 8698

[cloudflare]
api_key = "your-cloudflare-api-key"
```

### 4. 新增使用者

```bash
./ddns-server server add-user <username>
```

### 5. 啟動伺服器

```bash
just run-server start        # 以 config.toml 設定啟動
just run-server              # 進入互動式 REPL 模式
```

## API 端點

### 認證端點（`/api/auth`）

| 方法 | 路徑 | 說明 |
|------|------|------|
| `POST` | `/api/auth/login` | 密碼登入，回傳 JWT Token |
| `GET` | `/api/auth/profile` | 取得目前登入使用者資訊 |
| `GET` | `/api/auth/is_login` | 確認 JWT 是否有效 |
| `POST` | `/api/auth/devices` | 註冊新裝置，取得 API Key |
| `DELETE` | `/api/auth/devices/{device_name}` | 移除裝置 |

### DNS 端點（`/api/v1`，需裝置 Token 認證）

| 方法 | 路徑 | 說明 |
|------|------|------|
| `GET` | `/api/v1/dns_records/{deviceid}` | 取得裝置所有活躍域名及目前 IP |
| `PATCH` | `/api/v1/dns_records/{deviceid}` | 更新裝置所有域名的 IP（同步至 Cloudflare） |

### Token 認證階層

1. `POST /api/auth/login` — 密碼驗證 → JWT（使用者層級，有效期 5 分鐘）
2. `POST /api/auth/devices` — JWT 驗證 → 裝置 API Token（裝置層級，用於 DNS 更新）

### 登入範例

```bash
# 1. 取得 JWT
curl -X POST http://127.0.0.1:8698/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "yourpassword"}'

# 2. 用 JWT 註冊裝置，取得 API Key
curl -X POST http://127.0.0.1:8698/api/auth/devices \
  -H "Authorization: Bearer <JWT>" \
  -H "Content-Type: application/json" \
  -d '{"device_name": "my-server", "device_id": "<UUID>"}'

# 3. 用 API Key 更新 DNS
curl -X PATCH http://127.0.0.1:8698/api/v1/dns_records/<device-uuid> \
  -H "Authorization: Bearer <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"Ip": "1.2.3.4"}'
```

## CLI 管理指令（REPL 模式）

進入 REPL：

```bash
just run-server
```

可用指令：

```
# 使用者管理
server add-user <username>
server remove-user <username>
server list-users

# 裝置管理
server add-device <device_name> --owner <username>
server remove-device <device_name>
server list-devices

# 域名管理
server add-domain <device_name> <hostname>
server remove-domain <hostname>
server list-domains

# 設定管理
config generate
config check
config get <key>           # e.g. config get cloudflare.api_key
config set <key> <value>   # e.g. config set server.port 9000
```

## 資料庫遷移

```bash
just migration-run              # 套用待執行的遷移
just migration-revert           # 回滾最後一次遷移
just migration-redo             # 回滾後重新套用
just migration-generate <name>  # 產生新的遷移檔
just migration-list             # 列出所有遷移狀態
```

遷移檔以 `diesel_migrations!` 嵌入二進位，伺服器啟動時自動執行。

## 測試

```bash
cargo test                                               # 執行所有測試
cargo test -p ddns-server                               # 僅測試 Server Crate
cargo test test_create_user                             # 執行指定測試
cargo test -p ddns-server --lib -- --nocapture          # 顯示 println! 輸出
```

測試使用記憶體內 SQLite（`file:memdb{uuid}?mode=memory&cache=shared`），每個測試均建立獨立的 `DbService` 實例。

## 程式碼品質

```bash
cargo clippy -- -D warnings   # 靜態分析（警告視為錯誤）
cargo fmt                      # 格式化程式碼
```

程式碼風格設定於 `rustfmt.toml`：行寬 100 字元、block indent 風格、依 crate 分組 import。

## DNS Provider 抽象化

`crates/DDNS-Server/src/providers/` 定義兩個 Trait：

- `ZoneHandler` — 列出 DNS Zone
- `RecordHandler` — 查詢、新增、更新、刪除 DNS 記錄

透過 `DnsFactory` 建立 Provider 實例。目前實作：

- **Cloudflare**（`providers/cloudflare.rs`）

新增其他 Provider（如 Cloudns、Duck DNS）只需實作上述兩個 Trait。

## OpenAPI / Swagger

Debug 建置自動產生 Swagger UI，可在以下位址存取：

```
http://127.0.0.1:8698/api/swagger-ui/
```

## 專案狀態

| 模組 | 完成度 |
|------|--------|
| DDNS-Server（DB + ORM） | ~90% |
| DDNS-Server（Auth API） | ~85% |
| DDNS-Server（DNS API）  | ~80% |
| DDNS-Server（CLI）      | ~70% |
| DDNS-Client             | ~5%（開發中） |
| DDNS-Core（DTOs）       | ~85% |
