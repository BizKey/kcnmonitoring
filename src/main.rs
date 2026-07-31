// src/main.rs
mod api {
    pub mod db;
    pub mod models;
    pub mod requests;
    pub mod tools;
}

mod application;
mod container;
mod domain;
mod infrastructure;

use crate::api::tools::get_env;
use crate::container::Container;
use anyhow::{Context, Result};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

const EXCHANGE: &str = "kucoin";
const CRON_EVERY_5_MIN: &str = "0 */5 * * * *";

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();
}

async fn create_db_pool() -> Result<sqlx::PgPool> {
    let database_url = get_env("DATABASE_URL").context("DATABASE_URL not set")?;

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .context("Failed to connect to database")
}

async fn create_sync_job(scheduler: &mut JobScheduler, container: &Container) -> Result<()> {
    let sync_service = container.create_sync_service();
    let job_name = "sync_all";

    let job = Job::new_async(CRON_EVERY_5_MIN, move |_, _| {
        let sync_service = sync_service.clone();
        Box::pin(async move {
            match sync_service.sync_all().await {
                Ok(stats) => {
                    info!(
                        "Sync completed: {} tickers, {} symbols, {} currencies",
                        stats.tickers_processed,
                        stats.symbols_processed,
                        stats.currencies_processed
                    );
                }
                Err(e) => {
                    error!("Sync failed: {}", e);
                }
            }
        })
    })?;

    scheduler.add(job).await?;
    info!("Added job: {}", job_name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    dotenv().ok();

    info!("Starting KuCoin data fetcher");
    info!("Exchange: {}", EXCHANGE);

    let pool = create_db_pool().await?;
    info!("Database connection pool created successfully");

    let container = Container::new(pool).await?;
    info!("Container initialized");

    // Создаем scheduler отдельно
    let mut scheduler = JobScheduler::new().await?;

    // Создаем задачу, передавая scheduler и container
    create_sync_job(&mut scheduler, &container).await?;

    // Запускаем scheduler
    scheduler.start().await?;
    info!("Scheduler started. All jobs will run every 5 minutes");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    info!("Shutting down gracefully...");
    scheduler.shutdown().await?;

    Ok(())
}
