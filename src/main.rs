mod application;
mod domain;
mod infrastructure;

use anyhow::Result;
use dotenvy::dotenv;

use application::{
    fetch_currencies::FetchCurrenciesUseCase, fetch_symbols::FetchSymbolsUseCase,
    fetch_tickers::FetchTickersUseCase, scheduler::SchedulerService,
};
use infrastructure::{
    api::client::init_client,
    config::Config,
    db::postgres::{
        connection::create_db_pool, currency_repository::PostgresCurrencyRepository,
        symbol_repository::PostgresSymbolRepository, ticker_repository::PostgresTickerRepository,
    },
    logging::init_tracing,
};

const EXCHANGE: &str = "kucoin";
const CRON_EVERY_5_MIN: &str = "0 */5 * * * *";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    dotenv().ok();

    tracing::info!("Starting KuCoin data fetcher");
    tracing::info!("Exchange: {}", EXCHANGE);

    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    init_client(&config)?;
    tracing::info!("API client initialized");

    let pool = create_db_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    let currency_repo = PostgresCurrencyRepository::new(pool.clone());
    let symbol_repo = PostgresSymbolRepository::new(pool.clone());
    let ticker_repo = PostgresTickerRepository::new(pool.clone());

    let fetch_currencies = FetchCurrenciesUseCase::new(currency_repo);
    let fetch_symbols = FetchSymbolsUseCase::new(symbol_repo);
    let fetch_tickers = FetchTickersUseCase::new(ticker_repo);

    let mut scheduler = SchedulerService::new().await?;
    tracing::info!("Scheduler created");

    scheduler
        .add_job(CRON_EVERY_5_MIN, "Currencies fetcher", move || {
            let use_case = fetch_currencies.clone();
            async move {
                if let Err(e) = use_case.execute(EXCHANGE).await {
                    tracing::error!("Currency fetch failed: {}", e);
                }
            }
        })
        .await?;

    scheduler
        .add_job(CRON_EVERY_5_MIN, "Symbols fetcher", move || {
            let use_case = fetch_symbols.clone();
            async move {
                if let Err(e) = use_case.execute(EXCHANGE).await {
                    tracing::error!("Symbol fetch failed: {}", e);
                }
            }
        })
        .await?;

    scheduler
        .add_job(CRON_EVERY_5_MIN, "Tickers fetcher", move || {
            let use_case = fetch_tickers.clone();
            async move {
                if let Err(e) = use_case.execute(EXCHANGE).await {
                    tracing::error!("Ticker fetch failed: {}", e);
                }
            }
        })
        .await?;

    scheduler.start().await?;

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    tracing::info!("Shutting down gracefully...");
    scheduler.shutdown().await?;

    Ok(())
}
