use super::errors::DomainResult;
use super::models::{Currency, Symbol, Ticker};
use async_trait::async_trait;

#[async_trait]
pub trait ExchangeDataSource: Send + Sync {
    async fn get_tickers(&self) -> DomainResult<Vec<Ticker>>;
    async fn get_symbols(&self) -> DomainResult<Vec<Symbol>>;
    async fn get_currencies(&self) -> DomainResult<Vec<Currency>>;
    async fn get_exchange_name(&self) -> &str;
}
