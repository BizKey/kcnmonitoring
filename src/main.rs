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
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
const EXCHANGE: &str = "kucoin";

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&get_env("DATABASE_URL")?)
        .await?;

    let pool_tickers = pool.clone();
    let pool_currency = pool.clone();
    let pool_symbols = pool.clone();

    let scheduler = JobScheduler::new().await?;

    let job_tickers = Job::new_async("0 */5 * * * *", move |_, _| {
        let pool = pool_tickers.clone();
        Box::pin(async move {
            let tickers = match api_v1_market_all_tickers_get().await {
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    error!("{}", msg);
                    return;
                }
                Ok(tickers) => tickers,
            };

            let Some(tickers) = tickers else {
                return;
            };

            match insert_tickers_to_db(pool, EXCHANGE, tickers).await {
                Ok(_) => info!("Success tickers send to db"),
                _ => {}
            }
        })
    })?;

    scheduler.add(job_tickers).await?;

    info!("Добавили задачу api_v1_market_alltickers");

    let job_currencies = Job::new_async("0 */5 * * * *", move |_, _| {
        let pool = pool_currency.clone();
        Box::pin(async move {
            let currencies = match api_v3_currencies_get().await {
                Ok(currencies) => currencies,
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    error!("{}", msg);
                    return;
                }
            };

            let Some(currencies) = currencies else {
                return;
            };

            match insert_currencies_to_db(pool, EXCHANGE, currencies).await {
                Ok(_) => info!("Success currency send to db"),
                _ => {}
            }
        })
    })?;

    scheduler.add(job_currencies).await?;

    info!("Добавили задачу api_v3_currencies");

    let job_symbols = Job::new_async("0 */5 * * * *", move |_, _| {
        let pool = pool_symbols.clone();
        Box::pin(async move {
            let symbols = match api_v2_symbols_get().await {
                Ok(symbols) => symbols,
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    error!("{}", msg);
                    return;
                }
            };

            let Some(symbols) = symbols else {
                return;
            };

            match insert_symbols_to_db(pool, EXCHANGE, symbols).await {
                Ok(_) => info!("Success symbol send to db"),
                _ => {}
            }
        })
    })?;

    scheduler.add(job_symbols).await?;
    info!("Добавили задачу api_v2_symbols");

    scheduler.start().await?;

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
