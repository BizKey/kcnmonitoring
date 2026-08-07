use crate::domain::entities::symbol::Symbol;
use async_trait::async_trait;

#[async_trait]
pub trait SymbolRepository: Send + Sync {
    async fn save(
        &self,
        exchange: &str,
        symbols: &[Symbol],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
