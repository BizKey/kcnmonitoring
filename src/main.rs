use dotenv::dotenv;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
mod api {
    pub mod common;
    pub mod currencies;
    pub mod symbols;
    pub mod tickers;
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
    let sched = JobScheduler::new().await?;

    sched
        .add(Job::new_async("*/5 * * * * *", |_, _| {
            Box::pin(async move {
                match api::symbols::get_symbols().await {
                    Ok(symbols) => {
                        for symbol in symbols.iter() {
                            info!("Символ: {:?}", symbol.symbol);
                        }
                    }
                    Err(e) => {
                        error!("Ошибка при выполнении запроса: {}", e)
                    }
                }
            })
        })?)
        .await?;

    sched
        .add(Job::new_async("*/7 * * * * *", |_, _| {
            Box::pin(async move {
                match api::tickers::get_tickers().await {
                    Ok(tickers) => {
                        for ticker in tickers.iter() {
                            info!("Символ: {:?}", ticker.symbol);
                        }
                    }
                    Err(e) => {
                        error!("Ошибка при выполнении запроса: {}", e)
                    }
                }
            })
        })?)
        .await?;

    sched.start().await?;
    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
