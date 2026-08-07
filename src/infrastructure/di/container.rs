use crate::application::factories::job_factory::JobFactory;
use crate::application::services::monitoring_service::{MonitoringService, MonitoringServiceImpl};
use crate::domain::repositories::currency_repository::CurrencyRepository;
use crate::domain::repositories::symbol_repository::SymbolRepository;
use crate::domain::repositories::ticker_repository::TickerRepository;
use crate::infrastructure::api::api_client::ApiClient;
use crate::infrastructure::api::kucoin_client::KuCoinClient;
use crate::infrastructure::config::Config;
use crate::infrastructure::db::postgres::currency_repository::PostgresCurrencyRepository;
use crate::infrastructure::db::postgres::symbol_repository::PostgresSymbolRepository;
use crate::infrastructure::db::postgres::ticker_repository::PostgresTickerRepository;
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
pub struct Container {
    pub config: Config,
    pub api_client: Arc<dyn ApiClient>,
    pub currency_repo: Arc<dyn CurrencyRepository>,
    pub symbol_repo: Arc<dyn SymbolRepository>,
    pub ticker_repo: Arc<dyn TickerRepository>,
    pub monitoring_service: Arc<dyn MonitoringService>,
    pub job_factory: JobFactory,
}

impl Container {
    pub async fn build(config: Config, pool: PgPool) -> Result<Self> {
        // Создаем API клиент
        let api_client = Arc::new(KuCoinClient::new(&config)?);

        // Создаем репозитории
        let currency_repo = Arc::new(PostgresCurrencyRepository::new(pool.clone()));
        let symbol_repo = Arc::new(PostgresSymbolRepository::new(pool.clone()));
        let ticker_repo = Arc::new(PostgresTickerRepository::new(pool.clone()));

        // Создаем сервис мониторинга
        let monitoring_service = Arc::new(MonitoringServiceImpl::new(
            api_client.clone(),
            currency_repo.clone(),
            symbol_repo.clone(),
            ticker_repo.clone(),
        ));

        // Создаем фабрику задач
        let job_factory = JobFactory::new(monitoring_service.clone(), "kucoin".to_string());

        Ok(Self {
            config,
            api_client,
            currency_repo,
            symbol_repo,
            ticker_repo,
            monitoring_service,
            job_factory,
        })
    }
}
