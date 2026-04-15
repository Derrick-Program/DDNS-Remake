mod apis;
mod config;
mod parser;
mod command;
mod db;
mod middlewares;
mod models;
mod providers;
mod schema;
mod tui;

use std::sync::Arc;

use anyhow::Result;
mod error;
mod translate;
use clap::Parser;
use diesel::{
    RunQueryDsl, SqliteConnection,
    r2d2::{ConnectionManager, CustomizeConnection, Pool},
    sql_query,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::{
    command::{CommandResult, handle},
    db::DbService,
};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
#[derive(Debug)]
pub struct SqliteCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        sql_query("PRAGMA foreign_keys = ON;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }

    fn on_release(&self, _conn: SqliteConnection) {}
}

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

fn init_tracing(verbosity: &str) -> Result<()> {
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true);

    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(verbosity));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(stdout_layer)
        .try_init()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("需設定 DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().connection_customizer(Box::new(SqliteCustomizer)).build(manager)?;
    {
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!("資料庫遷移失敗: {}", e))?;
    }
    let db_service = DbService::new(pool);
    if std::env::args_os().len() > 1 {
        let cli = parser::cli::Cli::parse();
        init_tracing(&cli.verbosity.to_string())?;
        let config = config::AppConfig::load_or_default(&cli.config);
        let config_path = cli.config.clone();
        let ctx = Arc::new(command::AppState { db_service, config, config_path });
        match handle(cli, &ctx).await? {
            CommandResult::Continue | CommandResult::Exit => {}
        }
    } else {
        init_tracing("info")?;
        let config_path = std::env::var("DDNS_CONFIG")
            .unwrap_or_else(|_| config::default_config_path().to_string_lossy().into_owned());
        let config = config::AppConfig::load_or_default(&config_path);
        let ctx = Arc::new(command::AppState {
            db_service,
            config,
            config_path,
        });
        parser::repl::run(&ctx).await?;
    }
    
    Ok(())
}
