use crate::domain::entities::symbol::Symbol;
use anyhow::Result;
use async_trait::async_trait;
#[async_trait]
pub trait SymbolReadRepository: Send + Sync {}

#[async_trait]
pub trait SymbolWriteRepository: Send + Sync {
    async fn save(&self, exchange: &str, symbols: &[Symbol]) -> Result<()>;
}

#[async_trait]
pub trait SymbolRepository: SymbolReadRepository + SymbolWriteRepository {}

impl<T> SymbolRepository for T where T: SymbolReadRepository + SymbolWriteRepository {}
