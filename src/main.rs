use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
mod api {
    pub mod common;
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
            match Job::new_async("*/5 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v3_project_list().await {
                            Ok(t) => {
                                for d in t.iter() {
                                    info!("{:?}", d);
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
                    Err(e) => return Err(e.into()),
                },
                Err(e) => return Err(e.into()),
            };

            // match Job::new_async("*/3 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
            //             Ok(client) => match client.api_v1_market_alltickers().await {
            //                 Ok(t) => {
            //                     info!("{:?}", t);
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
            //             info!("Добавили задачу api_v1_market_alltickers")
            //         }
            //         Err(e) => return Err(e.into()),
            //     },
            //     Err(e) => return Err(e.into()),
            // };

            // match Job::new_async("*/7 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
            //             Ok(client) => match client.api_v3_currencies().await {
            //                 Ok(t) => {
            //                     info!("{:?}", t);
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
            //             info!("Добавили задачу api_v3_currencies")
            //         }
            //         Err(e) => return Err(e.into()),
            //     },
            //     Err(e) => return Err(e.into()),
            // }

            // match Job::new_async("*/7 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
            //             Ok(client) => match client.api_v2_symbols().await {
            //                 Ok(t) => {
            //                     info!("{:?}", t);
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
            //             info!("Добавили задачу api_v2_symbols")
            //         }
            //         Err(e) => return Err(e.into()),
            //     },
            //     Err(e) => return Err(e.into()),
            // }
            match Job::new_async("*/7 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::requests::KuCoinClient::new("https://api.kucoin.com".to_string()) {
                        Ok(client) => match client.api_v1_timestamp().await {
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
                    }
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу api_v1_timestamp")
                    }
                    Err(e) => return Err(e.into()),
                },
                Err(e) => return Err(e.into()),
            }

            match s.start().await {
                Ok(_) => {}
                Err(e) => return Err(e.into()),
            }
        }
        Err(e) => return Err(e.into()),
    };

    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
