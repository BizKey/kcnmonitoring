use crate::domain::repositories::currency_repository::CurrencyRepository;
use crate::domain::repositories::symbol_repository::SymbolRepository;
use crate::domain::repositories::ticker_repository::TickerRepository;
use crate::infrastructure::api::api_client::ApiClient;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

#[async_trait]
pub trait MonitoringService: Send + Sync {
    async fn fetch_and_save_currencies(&self, exchange: &str) -> Result<()>;
    async fn fetch_and_save_symbols(&self, exchange: &str) -> Result<()>;
    async fn fetch_and_save_tickers(&self, exchange: &str) -> Result<()>;
}

pub struct MonitoringServiceImpl {
    api_client: Arc<dyn ApiClient>,
    currency_repo: Arc<dyn CurrencyRepository>,
    symbol_repo: Arc<dyn SymbolRepository>,
    ticker_repo: Arc<dyn TickerRepository>,
}

impl MonitoringServiceImpl {
    pub fn new(
        api_client: Arc<dyn ApiClient>,
        currency_repo: Arc<dyn CurrencyRepository>,
        symbol_repo: Arc<dyn SymbolRepository>,
        ticker_repo: Arc<dyn TickerRepository>,
    ) -> Self {
        Self {
            api_client,
            currency_repo,
            symbol_repo,
            ticker_repo,
        }
    }
}

#[async_trait]
impl MonitoringService for MonitoringServiceImpl {
    async fn fetch_and_save_currencies(&self, exchange: &str) -> Result<()> {
        info!("Fetching currencies for exchange: {}", exchange);
        let currencies = self.api_client.fetch_currencies().await?;
        self.currency_repo.save(exchange, &currencies).await?;
        info!("Saved {} currencies", currencies.len());
        Ok(())
    }

    async fn fetch_and_save_symbols(&self, exchange: &str) -> Result<()> {
        info!("Fetching symbols for exchange: {}", exchange);
        let symbols = self.api_client.fetch_symbols().await?;
        self.symbol_repo.save(exchange, &symbols).await?;
        info!("Saved {} symbols", symbols.len());
        Ok(())
    }

    async fn fetch_and_save_tickers(&self, exchange: &str) -> Result<()> {
        info!("Fetching tickers for exchange: {}", exchange);
        let tickers = self.api_client.fetch_tickers().await?;
        self.ticker_repo.save(exchange, &tickers).await?;
        info!("Saved {} tickers", tickers.len());
        Ok(())
    }
}
