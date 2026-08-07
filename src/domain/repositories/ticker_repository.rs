use crate::domain::entities::ticker::Ticker;
use async_trait::async_trait;

#[async_trait]
pub trait TickerRepository: Send + Sync {
    async fn save(
        &self,
        exchange: &str,
        tickers: &[Ticker],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
