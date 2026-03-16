mod apis;
mod cli;
mod db;
mod middleware;
mod models;
mod providers;
mod schema;
mod server;
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
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{cli::{Commands, ConfigSubcommands}, db::DbService};
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
#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let stdout_layer = fmt::layer().with_target(true).with_thread_ids(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cli.verbosity.to_string()));
    tracing_subscriber::registry().with(filter_layer).with(stdout_layer).init();
    info!("DDNS Server is initializing...");
    debug!("Read .env file");
    dotenvy::dotenv().ok();
    debug!("Environment variables loaded: {:#?}", std::env::var("DATABASE_URL"));
    info!("Initializing database connection pool");
    let db_url = std::env::var("DATABASE_URL").expect("需設定 DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().connection_customizer(Box::new(SqliteCustomizer)).build(manager)?;
    info!("Running database migrations");
    {
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!("資料庫遷移失敗: {}", e))?;
    }
    debug!("Database connection pool established and migrations applied");
    debug!("Service dependencies initialized");
    let db_service = DbService::new(pool);
    let app_state = server::AppState { db_service };

    // let cf_token = "ad74vvoDGK0M3aE5Lzgzy8aZDZsXvaYvHP5p0Hfn";
    // let zone_name = "duacodie.com";
    // info!("Starting DDNS Client");
    // debug!("Using Cloudflare Token: {}", cf_token);
    // let cf_h = DnsFactory::create(ProviderType::Cloudflare, cf_token);
    // let zones = cf_h.list_zones(Some(zone_name)).await?;
    // if zones.is_empty() {
    //     error!("No zones found");
    //     return Ok(());
    // }
    // let zt: HashMap<String, String> = zones.into_iter().map(|(id, name)| (name, id)).collect();
    // debug!("Zone map created: {:#?}", zt);
    // match zt.get(zone_name) {
    //     Some(zone_id) => {
    //         debug!("Zone ID for '{}': {}", zone_name, zone_id);
    //         let records = cf_h.list_records(zone_id, None).await?;
    //         debug!("DNS Records for '{}': {:#?}", zone_name, records);
    //         //TODO: fn (zone_id)
    //     }
    //     None => error!("Zone '{}' not found.", zone_name),
    // }
    match &cli.command {
        Commands::Config(config_args) => match &config_args.action {
            ConfigSubcommands::Generate { force, format } => {
                println!("正在產生 {} 格式的設定檔 (強制覆蓋: {})", format, force);
            }
            ConfigSubcommands::Check => {
                println!("正在檢查設定檔：{}", cli.config);
            }
        },
        Commands::Start { port } => {
            println!("伺服器正在埠號 {} 啟動...", port);
            info!("Starting DDNS Server");
            server::start_server(app_state).await?;
        }
    }
    Ok(())
}
