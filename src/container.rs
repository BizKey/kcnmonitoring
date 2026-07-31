// src/container.rs
use crate::application::sync_service::SyncService;
use crate::infrastructure::api::kucoin_data_source::KuCoinDataSource;
use crate::infrastructure::db::postgres_repositories::{
    PostgresCurrencyRepository, PostgresSymbolRepository, PostgresTickerRepository,
};
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

pub struct Container {
    pool: PgPool,
    ticker_repo: Arc<PostgresTickerRepository>,
    symbol_repo: Arc<PostgresSymbolRepository>,
    currency_repo: Arc<PostgresCurrencyRepository>,
    data_source: Arc<KuCoinDataSource>,
}

impl Container {
    pub async fn new(pool: PgPool) -> Result<Self> {
        let ticker_repo = Arc::new(PostgresTickerRepository::new(pool.clone()));
        let symbol_repo = Arc::new(PostgresSymbolRepository::new(pool.clone()));
        let currency_repo = Arc::new(PostgresCurrencyRepository::new(pool.clone()));
        let data_source = Arc::new(KuCoinDataSource::new());

        Ok(Self {
            pool,
            ticker_repo,
            symbol_repo,
            currency_repo,
            data_source,
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn ticker_repository(&self) -> Arc<PostgresTickerRepository> {
        self.ticker_repo.clone()
    }

    pub fn symbol_repository(&self) -> Arc<PostgresSymbolRepository> {
        self.symbol_repo.clone()
    }

    pub fn currency_repository(&self) -> Arc<PostgresCurrencyRepository> {
        self.currency_repo.clone()
    }

    pub fn data_source(&self) -> Arc<KuCoinDataSource> {
        self.data_source.clone()
    }

    pub fn create_sync_service(
        &self,
    ) -> SyncService<
        PostgresTickerRepository,
        PostgresSymbolRepository,
        PostgresCurrencyRepository,
        KuCoinDataSource,
    > {
        SyncService::new(
            self.ticker_repo.clone(),
            self.symbol_repo.clone(),
            self.currency_repo.clone(),
            self.data_source.clone(),
        )
    }
}
