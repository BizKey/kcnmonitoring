use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;

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

    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&database_url)
    //     .await
    //     .expect("Failed to create pool");

    match JobScheduler::new().await {
        Ok(s) => {
            match Job::new_async("59 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_project_list().await {
                            Ok(t) => {
                                for d in t.iter() {
                                    info!(
                                        "currency:{:10}market_interest_rate:{} purchase_enable:{} redeem_enable:{} increment:{} min_purchase_size:{} max_purchase_size:{} interest_increment:{} min_interest_rate:{} max_interest_rate:{} auto_purchase_enable:{}",
                                        d.currency,
                                        d.market_interest_rate,
                                        d.purchase_enable,
                                        d.redeem_enable,
                                        d.increment,
                                        d.min_purchase_size,
                                        d.max_purchase_size,
                                        d.interest_increment,
                                        d.min_interest_rate,
                                        d.max_interest_rate,
                                        d.auto_purchase_enable,
                                    );
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
            match Job::new_async("59 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_margin_borrowrate().await {
                            Ok(t) => {
                                for d in t.items.iter() {
                                    info!(
                                        "VIP level:{} currency:{:10}hourly_borrow_rate:{:12}annualized_borrow_rate:{}",
                                        t.vip_level,
                                        d.currency,
                                        d.hourly_borrow_rate,
                                        d.annualized_borrow_rate
                                    );
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

            match Job::new_async("59 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v1_market_alltickers().await {
                            Ok(t) => {
                                info!("{:?}", t);
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

            match Job::new_async("59 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_currencies().await {
                            Ok(t) => {
                                info!("{:?}", t);
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

            match Job::new_async("59 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v2_symbols().await {
                            Ok(t) => {
                                info!("{:?}", t);
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
            match Job::new_async("59 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v1_timestamp().await {
                            Ok(timestamp) => {
                                info!("Server timestamp:{}", timestamp);
                            }
                            Err(e) => {
                                error!("Ошибка при выполнении запроса: {}", e)
                            }
                        },
                        Err(e) => {
                            error!("Ошибка при выполнении запроса: {}", e)
                        }
                    }
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу api_v1_timestamp")
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
