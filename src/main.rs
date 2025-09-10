use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
mod api {
    pub mod common;
    pub mod currencies;
    pub mod loan;
    pub mod requests;
    pub mod symbols;
    pub mod tickers;
}

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    env_logger::init();
    dotenv().ok();

    match env::var("KUCOIN_PASS") {
        Ok(val) => println!("KUCOIN_PASS: {}", val),
        Err(e) => error!("Не удалось получить переменную:'KUCOIN_PASS' {}", e),
    }

    match env::var("KUCOIN_KEY") {
        Ok(val) => println!("KUCOIN_KEY: {}", val),
        Err(e) => error!("Не удалось получить переменную:'KUCOIN_KEY' {}", e),
    }

    match env::var("KUCOIN_SECRET") {
        Ok(val) => println!("KUCOIN_SECRET: {}", val),
        Err(e) => error!("Не удалось получить переменную:'KUCOIN_SECRET' {}", e),
    }

    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&database_url)
    //     .await
    //     .expect("Failed to create pool");

    match JobScheduler::new().await {
        Ok(s) => {
            // match Job::new_async("*/5 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::symbols::get_symbols().await {
            //             Ok(symbols) => {
            //                 for symbol in symbols.iter() {
            //                     info!("Символ: {:?}", symbol.symbol);
            //                 }
            //             }
            //             Err(e) => {
            //                 error!("Ошибка при выполнении запроса: {}", e)
            //             }
            //         }
            //     })
            // }) {
            //     Ok(job) => match s.add(job).await {
            //         Ok(_) => {
            //             info!("Добавили задачу get_symbols")
            //         }
            //         Err(e) => return Err(e.into()),
            //     },
            //     Err(e) => return Err(e.into()),
            // };

            // match Job::new_async("*/3 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::currencies::get_currencies().await {
            //             Ok(symbols) => {
            //                 for symbol in symbols.iter() {
            //                     info!("Символ: {:?}", symbol.currency);
            //                 }
            //             }
            //             Err(e) => {
            //                 error!("Ошибка при выполнении запроса: {}", e)
            //             }
            //         }
            //     })
            // }) {
            //     Ok(job) => match s.add(job).await {
            //         Ok(_) => {
            //             info!("Добавили задачу get_currencies")
            //         }
            //         Err(e) => return Err(e.into()),
            //     },
            //     Err(e) => return Err(e.into()),
            // };

            // match Job::new_async("*/7 * * * * *", |_, _| {
            //     Box::pin(async move {
            //         match api::tickers::get_tickers().await {
            //             Ok(tickers) => {
            //                 for ticker in tickers.iter() {
            //                     info!("Символ: {:?}", ticker.symbol);
            //                 }
            //             }
            //             Err(e) => {
            //                 error!("Ошибка при выполнении запроса: {}", e)
            //             }
            //         }
            //     })
            // }) {
            //     Ok(job) => match s.add(job).await {
            //         Ok(_) => {
            //             info!("Добавили задачу get_tickers")
            //         }
            //         Err(e) => return Err(e.into()),
            //     },
            //     Err(e) => return Err(e.into()),
            // }

            match Job::new_async("*/7 * * * * *", |_, _| {
                Box::pin(async move {
                    match api::loan::get_loan_market().await {
                        Ok(tickers) => {
                            for ticker in tickers.iter() {
                                info!("Символ: {:?}", ticker.currency);
                            }
                        }
                        Err(e) => {
                            error!("Ошибка при выполнении запроса: {}", e)
                        }
                    }
                })
            }) {
                Ok(job) => match s.add(job).await {
                    Ok(_) => {
                        info!("Добавили задачу get_loan_market")
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
