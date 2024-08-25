use std::time::Duration;

use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    let mut opt = ConnectOptions::new("sqlite://simple_bulletin.db?mode=rwc");
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("schema"); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await.expect("Failed to connect");

    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to migrate");
}
