use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

const DB_MAX_CONNECTIONS: u32 = 10;
const DB_MIN_CONNECTIONS: u32 = 1;
const DB_ACQUIRE_TIMEOUT: u64 = 10;
const DB_IDLE_TIMEOUT: u64 = 600;
const DB_MAX_LIFETIME: u64 = 1800;

pub async fn create_db_pool(database_url: &str) -> Result<sqlx::PgPool> {
    PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .min_connections(DB_MIN_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(DB_ACQUIRE_TIMEOUT))
        .idle_timeout(Duration::from_secs(DB_IDLE_TIMEOUT))
        .max_lifetime(Duration::from_secs(DB_MAX_LIFETIME))
        .connect(database_url)
        .await
        .map_err(Into::into)
}
