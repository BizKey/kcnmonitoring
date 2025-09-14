use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, QueryBuilder};
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
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let pool_tickers = pool.clone();
    let pool_currency = pool.clone();
    let pool_symbols = pool.clone();

    match JobScheduler::new().await {
        Ok(s) => {
            // match Job::new_async("59 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
            //             Ok(client) => match client.api_v3_project_list().await {
            //                 Ok(t) => {
            //                     for d in t.iter() {
            //                         info!(
            //                             "currency:{:10}market_interest_rate:{} purchase_enable:{} redeem_enable:{} increment:{} min_purchase_size:{} max_purchase_size:{} interest_increment:{} min_interest_rate:{} max_interest_rate:{} auto_purchase_enable:{}",
            //                             d.currency,
            //                             d.market_interest_rate,
            //                             d.purchase_enable,
            //                             d.redeem_enable,
            //                             d.increment,
            //                             d.min_purchase_size,
            //                             d.max_purchase_size,
            //                             d.interest_increment,
            //                             d.min_interest_rate,
            //                             d.max_interest_rate,
            //                             d.auto_purchase_enable,
            //                         );
            //                     }
            //                 }
            //                 Err(e) => {
            //                     error!("Ошибка при выполнении запроса: {}", e)
            //                 }
            //             },
            //             Err(e) => {
            //                 error!("Ошибка при выполнении запроса: {}", e)
            //             }
            //         };
            //     })
            // }) {
            //     Ok(job) => match s.add(job).await {
            //         Ok(_) => {
            //             info!("Добавили задачу api_v3_project_list")
            //         }
            //         Err(e) => return Err(e),
            //     },
            //     Err(e) => return Err(e),
            // };
            // match Job::new_async("59 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
            //             Ok(client) => match client.api_v3_margin_borrowrate().await {
            //                 Ok(t) => {
            //                     for d in t.items.iter() {
            //                         info!(
            //                             "VIP level:{} currency:{:10}hourly_borrow_rate:{:12}annualized_borrow_rate:{}",
            //                             t.vip_level,
            //                             d.currency,
            //                             d.hourly_borrow_rate,
            //                             d.annualized_borrow_rate
            //                         );
            //                     }
            //                 }
            //                 Err(e) => {
            //                     error!("Ошибка при выполнении запроса: {}", e)
            //                 }
            //             },
            //             Err(e) => {
            //                 error!("Ошибка при выполнении запроса: {}", e)
            //             }
            //         };
            //     })
            // }) {
            //     Ok(job) => match s.add(job).await {
            //         Ok(_) => {
            //             info!("Добавили задачу api_v3_margin_borrowrate")
            //         }
            //         Err(e) => return Err(e),
            //     },
            //     Err(e) => return Err(e),
            // };

            match Job::new_async("0 0 * * * *", move |_, _| {
                let pool = pool_tickers.clone();
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v1_market_alltickers().await {
                            Ok(tickers) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO Ticker (
                                        symbol, symbol_name, buy, best_bid_size, sell, best_ask_size,
                                        change_rate, change_price, high, low, vol, vol_value, last,
                                        average_price, taker_fee_rate, maker_fee_rate, taker_coefficient,
                                        maker_coefficient
                                    )",
                                );

                                query_builder.push_values(&tickers.ticker, |mut b, d| {
                                    b.push_bind(&d.symbol)
                                        .push_bind(&d.symbol_name)
                                        .push_bind(&d.buy)
                                        .push_bind(&d.best_bid_size)
                                        .push_bind(&d.sell)
                                        .push_bind(&d.best_ask_size)
                                        .push_bind(&d.change_rate)
                                        .push_bind(&d.change_price)
                                        .push_bind(&d.high)
                                        .push_bind(&d.low)
                                        .push_bind(&d.vol)
                                        .push_bind(&d.vol_value)
                                        .push_bind(&d.last)
                                        .push_bind(&d.average_price)
                                        .push_bind(&d.taker_fee_rate)
                                        .push_bind(&d.maker_fee_rate)
                                        .push_bind(&d.taker_coefficient)
                                        .push_bind(&d.maker_coefficient);
                                });

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
                let pool = pool_currency.clone();
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_currencies().await {
                            Ok(currencies) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO Currency (
                                        currency, name, full_name, precision, confirms, 
                                        contract_address, is_margin_enabled, is_debit_enabled
                                    )",
                                );

                                query_builder.push_values(&currencies, |mut b, d| {
                                    b.push_bind(&d.currency)
                                        .push_bind(&d.name)
                                        .push_bind(&d.full_name)
                                        .push_bind(&d.precision)
                                        .push_bind(&d.confirms)
                                        .push_bind(&d.contract_address)
                                        .push_bind(&d.is_margin_enabled)
                                        .push_bind(&d.is_debit_enabled);
                                });

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
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v2_symbols().await {
                            Ok(symbols) => {
                                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                                    "INSERT INTO Symbol (
                                        symbol, name, base_currency, quote_currency, fee_currency, market, 
                                        base_min_size, quote_min_size, base_max_size, quote_max_size, base_increment,
                                        quote_increment, price_increment, price_limit_rate, min_funds, is_margin_enabled,
                                        enable_trading, fee_category, maker_fee_coefficient, taker_fee_coefficient, st, callauction_is_enabled,
                                        callauction_price_floor, callauction_price_ceiling, callauction_first_stage_start_time,
                                        callauction_second_stage_start_time, callauction_third_stage_start_time, trading_start_time
                                    )",
                                );

                                query_builder.push_values(&symbols, |mut b, d| {
                                    b.push_bind(&d.symbol)
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
            // match Job::new_async("* * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
            //             Ok(client) => match client.api_v1_timestamp().await {
            //                 Ok(timestamp) => {
            //                     info!("Server timestamp:{}", timestamp);
            //                 }
            //                 Err(e) => {
            //                     error!("Ошибка при выполнении запроса: {}", e)
            //                 }
            //             },
            //             Err(e) => {
            //                 error!("Ошибка при выполнении запроса: {}", e)
            //             }
            //         }
            //     })
            // }) {
            //     Ok(job) => match s.add(job).await {
            //         Ok(_) => {
            //             info!("Добавили задачу api_v1_timestamp")
            //         }
            //         Err(e) => return Err(e),
            //     },
            //     Err(e) => return Err(e),
            // }

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
