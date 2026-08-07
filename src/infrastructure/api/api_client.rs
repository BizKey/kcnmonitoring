use crate::domain::entities::{currency::Currency, symbol::Symbol, ticker::Ticker};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ApiClient: Send + Sync {
    async fn fetch_currencies(&self) -> Result<Vec<Currency>>;

    async fn fetch_symbols(&self) -> Result<Vec<Symbol>>;

    async fn fetch_tickers(&self) -> Result<Vec<Ticker>>;
}
