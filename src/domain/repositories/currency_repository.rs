use crate::domain::entities::currency::Currency;
use anyhow::Result;
use async_trait::async_trait;
#[async_trait]
pub trait CurrencyReadRepository: Send + Sync {}

#[async_trait]
pub trait CurrencyWriteRepository: Send + Sync {
    async fn save(&self, exchange: &str, currencies: &[Currency]) -> Result<()>;
}

#[async_trait]
pub trait CurrencyRepository: CurrencyReadRepository + CurrencyWriteRepository {}

impl<T> CurrencyRepository for T where T: CurrencyReadRepository + CurrencyWriteRepository {}
