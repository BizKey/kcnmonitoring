// src/application/sync_service.rs
use crate::domain::{
    CurrencyRepository, DomainResult, ExchangeDataSource, SymbolRepository, SyncStats,
    TickerRepository,
};
use std::sync::Arc;
use tracing::{error, info};

pub struct SyncService<T, S, C, D>
where
    T: TickerRepository,
    S: SymbolRepository,
    C: CurrencyRepository,
    D: ExchangeDataSource,
{
    ticker_repo: Arc<T>,
    symbol_repo: Arc<S>,
    currency_repo: Arc<C>,
    data_source: Arc<D>,
}

impl<T, S, C, D> SyncService<T, S, C, D>
where
    T: TickerRepository,
    S: SymbolRepository,
    C: CurrencyRepository,
    D: ExchangeDataSource,
{
    pub fn new(
        ticker_repo: Arc<T>,
        symbol_repo: Arc<S>,
        currency_repo: Arc<C>,
        data_source: Arc<D>,
    ) -> Self {
        Self {
            ticker_repo,
            symbol_repo,
            currency_repo,
            data_source,
        }
    }

    pub async fn sync_all(&self) -> DomainResult<SyncStats> {
        let exchange = self.data_source.get_exchange_name().await;
        info!("Starting sync for exchange: {}", exchange);

        let mut stats = SyncStats::new();

        match self.sync_tickers(exchange).await {
            Ok(count) => {
                stats.tickers_processed = count;
                info!("Synced {} tickers for {}", count, exchange);
            }
            Err(e) => {
                error!("Failed to sync tickers for {}: {}", exchange, e);
                return Err(e);
            }
        }

        match self.sync_symbols(exchange).await {
            Ok(count) => {
                stats.symbols_processed = count;
                info!("Synced {} symbols for {}", count, exchange);
            }
            Err(e) => {
                error!("Failed to sync symbols for {}: {}", exchange, e);
                return Err(e);
            }
        }

        match self.sync_currencies(exchange).await {
            Ok(count) => {
                stats.currencies_processed = count;
                info!("Synced {} currencies for {}", count, exchange);
            }
            Err(e) => {
                error!("Failed to sync currencies for {}: {}", exchange, e);
                return Err(e);
            }
        }

        Ok(stats)
    }

    async fn sync_tickers(&self, exchange: &str) -> DomainResult<usize> {
        let tickers = self.data_source.get_tickers().await?;
        let count = self.ticker_repo.save_tickers(exchange, tickers).await?;
        Ok(count)
    }

    async fn sync_symbols(&self, exchange: &str) -> DomainResult<usize> {
        let symbols = self.data_source.get_symbols().await?;
        let count = self.symbol_repo.save_symbols(exchange, symbols).await?;
        Ok(count)
    }

    async fn sync_currencies(&self, exchange: &str) -> DomainResult<usize> {
        let currencies = self.data_source.get_currencies().await?;
        let count = self
            .currency_repo
            .save_currencies(exchange, currencies)
            .await?;
        Ok(count)
    }
}

// Добавляем Clone, чтобы можно было клонировать для задач
impl<T, S, C, D> Clone for SyncService<T, S, C, D>
where
    T: TickerRepository,
    S: SymbolRepository,
    C: CurrencyRepository,
    D: ExchangeDataSource,
{
    fn clone(&self) -> Self {
        Self {
            ticker_repo: self.ticker_repo.clone(),
            symbol_repo: self.symbol_repo.clone(),
            currency_repo: self.currency_repo.clone(),
            data_source: self.data_source.clone(),
        }
    }
}
