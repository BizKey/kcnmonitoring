use crate::api::db::{insert_currencies_to_db, insert_symbols_to_db, insert_tickers_to_db};
use crate::api::models::{Currencies, Symbol, TickerData};
use crate::api::requests::{
    api_v1_market_all_tickers_get, api_v2_symbols_get, api_v3_currencies_get,
};
use crate::api::tools::get_env;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};

mod api {
    pub mod db;
    pub mod models;
    pub mod requests;
    pub mod tools;
}
use sqlx::PgPool;
use tracing::{error, info};
const EXCHANGE: &str = "kucoin";

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    dotenv().ok();

    let database_url: String = get_env("DATABASE_URL")?;

    let pool: PgPool = match PgPoolOptions::new()
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
            error!("{}", msg);
            return Err(msg);
        }
    };

    let pool_tickers: PgPool = pool.clone();
    let pool_currency: PgPool = pool.clone();
    let pool_symbols: PgPool = pool.clone();

    let scheduler: JobScheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => {
            let msg: String = format!("Failed init scheduler:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    };

    let job_tickers: Job = match Job::new_async("0 */5 * * * *", move |_, _| {
        let pool: PgPool = pool_tickers.clone();
        Box::pin(async move {
            let tickers_option: Option<TickerData> = match api_v1_market_all_tickers_get().await {
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    error!("{}", msg);
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
                Ok(_) => info!("Success tickers send to db"),
                _ => {}
            }
        })
    }) {
        Ok(job_tickers) => job_tickers,
        Err(e) => {
            let msg: String = format!("Failed init scheduler ticker:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    };

    match scheduler.add(job_tickers).await {
        Ok(_) => info!("Добавили задачу api_v1_market_alltickers"),
        Err(e) => {
            let msg: String = format!("Failed add scheduler ticker:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    }

    let job_currencies: Job = match Job::new_async("0 */5 * * * *", move |_, _| {
        let pool: PgPool = pool_currency.clone();
        Box::pin(async move {
            let currencies_option: Option<Vec<Currencies>> = match api_v3_currencies_get().await {
                Ok(currencies_option) => currencies_option,
                Err(e) => {
                    let msg = format!("Ошибка при выполнении запроса: {}", e);
                    error!("{}", msg);
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
                Ok(_) => info!("Success currency send to db"),
                _ => {}
            }
        })
    }) {
        Ok(job_currencies) => job_currencies,
        Err(e) => {
            let msg: String = format!("Failed init scheduler currency:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    };

    match scheduler.add(job_currencies).await {
        Ok(_) => info!("Добавили задачу api_v3_currencies"),
        Err(e) => {
            let msg: String = format!("Failed add scheduler currency:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    }

    let job_symbols: Job = match Job::new_async("0 */5 * * * *", move |_, _| {
        let pool: PgPool = pool_symbols.clone();
        Box::pin(async move {
            let symbols_option: Option<Vec<Symbol>> = match api_v2_symbols_get().await {
                Ok(symbols_option) => symbols_option,
                Err(e) => {
                    let msg: String = format!("Ошибка при выполнении запроса: {}", e);
                    error!("{}", msg);
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
                Ok(_) => info!("Success symbol send to db"),
                _ => {}
            }
        })
    }) {
        Ok(job_symbols) => job_symbols,
        Err(e) => {
            let msg: String = format!("Failed init scheduler symbols:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    };

    match scheduler.add(job_symbols).await {
        Ok(_) => info!("Добавили задачу api_v2_symbols"),
        Err(e) => {
            let msg: String = format!("Failed add scheduler symbols:{}", e);
            error!("{}", msg);
            return Err(msg);
        }
    }

    scheduler.start().await.map_err(|e| {
        error!("Failed start scheduler:{}", e);
        format!("Failed start scheduler:{}", e)
    })?;

    loop {
        sleep(Duration::from_secs(100)).await;
    }
}
