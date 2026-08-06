mod api;
mod db;
mod infra;
mod tools;
use anyhow::Result;
use api::requests::{api_v1_market_all_tickers_get, api_v2_symbols_get, api_v3_currencies_get};

use db::command::{insert_currencies_to_db, insert_symbols_to_db, insert_tickers_to_db};
use db::models::SymbolDb;
use db::postgres::create_db_pool;
use dotenvy::dotenv;
use infra::logging::init_tracing;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::db::models::{CurrenciesDb, TickerDb};

const EXCHANGE: &str = "kucoin";
const CRON_EVERY_5_MIN: &str = "0 */5 * * * *";

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
    let tickers = tickers.ticker.into_iter().map(TickerDb::from).collect();

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
    let currencies = currencies.into_iter().map(CurrenciesDb::from).collect();

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

    let symbols = symbols.into_iter().map(SymbolDb::from).collect();

    if let Err(e) = insert_symbols_to_db(&pool, EXCHANGE, symbols).await {
        error!("Failed to insert symbols to DB: {}", e);
    } else {
        info!("Successfully inserted symbols to DB");
    }
}

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
