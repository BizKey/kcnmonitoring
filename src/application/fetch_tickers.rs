use crate::domain::{entities::ticker::Ticker, repositories::ticker_repository::TickerRepository};
use crate::infrastructure::api::client::get_client;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct FetchTickersUseCase<R: TickerRepository> {
    repository: Arc<R>,
}

impl<R: TickerRepository> Clone for FetchTickersUseCase<R> {
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
        }
    }
}

impl<R: TickerRepository> FetchTickersUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn execute(
        &self,
        exchange: &str,
    ) -> Result<()> {
        info!("Starting fetch tickers for exchange: {}", exchange);

        let client = get_client()?;
        let ticker_data = match client.api_v1_market_all_tickers_get().await {
            Ok(Some(data)) => data,
            Ok(None) => {
                warn!("No tickers data received from API");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to fetch tickers: {}", e);
                return Err(e.into());
            }
        };

        let tickers: Vec<Ticker> = ticker_data
            .ticker
            .into_iter()
            .map(|t| {
                Ticker::new(
                    t.symbol,
                    t.symbol_name,
                    t.taker_fee_rate,
                    t.maker_fee_rate,
                    t.taker_coefficient,
                    t.maker_coefficient,
                )
            })
            .collect();

        info!("Fetched {} tickers from API", tickers.len());

        if let Err(e) = self.repository.save(exchange, &tickers).await {
            error!("Failed to save tickers: {}", e);
            return Err(e.into());
        }

        info!("Successfully saved {} tickers to database", tickers.len());
        Ok(())
    }
}
