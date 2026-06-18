use crate::api::models::{Currencies, Symbol, TickerData};
use crate::api::requests::{
    api_v1_market_all_tickers_get, api_v2_symbols_get, api_v3_currencies_get,
};
use crate::api::tools::get_env;
use dotenvy::dotenv;
use log;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Postgres, QueryBuilder};
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

mod api {
    pub mod models;
    pub mod requests;
    pub mod tools;
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    dotenv().ok();

    let database_url: String = get_env("DATABASE_URL")?;

    let exchange = "kucoin";

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

    let scheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => return Err(e.to_string()),
    };

    match Job::new_async("10 0 * * * *", move |_, _| {
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

            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO ticker 
                                    (exchange, symbol, symbol_name, taker_fee_rate, 
                                    maker_fee_rate, taker_coefficient, maker_coefficient, updated_at)",
                                );

            query_builder.push_values(&tickers.ticker, |mut b, d| {
                b.push_bind(&exchange)
                    .push_bind(&d.symbol)
                    .push_bind(&d.symbol_name)
                    .push_bind(&d.taker_fee_rate)
                    .push_bind(&d.maker_fee_rate)
                    .push_bind(&d.taker_coefficient)
                    .push_bind(&d.maker_coefficient)
                    .push_bind(chrono::Utc::now());
            });

            query_builder.push(
                " ON CONFLICT (exchange, symbol)
                                                DO UPDATE SET
                                                    symbol_name = EXCLUDED.symbol_name,
                                                    taker_fee_rate = EXCLUDED.taker_fee_rate,
                                                    maker_fee_rate = EXCLUDED.maker_fee_rate,
                                                    taker_coefficient = EXCLUDED.taker_coefficient,
                                                    maker_coefficient = EXCLUDED.maker_coefficient,
                                                    updated_at = CURRENT_TIMESTAMP",
            );

            match query_builder.build().execute(&pool).await {
                Ok(_) => log::info!("Success insert {} tickers", tickers.ticker.len()),
                Err(e) => log::error!("Error on bulk insert tickers to db: {}", e),
            }
        })
    }) {
        Ok(job) => match scheduler.add(job).await {
            Ok(_) => log::info!("Добавили задачу api_v1_market_alltickers"),
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => return Err(e.to_string()),
    };

    match Job::new_async("20 0 * * * *", move |_, _| {
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

            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO currency
                                    (exchange, currency, currency_name, full_name, is_margin_enabled, is_debit_enabled, updated_at)",
                                );

            query_builder.push_values(&currencies, |mut b, d| {
                b.push_bind(&exchange)
                    .push_bind(&d.currency)
                    .push_bind(&d.name)
                    .push_bind(&d.full_name)
                    .push_bind(d.is_margin_enabled)
                    .push_bind(d.is_debit_enabled)
                    .push_bind(chrono::Utc::now());
            });

            query_builder.push(
                " ON CONFLICT (exchange, currency)
                                                DO UPDATE SET
                                                    currency_name = EXCLUDED.currency_name,
                                                    full_name = EXCLUDED.full_name,
                                                    is_margin_enabled = EXCLUDED.is_margin_enabled,
                                                    is_debit_enabled = EXCLUDED.is_debit_enabled,
                                                    updated_at = CURRENT_TIMESTAMP",
            );

            match query_builder.build().execute(&pool).await {
                Ok(_) => log::info!("Success insert {} currencies", currencies.len()),
                Err(e) => log::error!("Error on bulk insert currencies to db: {}", e),
            }
        })
    }) {
        Ok(job) => match scheduler.add(job).await {
            Ok(_) => log::info!("Добавили задачу api_v3_currencies"),
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => return Err(e.to_string()),
    }

    match Job::new_async("30 0 * * * *", move |_, _| {
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
            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO symbol
                                    (exchange, symbol, symbol_name, base_currency, quote_currency, fee_currency,
                                    market, base_min_size, quote_min_size, base_max_size, quote_max_size,
                                    base_increment, quote_increment, price_increment, price_limit_rate,
                                    min_funds, is_margin_enabled, enable_trading, fee_category,
                                    maker_fee_coefficient, taker_fee_coefficient, st, updated_at)",
                                );

            query_builder.push_values(&symbols, |mut b, d| {
                b.push_bind(&exchange)
                    .push_bind(&d.symbol)
                    .push_bind(&d.name)
                    .push_bind(&d.base_currency)
                    .push_bind(&d.quote_currency)
                    .push_bind(&d.fee_currency)
                    .push_bind(&d.market)
                    .push_bind(&d.base_min_size)
                    .push_bind(&d.quote_min_size)
                    .push_bind(&d.base_max_size)
                    .push_bind(&d.quote_max_size)
                    .push_bind(&d.base_increment)
                    .push_bind(&d.quote_increment)
                    .push_bind(&d.price_increment)
                    .push_bind(&d.price_limit_rate)
                    .push_bind(&d.min_funds)
                    .push_bind(d.is_margin_enabled)
                    .push_bind(d.enable_trading)
                    .push_bind(d.fee_category)
                    .push_bind(&d.maker_fee_coefficient)
                    .push_bind(&d.taker_fee_coefficient)
                    .push_bind(d.st)
                    .push_bind(chrono::Utc::now());
            });

            query_builder.push(
                                        " ON CONFLICT (exchange, symbol)
                                                DO UPDATE SET
                                                    symbol_name = EXCLUDED.symbol_name,
                                                    base_currency = EXCLUDED.base_currency,
                                                    quote_currency = EXCLUDED.quote_currency,
                                                    fee_currency = EXCLUDED.fee_currency,
                                                    market = EXCLUDED.market,
                                                    base_min_size = EXCLUDED.base_min_size,
                                                    quote_min_size = EXCLUDED.quote_min_size,
                                                    base_max_size = EXCLUDED.base_max_size,
                                                    quote_max_size = EXCLUDED.quote_max_size,
                                                    base_increment = EXCLUDED.base_increment,
                                                    quote_increment = EXCLUDED.quote_increment,
                                                    price_increment = EXCLUDED.price_increment,
                                                    price_limit_rate = EXCLUDED.price_limit_rate,
                                                    min_funds = EXCLUDED.min_funds,
                                                    is_margin_enabled = EXCLUDED.is_margin_enabled,
                                                    enable_trading = EXCLUDED.enable_trading,
                                                    fee_category = EXCLUDED.fee_category,
                                                    maker_fee_coefficient = EXCLUDED.maker_fee_coefficient,
                                                    taker_fee_coefficient = EXCLUDED.taker_fee_coefficient,
                                                    st = EXCLUDED.st,
                                                    updated_at = CURRENT_TIMESTAMP",
                                    );

            match query_builder.build().execute(&pool).await {
                Ok(_) => log::info!("Success insert {} symbols", symbols.len()),
                Err(e) => log::error!("Error on bulk insert symbols to db: {}", e),
            }
        })
    }) {
        Ok(job) => match scheduler.add(job).await {
            Ok(_) => log::info!("Добавили задачу api_v2_symbols"),
            Err(e) => return Err(e.to_string()),
        },
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
