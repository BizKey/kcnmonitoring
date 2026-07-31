use super::errors::DomainResult;
use super::models::{Currency, Symbol, Ticker};
use async_trait::async_trait;

#[async_trait]
pub trait TickerRepository: Send + Sync {
    async fn save_tickers(&self, exchange: &str, tickers: Vec<Ticker>) -> DomainResult<usize>;
    async fn get_ticker(&self, exchange: &str, symbol: &str) -> DomainResult<Option<Ticker>>;
    async fn get_all_tickers(&self, exchange: &str) -> DomainResult<Vec<Ticker>>;
    async fn delete_old_tickers(
        &self,
        exchange: &str,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> DomainResult<usize>;
}

#[async_trait]
pub trait SymbolRepository: Send + Sync {
    async fn save_symbols(&self, exchange: &str, symbols: Vec<Symbol>) -> DomainResult<usize>;
    async fn get_symbol(&self, exchange: &str, symbol: &str) -> DomainResult<Option<Symbol>>;
    async fn get_all_symbols(&self, exchange: &str) -> DomainResult<Vec<Symbol>>;
}

#[async_trait]
pub trait CurrencyRepository: Send + Sync {
    async fn save_currencies(
        &self,
        exchange: &str,
        currencies: Vec<Currency>,
    ) -> DomainResult<usize>;
    async fn get_currency(&self, exchange: &str, currency: &str) -> DomainResult<Option<Currency>>;
    async fn get_all_currencies(&self, exchange: &str) -> DomainResult<Vec<Currency>>;
}
