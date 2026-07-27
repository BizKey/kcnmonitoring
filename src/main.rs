mod api {
    pub mod db;
    pub mod models;
    pub mod requests;
    pub mod tools;
}

use crate::api::db::{insert_currencies_to_db, insert_symbols_to_db, insert_tickers_to_db};
use crate::api::requests::{
    api_v1_market_all_tickers_get, api_v2_symbols_get, api_v3_currencies_get,
};
use crate::api::tools::get_env;
use anyhow::{Context, Result};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

const EXCHANGE: &str = "kucoin";
const CRON_EVERY_5_MIN: &str = "0 */5 * * * *";
const DB_MAX_CONNECTIONS: u32 = 10;
const DB_MIN_CONNECTIONS: u32 = 1;
const DB_ACQUIRE_TIMEOUT: u64 = 10;
const DB_IDLE_TIMEOUT: u64 = 600;
const DB_MAX_LIFETIME: u64 = 1800;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();
}

async fn create_db_pool() -> Result<sqlx::PgPool> {
    let database_url = get_env("DATABASE_URL").context("DATABASE_URL not set")?;

    PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .min_connections(DB_MIN_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(DB_ACQUIRE_TIMEOUT))
        .idle_timeout(Duration::from_secs(DB_IDLE_TIMEOUT))
        .max_lifetime(Duration::from_secs(DB_MAX_LIFETIME))
        .connect(&database_url)
        .await
        .context("Failed to connect to database")
}

async fn fetch_and_store_tickers(pool: sqlx::PgPool) {
    let tickers = match api_v1_market_all_tickers_get().await {
        Ok(Some(tickers)) => tickers,
        Ok(None) => {
            warn!("No tickers data received from API");
            return;
        }
        Err(e) => {
            error!("Failed to fetch tickers: {}", e);
            return;
        }
    };

    if let Err(e) = insert_tickers_to_db(&pool, EXCHANGE, tickers).await {
        error!("Failed to insert tickers to DB: {}", e);
    } else {
        info!("Successfully inserted tickers to DB");
    }
}

async fn fetch_and_store_currencies(pool: sqlx::PgPool) {
    let currencies = match api_v3_currencies_get().await {
        Ok(Some(currencies)) => currencies,
        Ok(None) => {
            warn!("No currencies data received from API");
            return;
        }
        Err(e) => {
            error!("Failed to fetch currencies: {}", e);
            return;
        }
    };

    if let Err(e) = insert_currencies_to_db(&pool, EXCHANGE, currencies).await {
        error!("Failed to insert currencies to DB: {}", e);
    } else {
        info!("Successfully inserted currencies to DB");
    }
}

async fn fetch_and_store_symbols(pool: sqlx::PgPool) {
    let symbols = match api_v2_symbols_get().await {
        Ok(Some(symbols)) => symbols,
        Ok(None) => {
            warn!("No symbols data received from API");
            return;
        }
        Err(e) => {
            error!("Failed to fetch symbols: {}", e);
            return;
        }
    };

    if let Err(e) = insert_symbols_to_db(&pool, EXCHANGE, symbols).await {
        error!("Failed to insert symbols to DB: {}", e);
    } else {
        info!("Successfully inserted symbols to DB");
    }
}

// ✅ Правильная реализация - замыкание захватывает все нужные данные
async fn create_ticker_job(scheduler: &JobScheduler, pool: sqlx::PgPool) -> Result<()> {
    let job = Job::new_async(CRON_EVERY_5_MIN, move |_, _| {
        let pool = pool.clone();
        Box::pin(async move {
            fetch_and_store_tickers(pool).await;
        })
    })?;

    scheduler.add(job).await?;
    info!("Added job: Tickers fetcher");
    Ok(())
}

async fn create_currency_job(scheduler: &JobScheduler, pool: sqlx::PgPool) -> Result<()> {
    let job = Job::new_async(CRON_EVERY_5_MIN, move |_, _| {
        let pool = pool.clone();
        Box::pin(async move {
            fetch_and_store_currencies(pool).await;
        })
    })?;

    scheduler.add(job).await?;
    info!("Added job: Currencies fetcher");
    Ok(())
}

async fn create_symbol_job(scheduler: &JobScheduler, pool: sqlx::PgPool) -> Result<()> {
    let job = Job::new_async(CRON_EVERY_5_MIN, move |_, _| {
        let pool = pool.clone();
        Box::pin(async move {
            fetch_and_store_symbols(pool).await;
        })
    })?;

    scheduler.add(job).await?;
    info!("Added job: Symbols fetcher");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    dotenv().ok();

    info!("Starting KuCoin data fetcher");
    info!("Exchange: {}", EXCHANGE);

    let pool = create_db_pool().await?;
    info!("Database connection pool created successfully");

    let mut scheduler = JobScheduler::new().await?;

    create_ticker_job(&scheduler, pool.clone()).await?;
    create_currency_job(&scheduler, pool.clone()).await?;
    create_symbol_job(&scheduler, pool).await?;

    scheduler.start().await?;
    info!("Scheduler started. All jobs will run every 5 minutes");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    info!("Shutting down gracefully...");
    scheduler.shutdown().await?;

    Ok(())
}
