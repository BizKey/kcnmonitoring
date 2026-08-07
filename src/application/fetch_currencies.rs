use crate::domain::{
    entities::currency::Currency, repositories::currency_repository::CurrencyRepository,
};
use crate::infrastructure::api::client::get_client;
use std::sync::Arc;
use tracing::{error, info, warn};
pub struct FetchCurrenciesUseCase<R: CurrencyRepository> {
    repository: Arc<R>,
}

impl<R: CurrencyRepository> Clone for FetchCurrenciesUseCase<R> {
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
        }
    }
}

impl<R: CurrencyRepository> FetchCurrenciesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn execute(
        &self,
        exchange: &str,
    ) -> Result<()> {
        info!("Starting fetch currencies for exchange: {}", exchange);

        let client = get_client()?;
        let currencies_api = match client.api_v3_currencies_get().await {
            Ok(Some(currencies)) => currencies,
            Ok(None) => {
                warn!("No currencies data received from API");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to fetch currencies: {}", e);
                return Err(e.into());
            }
        };

        let currencies: Vec<Currency> = currencies_api
            .into_iter()
            .map(|c| {
                Currency::new(
                    c.currency,
                    c.name,
                    c.full_name,
                    c.precision,
                    c.is_margin_enabled,
                    c.is_debit_enabled,
                )
            })
            .collect();

        info!("Fetched {} currencies from API", currencies.len());

        if let Err(e) = self.repository.save(exchange, &currencies).await {
            error!("Failed to save currencies: {}", e);
            return Err(e.into());
        }

        info!(
            "Successfully saved {} currencies to database",
            currencies.len()
        );
        Ok(())
    }
}
