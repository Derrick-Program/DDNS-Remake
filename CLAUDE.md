# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DDNS Remake is a Rust workspace implementing a Dynamic DNS system with three crates:
- **`crates/DDNS-Server`** — REST API server (Salvo web framework + Diesel/SQLite ORM)
- **`crates/DDNS-Client`** — Client daemon that fetches public IP and updates DNS records
- **`crates/DDNS-Core`** — Shared DTOs and type definitions used by both crates

## Build & Run

```bash
just build-server          # Release build (ddns-server, default --release)
just build-client          # Release build (ddns-client, default --release)
just build-server --debug  # Debug build

just run-server            # Interactive REPL mode
just run-server start      # Start HTTP server (default: 127.0.0.1:8698)
just run-client            # Run client
```

## Testing

```bash
cargo test                           # All tests
cargo test -p ddns-server            # Server crate only
cargo test test_create_user          # Single test by name
cargo test -p ddns-server --lib -- --nocapture  # Show println output
```

Tests use in-memory SQLite with shared cache URIs (`file:memdb{uuid}?mode=memory&cache=shared`) — each test creates an isolated `DbService` via `setup_test_service()` in `db.rs`.

## Lint & Format

```bash
cargo clippy -- -D warnings
cargo fmt
```

## Database Migrations

```bash
just migration-run              # Apply pending
just migration-revert           # Revert last
just migration-redo             # Revert then re-apply
just migration-generate <name>  # Generate new migration
just migration-list             # List all migrations
```

Migrations are embedded into the binary via `diesel_migrations!` and run automatically on startup. The `.env` file at the workspace root sets `DATABASE_URL`.

## Architecture

### Request Flow

```
HTTP Request → Salvo Router → Token Validator Middleware → Handler → DbService → SQLite
```

### Auth Token Hierarchy

1. `POST /api/auth/login` — password auth → JWT (user-level)
2. `GET /api/auth/exchange` — JWT → device API token (device-level, used for DNS updates)

V1 DNS endpoints (`/api/v1/dns_records/{deviceid}`) authenticate via device token, not user JWT.

### Key Files

| Path | Purpose |
|------|---------|
| `crates/DDNS-Server/src/db.rs` | `DbService` — all database CRUD, connection pool, test helpers |
| `crates/DDNS-Server/src/apis/` | Route handlers: `auth/` and `v1/` |
| `crates/DDNS-Server/src/middlewares/` | JWT validator, user claims |
| `crates/DDNS-Server/src/command/start.rs` | Server startup, graceful shutdown |
| `crates/DDNS-Server/src/providers/` | Trait-based DNS provider abstraction (Cloudflare implemented) |
| `crates/DDNS-Server/src/parser/` | Clap CLI + REPL command parsing |
| `crates/DDNS-Server/migrations/` | Diesel SQL migration files |
| `crates/DDNS-Core/src/lib.rs` | Shared request/response DTOs |

### DNS Provider Abstraction

`src/providers/` exposes two traits — `ZoneHandler` and `RecordHandler` — plus a `DnsFactory`. Currently only Cloudflare is implemented. New providers implement these traits.

### Database Schema

- `users` → `devices` (1:many, FK `user_id`) → `domains` (1:many, FK `device_id`)
- Foreign keys enforced via r2d2 connection customizer (`PRAGMA foreign_keys = ON`)
- Device lookup is indexed on `device_identifier` (UUID v5)

### OpenAPI / Swagger

Auto-generated from Salvo endpoint macros; available at `/api/swagger-ui/` in debug builds only.

## Toolchain

Pinned via `rust-toolchain.toml` to Rust **1.94.0** with clippy, rustfmt, rust-src, rust-analyzer.  
`rustfmt.toml`: 100-char line width, block indent style, imports grouped by crate.
