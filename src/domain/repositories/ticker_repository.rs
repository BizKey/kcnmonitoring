use crate::domain::entities::ticker::Ticker;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TickerReadRepository: Send + Sync {}

#[async_trait]
pub trait TickerWriteRepository: Send + Sync {
    async fn save(&self, exchange: &str, tickers: &[Ticker]) -> Result<()>;
}

#[async_trait]
pub trait TickerRepository: TickerReadRepository + TickerWriteRepository {}

impl<T> TickerRepository for T where T: TickerReadRepository + TickerWriteRepository {}
