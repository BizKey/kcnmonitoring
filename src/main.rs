mod application;
mod domain;
mod infrastructure;

use anyhow::Result;
use dotenvy::dotenv;

use application::scheduler::SchedulerService;
use infrastructure::{
    config::Config, db::postgres::connection::create_db_pool, di::container::Container,
    logging::init_tracing,
};

const CRON_EVERY_5_MIN: &str = "0 */5 * * * *";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    dotenv().ok();

    tracing::info!("Starting KuCoin data fetcher");

    // Загружаем конфигурацию
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Создаем пул соединений с БД
    let pool = create_db_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Строим DI контейнер
    let container = Container::build(config, pool).await?;
    tracing::info!("DI container built");

    // Создаем и настраиваем планировщик
    let mut scheduler = SchedulerService::new().await?;
    tracing::info!("Scheduler created");

    // Добавляем задачи через фабрику
    scheduler
        .add_job(
            CRON_EVERY_5_MIN,
            "Currencies fetcher",
            container.job_factory.create_currencies_job(),
        )
        .await?;

    scheduler
        .add_job(
            CRON_EVERY_5_MIN,
            "Symbols fetcher",
            container.job_factory.create_symbols_job(),
        )
        .await?;

    scheduler
        .add_job(
            CRON_EVERY_5_MIN,
            "Tickers fetcher",
            container.job_factory.create_tickers_job(),
        )
        .await?;

    // Запускаем планировщик
    scheduler.start().await?;

    // Ожидаем сигнал завершения
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    tracing::info!("Shutting down gracefully...");
    scheduler.shutdown().await?;

    Ok(())
}
