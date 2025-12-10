use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Postgres, QueryBuilder};
use std::env;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

mod api {
    pub mod models;
    pub mod requests;
}

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    env_logger::init();
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let pool_tickers = pool.clone();
    let pool_currency = pool.clone();
    let pool_symbols = pool.clone();

    match JobScheduler::new().await {
        Ok(s) => {
            match Job::new_async("0 0 0 * * *", move |_, _| {
                let pool = pool_tickers.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v1_market_alltickers().await {
                            Ok(tickers) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO ticker 
                                    (exchange, symbol, symbol_name, taker_fee_rate, 
                                    maker_fee_rate, taker_coefficient, maker_coefficient)",
                                );

                                query_builder.push_values(&tickers.ticker, |mut b, d| {
                                    b.push_bind(&exchange)
                                        .push_bind(&d.symbol)
                                        .push_bind(&d.symbol_name)
                                        .push_bind(&d.taker_fee_rate)
                                        .push_bind(&d.maker_fee_rate)
                                        .push_bind(&d.taker_coefficient)
                                        .push_bind(&d.maker_coefficient);
                                });

                                query_builder.push(
                                    " ON CONFLICT (exchange, symbol)
                                                DO UPDATE SET
                                                    symbol_name = EXCLUDED.symbol_name,
                                                    taker_fee_rate = EXCLUDED.taker_fee_rate,
                                                    maker_fee_rate = EXCLUDED.maker_fee_rate,
                                                    taker_coefficient = EXCLUDED.taker_coefficient,
                                                    maker_coefficient = EXCLUDED.maker_coefficient",
                                );

                                match query_builder.build().execute(&pool).await {
                                    Ok(_) => {
                                        info!("Success insert {} tickers", tickers.ticker.len())
                                    }
                                    Err(e) => error!("Error on bulk insert tickers to db: {}", e),
                                }
                            }
                            Err(e) => {
                                error!("Ошибка при выполнении запроса: {}", e)
                            }
                        },
                        Err(e) => {
                            error!("Ошибка при выполнении запроса: {}", e)
                        }
                    };
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу api_v1_market_alltickers")
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            };

            match Job::new_async("0 0 0 * * *", move |_, _| {
                let pool: sqlx::Pool<Postgres> = pool_currency.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_currencies().await {
                            Ok(currencies) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO currency 
                                    (exchange, currency, currency_name, full_name, is_margin_enabled, is_debit_enabled)",
                                );

                                query_builder.push_values(&currencies, |mut b, d| {
                                    b.push_bind(&exchange)
                                        .push_bind(&d.currency)
                                        .push_bind(&d.name)
                                        .push_bind(&d.full_name)
                                        .push_bind(d.is_margin_enabled)
                                        .push_bind(d.is_debit_enabled);
                                });

                                query_builder.push(
                                    " ON CONFLICT (exchange, currency)
                                                DO UPDATE SET
                                                    name = EXCLUDED.name,
                                                    full_name = EXCLUDED.full_name,
                                                    is_margin_enabled = EXCLUDED.is_margin_enabled,
                                                    is_debit_enabled = EXCLUDED.is_debit_enabled",
                                );

                                match query_builder.build().execute(&pool).await {
                                    Ok(_) => {
                                        info!("Success insert {} currencies", currencies.len())
                                    }
                                    Err(e) => {
                                        error!("Error on bulk insert currencies to db: {}", e)
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Ошибка при выполнении запроса: {}", e)
                            }
                        },
                        Err(e) => {
                            error!("Ошибка при выполнении запроса: {}", e)
                        }
                    };
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу api_v3_currencies")
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            }

            match Job::new_async("0 0 0 * * *", move |_, _| {
                let pool = pool_symbols.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v2_symbols().await {
                            Ok(symbols) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO symbol 
                                    (exchange, symbol, symbol_name, base_currency, quote_currency, fee_currency, 
                                    market, base_min_size, quote_min_size, base_max_size, quote_max_size, 
                                    base_increment, quote_increment, price_increment, price_limit_rate, 
                                    min_funds, is_margin_enabled, enable_trading, fee_category, 
                                    maker_fee_coefficient, taker_fee_coefficient, st)",
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
                                        .push_bind(d.st);
                                });

                                query_builder.push(
                                        " ON CONFLICT (exchange, symbol)
                                                DO UPDATE SET
                                                    name = EXCLUDED.name,
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
                                                    st = EXCLUDED.st",
                                    );

                                match query_builder.build().execute(&pool).await {
                                    Ok(_) => {
                                        info!("Success insert {} symbols", symbols.len())
                                    }
                                    Err(e) => {
                                        error!("Error on bulk insert currencies to db: {}", e)
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Ошибка при выполнении запроса: {}", e)
                            }
                        },
                        Err(e) => {
                            error!("Ошибка при выполнении запроса: {}", e)
                        }
                    };
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу api_v2_symbols")
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            }

            match s.start().await {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        Err(e) => return Err(e),
    };

    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
