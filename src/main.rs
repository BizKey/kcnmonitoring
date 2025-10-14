use chrono::{Timelike, Utc};
use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Postgres, QueryBuilder};
use std::env;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::api::models;
mod api {
    pub mod models;
    pub mod requests;
}

fn get_timestamp() -> Result<String, Box<dyn std::error::Error>> {
    let now = Utc::now();

    Ok(now
        .date_naive()
        .and_hms_opt(now.hour(), 0, 0)
        .expect("Invalid time")
        .and_local_timezone(Utc)
        .unwrap()
        .timestamp()
        .to_string())
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
    let pool_borrow = pool.clone();
    let pool_lend = pool.clone();
    let pool_candle = pool.clone();

    match JobScheduler::new().await {
        Ok(s) => {
            match Job::new_async("0 0 * * * *", move |_, _| {
                let pool = pool_candle.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => {
                            match sqlx::query_as::<_, models::SymbolDb>(
                                "SELECT 
                                        exchange, 
                                        symbol
                                    FROM symbol
                                    WHERE 
                                        exchange = $1 
                                        AND enable_trading = true
                                        AND quote_currency = 'USDT';",
                            )
                            .bind(&exchange)
                            .fetch_all(&pool)
                            .await
                            {
                                Ok(symbols) => {
                                    for symbol in symbols.iter() {
                                        match client
                                            .api_v1_market_candles(
                                                symbol.symbol.clone(),
                                                String::from("1hour"),
                                            )
                                            .await
                                        {
                                            Ok(candles) => {
                                                let count_candles = candles.len();
                                                let mut query_builder: QueryBuilder<Postgres> =                                        QueryBuilder::new(
                                            "INSERT INTO candle 
                                            (exchange, symbol, interval, timestamp, open, high, low, close, volume, quote_volume) ",
                                        );
                                                query_builder.push_values(&candles, |mut b, d| {
                                                    b.push_bind(&d.exchange)
                                                        .push_bind(&d.symbol)
                                                        .push_bind(&d.interval)
                                                        .push_bind(&d.timestamp)
                                                        .push_bind(&d.open)
                                                        .push_bind(&d.high)
                                                        .push_bind(&d.low)
                                                        .push_bind(&d.close)
                                                        .push_bind(&d.volume)
                                                        .push_bind(&d.quote_volume);
                                                });

                                                query_builder.push(
                                            " ON CONFLICT (exchange, symbol, interval, timestamp)
                                                DO UPDATE SET
                                                    open = EXCLUDED.open,
                                                    high = EXCLUDED.high,
                                                    low = EXCLUDED.low,
                                                    close = EXCLUDED.close,
                                                    volume = EXCLUDED.volume,
                                                    quote_volume = EXCLUDED.quote_volume",
                                        );

                                                match query_builder.build().execute(&pool).await {
                                                    Ok(_) => {
                                                        info!(
                                                            "Success insert/update {} candles on symbol {}",
                                                            count_candles, &symbol.symbol
                                                        )
                                                    }
                                                    Err(e) => {
                                                        error!(
                                                            "Error on bulk insert/update candles to db: {}:{}",
                                                            e, &symbol.symbol
                                                        )
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("Ошибка при выполнении запроса: {}", e)
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Ошибка при выполнении запроса: {}", e)
                                }
                            };
                        }
                        Err(e) => {
                            error!("Ошибка при выполнении запроса: {}", e)
                        }
                    };
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу api_v1_market_candles")
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            };
            match Job::new_async("0 0 * * * *", move |_, _| {
                let pool = pool_lend.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_project_list().await {
                            Ok(lend) => {
                                let timestamp = get_timestamp().unwrap();
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO lend 
                                    (exchange, timestamp, currency, purchase_enable, redeem_enable, increment, 
                                    min_purchase_size, max_purchase_size, interest_increment, 
                                    min_interest_rate, market_interest_rate, max_interest_rate, 
                                    auto_purchase_enable)",
                                );
                                let count_lend = lend.len();

                                query_builder.push_values(&lend, |mut b, d| {
                                    b.push_bind(&exchange)
                                        .push_bind(&timestamp)
                                        .push_bind(&d.currency)
                                        .push_bind(&d.purchase_enable)
                                        .push_bind(&d.redeem_enable)
                                        .push_bind(&d.increment)
                                        .push_bind(&d.min_purchase_size)
                                        .push_bind(&d.max_purchase_size)
                                        .push_bind(&d.interest_increment)
                                        .push_bind(&d.min_interest_rate)
                                        .push_bind(&d.market_interest_rate)
                                        .push_bind(&d.max_interest_rate)
                                        .push_bind(&d.auto_purchase_enable);
                                });

                                query_builder.push(
                                        " ON CONFLICT (exchange, timestamp, currency)
                                                DO UPDATE SET
                                                    purchase_enable = EXCLUDED.purchase_enable,
                                                    redeem_enable = EXCLUDED.redeem_enable,
                                                    increment = EXCLUDED.increment,
                                                    min_purchase_size = EXCLUDED.min_purchase_size,
                                                    max_purchase_size = EXCLUDED.max_purchase_size,
                                                    interest_increment = EXCLUDED.interest_increment,
                                                    min_interest_rate = EXCLUDED.min_interest_rate,
                                                    market_interest_rate = EXCLUDED.market_interest_rate,
                                                    max_interest_rate = EXCLUDED.max_interest_rate,
                                                    auto_purchase_enable = EXCLUDED.auto_purchase_enable",
                                    );

                                match query_builder.build().execute(&pool).await {
                                    Ok(_) => {
                                        info!("Success insert {} lends", count_lend)
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
                        info!("Добавили задачу api_v3_project_list")
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            };
            match Job::new_async("0 0 * * * *", move |_, _| {
                let pool = pool_borrow.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_margin_borrowrate().await {
                            Ok(borrow) => {
                                let timestamp = get_timestamp().unwrap();
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO borrow 
                                    (exchange, timestamp, currency, hourly_borrow_rate, annualized_borrow_rate)",
                                );
                                let count_borrow = borrow.items.len();

                                query_builder.push_values(&borrow.items, |mut b, d| {
                                    b.push_bind(&exchange)
                                        .push_bind(&timestamp)
                                        .push_bind(&d.currency)
                                        .push_bind(&d.hourly_borrow_rate)
                                        .push_bind(&d.annualized_borrow_rate);
                                });

                                query_builder.push(
                                        " ON CONFLICT (exchange, timestamp, currency)
                                                DO UPDATE SET
                                                    hourly_borrow_rate = EXCLUDED.hourly_borrow_rate,
                                                    annualized_borrow_rate = EXCLUDED.annualized_borrow_rate",
                                    );

                                match query_builder.build().execute(&pool).await {
                                    Ok(_) => {
                                        info!("Success insert {} borrow", count_borrow)
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
                        info!("Добавили задачу api_v3_margin_borrowrate")
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            };

            match Job::new_async("0 0 * * * *", move |_, _| {
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

            match Job::new_async("0 0 * * * *", move |_, _| {
                let pool: sqlx::Pool<Postgres> = pool_currency.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_currencies().await {
                            Ok(currencies) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO currency 
                                    (exchange, currency, name, full_name, precision, confirms, 
                                    contract_address, is_margin_enabled, is_debit_enabled)",
                                );

                                query_builder.push_values(&currencies, |mut b, d| {
                                    b.push_bind(&exchange)
                                        .push_bind(&d.currency)
                                        .push_bind(&d.name)
                                        .push_bind(&d.full_name)
                                        .push_bind(&d.precision)
                                        .push_bind(&d.confirms)
                                        .push_bind(&d.contract_address)
                                        .push_bind(&d.is_margin_enabled)
                                        .push_bind(&d.is_debit_enabled);
                                });

                                query_builder.push(
                                    " ON CONFLICT (exchange, currency)
                                                DO UPDATE SET
                                                    name = EXCLUDED.name,
                                                    full_name = EXCLUDED.full_name,
                                                    precision = EXCLUDED.precision,
                                                    confirms = EXCLUDED.confirms,
                                                    contract_address = EXCLUDED.contract_address,
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

            match Job::new_async("0 0 * * * *", move |_, _| {
                let pool = pool_symbols.clone();
                let exchange = String::from("kucoin");
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v2_symbols().await {
                            Ok(symbols) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO symbol 
                                    (exchange, symbol, name, base_currency, quote_currency, fee_currency, 
                                    market, base_min_size, quote_min_size, base_max_size, quote_max_size, 
                                    base_increment, quote_increment, price_increment, price_limit_rate, 
                                    min_funds, is_margin_enabled, enable_trading, fee_category, 
                                    maker_fee_coefficient, taker_fee_coefficient, st, callauction_is_enabled, 
                                    callauction_price_floor, callauction_price_ceiling, 
                                    callauction_first_stage_start_time, callauction_second_stage_start_time, 
                                    callauction_third_stage_start_time, trading_start_time)",
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
                                        .push_bind(&d.is_margin_enabled)
                                        .push_bind(&d.enable_trading)
                                        .push_bind(&d.fee_category)
                                        .push_bind(&d.maker_fee_coefficient)
                                        .push_bind(&d.taker_fee_coefficient)
                                        .push_bind(&d.st)
                                        .push_bind(&d.callauction_is_enabled)
                                        .push_bind(&d.callauction_price_floor)
                                        .push_bind(&d.callauction_price_ceiling)
                                        .push_bind(&d.callauction_first_stage_start_time)
                                        .push_bind(&d.callauction_second_stage_start_time)
                                        .push_bind(&d.callauction_third_stage_start_time)
                                        .push_bind(&d.trading_start_time);
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
                                                    st = EXCLUDED.st,
                                                    callauction_is_enabled = EXCLUDED.callauction_is_enabled,
                                                    callauction_price_floor = EXCLUDED.callauction_price_floor,
                                                    callauction_price_ceiling = EXCLUDED.callauction_price_ceiling,
                                                    callauction_first_stage_start_time = EXCLUDED.callauction_first_stage_start_time,
                                                    callauction_second_stage_start_time = EXCLUDED.callauction_second_stage_start_time,
                                                    callauction_third_stage_start_time = EXCLUDED.callauction_third_stage_start_time,
                                                    trading_start_time = EXCLUDED.trading_start_time",
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
