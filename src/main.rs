use crate::api::db::{insert_currencies_to_db, insert_symbols_to_db, insert_tickers_to_db};
use crate::api::models::{Currencies, Symbol, TickerData};
use crate::api::requests::{
    api_v1_market_all_tickers_get, api_v2_symbols_get, api_v3_currencies_get,
};
use crate::api::tools::get_env;
use dotenvy::dotenv;

use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

mod api {
    pub mod db;
    pub mod models;
    pub mod requests;
    pub mod tools;
}

const EXCHANGE: &str = "kucoin";

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    dotenv().ok();

    let database_url: String = get_env("DATABASE_URL")?;

    let pool: sqlx::Pool<Postgres> = match PgPoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            let msg: String = format!("Failed to create pg pool:{}", e);
            log::error!("{}", msg);
            return Err(msg);
        }
    };

    let pool_tickers: sqlx::Pool<Postgres> = pool.clone();
    let pool_currency: sqlx::Pool<Postgres> = pool.clone();
    let pool_symbols: sqlx::Pool<Postgres> = pool.clone();

    let scheduler: JobScheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => {
            let msg: String = format!("Failed init scheduler:{}", e);
            log::error!("{}", msg);
            return Err(msg);
        }
    };

    let job_tickers: Job = match Job::new_async("10 0 * * * *", move |_, _| {
        let pool: sqlx::Pool<Postgres> = pool_tickers.clone();
        Box::pin(async move {
            let tickers_option: Option<TickerData> = match api_v1_market_all_tickers_get().await {
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    log::error!("{}", msg);
                    return;
                }
                Ok(tickers_option) => tickers_option,
            };

            let tickers: TickerData = match tickers_option {
                Some(tickers) => tickers,
                None => {
                    return;
                }
            };

            match insert_tickers_to_db(pool, EXCHANGE, tickers).await {
                Err(_) => {}
                Ok(_) => {}
            }
        })
    }) {
        Ok(job_tickers) => job_tickers,
        Err(e) => {
            let msg: String = format!("Failed init scheduler ticker:{}", e);
            log::error!("{}", msg);
            return Err(msg);
        }
    };

    match scheduler.add(job_tickers).await {
        Ok(_) => log::info!("Добавили задачу api_v1_market_alltickers"),
        Err(e) => return Err(e.to_string()),
    }

    let job_currencies: Job = match Job::new_async("20 0 * * * *", move |_, _| {
        let pool: sqlx::Pool<Postgres> = pool_currency.clone();
        Box::pin(async move {
            let currencies_option: Option<Vec<Currencies>> = match api_v3_currencies_get().await {
                Ok(currencies_option) => currencies_option,
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    log::error!("{}", msg);
                    return;
                }
            };

            let currencies: Vec<Currencies> = match currencies_option {
                Some(currencies_option) => currencies_option,
                None => {
                    return;
                }
            };

            match insert_currencies_to_db(pool, EXCHANGE, currencies).await {
                Err(_) => {}
                Ok(_) => {}
            }
        })
    }) {
        Ok(job_currencies) => job_currencies,
        Err(e) => {
            let msg: String = format!("Failed init scheduler currency:{}", e);
            log::error!("{}", msg);
            return Err(msg);
        }
    };

    match scheduler.add(job_currencies).await {
        Ok(_) => log::info!("Добавили задачу api_v3_currencies"),
        Err(e) => return Err(e.to_string()),
    }

    let job_symbols: Job = match Job::new_async("30 0 * * * *", move |_, _| {
        let pool: sqlx::Pool<Postgres> = pool_symbols.clone();
        Box::pin(async move {
            let symbols_option: Option<Vec<Symbol>> = match api_v2_symbols_get().await {
                Ok(symbols_option) => symbols_option,
                Err(e) => {
                    let msg: String = format!("Ошибка при выполнении запроса: {}", e);
                    log::error!("{}", msg);
                    return;
                }
            };

            let symbols: Vec<Symbol> = match symbols_option {
                Some(symbols_option) => symbols_option,
                None => {
                    return;
                }
            };

            match insert_symbols_to_db(pool, EXCHANGE, symbols).await {
                Err(_) => {}
                Ok(_) => {}
            }
        })
    }) {
        Ok(job_symbols) => job_symbols,
        Err(e) => {
            let msg: String = format!("Failed init scheduler symbols:{}", e);
            log::error!("{}", msg);
            return Err(msg);
        }
    };

    match scheduler.add(job_symbols).await {
        Ok(_) => log::info!("Добавили задачу api_v2_symbols"),
        Err(e) => return Err(e.to_string()),
    }

    match scheduler.start().await {
        Ok(_) => {}
        Err(e) => return Err(e.to_string()),
    };

    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
