use crate::domain::{entities::symbol::Symbol, repositories::symbol_repository::SymbolRepository};
use crate::infrastructure::api::client::get_client;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct FetchSymbolsUseCase<R: SymbolRepository> {
    repository: Arc<R>,
}

impl<R: SymbolRepository> Clone for FetchSymbolsUseCase<R> {
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
        }
    }
}

impl<R: SymbolRepository> FetchSymbolsUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn execute(
        &self,
        exchange: &str,
    ) -> Result<()> {
        info!("Starting fetch symbols for exchange: {}", exchange);

        let client = get_client()?;
        let symbols_api = match client.api_v2_symbols_get().await {
            Ok(Some(symbols)) => symbols,
            Ok(None) => {
                warn!("No symbols data received from API");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to fetch symbols: {}", e);
                return Err(e.into());
            }
        };

        let symbols: Vec<Symbol> = symbols_api
            .into_iter()
            .map(|s| {
                Symbol::new(
                    s.symbol,
                    s.name,
                    s.base_currency,
                    s.quote_currency,
                    s.fee_currency,
                    s.market,
                    s.base_min_size,
                    s.quote_min_size,
                    s.base_max_size,
                    s.quote_max_size,
                    s.base_increment,
                    s.quote_increment,
                    s.price_increment,
                    s.price_limit_rate,
                    s.min_funds,
                    s.is_margin_enabled,
                    s.enable_trading,
                    s.fee_category,
                    s.maker_fee_coefficient,
                    s.taker_fee_coefficient,
                    s.st,
                )
            })
            .collect();

        info!("Fetched {} symbols from API", symbols.len());

        if let Err(e) = self.repository.save(exchange, &symbols).await {
            error!("Failed to save symbols: {}", e);
            return Err(e.into());
        }

        info!("Successfully saved {} symbols to database", symbols.len());
        Ok(())
    }
}
