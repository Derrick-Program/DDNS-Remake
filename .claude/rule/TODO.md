# DDNS-Remake 開發 TODO

> 每次對話開始時，Claude 應讀取此檔並向使用者顯示目前進度與下一步任務。

## 完成度總覽

| 模組 | 完成度 | 狀態 |
|------|--------|------|
| DDNS-Server (DB + ORM) | ~90% | 穩固 |
| DDNS-Server (Auth API) | ~80% | Login + Token Validator 已修復，暫用 X-Device-ID header |
| DDNS-Server (DNS API) | ~30% | PATCH 未實作 |
| DDNS-Server (CLI Commands) | ~40% | 多數 unimplemented |
| DDNS-Client | ~5% | 僅 stub |
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

- [ ] **S2-1** 實作 `PATCH /api/v1/dns_records/{record_id}`，目前回傳 501
  - 檔案：`crates/DDNS-Server/src/apis/v1/mod.rs:39`
- [ ] **S2-2** 整合 Cloudflare Provider 到 DNS 更新流程（程式碼已存在，未接線）
  - 檔案：`crates/DDNS-Server/src/providers/cloudflare.rs`
- [x] **S2-3** CLI `server add-device` / `remove-device` / `list-devices`
- [x] **S2-4** CLI `server add-domain` / `remove-domain` / `list-domains`
- [x] **S2-5** CLI `server delete-user` 加入使用者存在性驗證（已確認早已實作）
- [ ] **S2-6** Config 子指令（generate / get / set / check），目前全部 unimplemented
  - 檔案：`crates/DDNS-Server/src/command/mod.rs:32`

---

## Sprint 3 — DDNS-Client 開發【目前僅有 stub】

- [ ] **S3-1** 讀取設定檔（Server URL + Device Token）
- [ ] **S3-2** 主迴圈：定期偵測 public IP，若有變更才觸發更新
- [ ] **S3-3** 呼叫 `PATCH /api/v1/dns_records/{record_id}` 更新 DNS 記錄
  - 使用 DDNS-Core 的 `UpdateDnsRecordRequest` DTO
- [ ] **S3-4** CLI 參數解析（Clap）
- [ ] **S3-5** systemd service 檔案（daemon 自動啟動）
- [ ] **S3-6** 錯誤重試邏輯（指數退避）

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
