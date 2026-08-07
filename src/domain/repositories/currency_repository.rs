use crate::domain::entities::currency::Currency;
use async_trait::async_trait;

#[async_trait]
pub trait CurrencyRepository: Send + Sync {
    async fn save(
        &self,
        exchange: &str,
        currencies: &[Currency],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
