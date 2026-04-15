# DDNS-Remake 開發 TODO

> 每次對話開始時，Claude 應讀取此檔並向使用者顯示目前進度與下一步任務。

## 完成度總覽

| 模組 | 完成度 | 狀態 |
|------|--------|------|
| DDNS-Server (DB + ORM) | ~90% | 穩固 |
| DDNS-Server (Auth API) | ~85% | Login + Token Validator 已修復，改用 path param 識別裝置 |
| DDNS-Server (DNS API) | ~80% | PATCH 已實作並整合 Cloudflare |
| DDNS-Server (CLI Commands) | ~70% | Config 子指令已實作 |
| DDNS-Client | ~80% | Config / HTTP Client / Daemon / CLI 已完成 |
| DDNS-Core (DTOs) | ~85% | 基本完整 |

---

## Sprint 1 — 核心安全漏洞修復【最高優先，請先完成】

- [x] **S1-1** 修復 Login 端點：從 DB 查詢使用者並用 Argon2 驗證密碼，移除 hardcoded `user_id = 9527`
  - 檔案：`crates/DDNS-Server/src/apis/auth/mod.rs:22`
- [x] **S1-2** 修復 Token Validator：用 Argon2 verify 比對 DB 中的 `token_hash`，目前只檢查 prefix
  - 檔案：`crates/DDNS-Server/src/middlewares/token_validator.rs:9`
  - ⚠️ **暫用方案 A**：device 識別透過 `X-Device-ID` header，待 S2-1 改 PATCH route 為 `/{deviceid}` 後應改為方案 B（從 path 取 device_id，移除 header 依賴）
- [~] **S1-3** ~~API Key 持久化：`generate-api-key` CLI 寫入 `devices.token_hash`~~
  - 已由 `POST /api/auth/devices` 取代，CLI 版本僅供測試用，暫不實作

---

## Sprint 2 — Server 業務邏輯

- [x] **S2-1** 實作 `PATCH /api/v1/dns_records/{deviceid}`，路由已合併（GET+PATCH 同路徑）
  - token_validator 改為方案 B（從 path param 取 deviceid，移除 X-Device-ID header 依賴）
- [x] **S2-2** 整合 Cloudflare Provider 到 DNS 更新流程
  - 查 Zone → 查 Record → 更新 Cloudflare → 寫回 DB `current_ip`
- [x] **S2-3** CLI `server add-device` / `remove-device` / `list-devices`
- [x] **S2-4** CLI `server add-domain` / `remove-domain` / `list-domains`
- [x] **S2-5** CLI `server delete-user` 加入使用者存在性驗證（已確認早已實作）
- [x] **S2-6** Config 子指令（generate / get / set / check）
  - 新增 `src/config.rs`（AppConfig TOML 結構），AppState 加入 config 欄位
  - generate: 產生預設 config.toml；check: 驗證設定；get/set: 讀寫指定 key

---

## Sprint 3 — DDNS-Client 開發【目前僅有 stub】

- [x] **S3-1** 讀取設定檔（Server URL + Device Token）
  - `crates/DDNS-Client/src/config.rs`：`ClientConfig`，讀寫 `~/.config/ddns-client/config.toml`
- [x] **S3-2** 主迴圈：定期偵測 public IP，若有變更才觸發更新
  - `crates/DDNS-Client/src/daemon.rs`
- [x] **S3-3** 呼叫 `PATCH /api/v1/dns_records/{device_id}` 更新 DNS 記錄
  - `crates/DDNS-Client/src/client.rs`：`DdnsClient`
  - 同時補上 `UpdateDnsRecordRequest` 的 `Serialize` derive（DDNS-Core）
- [x] **S3-4** CLI 參數解析（Clap）
  - `crates/DDNS-Client/src/main.rs`：`run` / `check` / `config init` / `config path`
- [x] **S3-5** systemd service 檔案（daemon 自動啟動）
  - `deploy/ddns-client.service`
- [x] **S3-6** 錯誤重試邏輯（指數退避）
  - 內建於 `daemon.rs`：5s → 10s → 20s → ... → 300s 上限

---

## Sprint 4 — 品質補強

- [ ] **S4-1** Auth 端點整合測試（login / exchange / profile）
- [ ] **S4-2** Token validator 單元測試
- [ ] **S4-3** DNS PATCH 端點整合測試
- [ ] **S4-4** 補全 Swagger / OpenAPI 文件（所有 endpoint 加描述）
- [ ] **S4-5** 結構化日誌：API request/response tracing spans

---

## 已完成

- [x] 資料庫 schema 設計（users / devices / domains）
- [x] Diesel ORM + r2d2 連線池
- [x] 資料庫 Migration 自動嵌入
- [x] DbService CRUD（User / Device / Domain）
- [x] JWT middleware 架構
- [x] Salvo router 結構
- [x] Cloudflare DNS Provider（trait + 實作）
- [x] Argon2 密碼雜湊工具
- [x] UUID v5 device identifier 生成
- [x] DB 層 11 個單元測試
- [x] 優雅關機（SIGTERM / Ctrl+C）
- [x] DDNS-Core DTOs 定義
