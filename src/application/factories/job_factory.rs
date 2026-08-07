use crate::application::services::monitoring_service::MonitoringService;
use std::sync::Arc;
pub struct JobFactory {
    monitoring_service: Arc<dyn MonitoringService>,
    exchange: String,
}

impl JobFactory {
    pub fn new(monitoring_service: Arc<dyn MonitoringService>, exchange: String) -> Self {
        Self {
            monitoring_service,
            exchange,
        }
    }

    pub fn create_currencies_job(
        &self,
    ) -> impl Fn() -> futures::future::BoxFuture<'static, ()> + Send + Sync + Clone + 'static {
        let service = self.monitoring_service.clone();
        let exchange = self.exchange.clone();

        move || {
            let service = service.clone();
            let exchange = exchange.clone();
            Box::pin(async move {
                if let Err(e) = service.fetch_and_save_currencies(&exchange).await {
                    tracing::error!("Currency fetch failed: {}", e);
                }
            })
        }
    }

    pub fn create_symbols_job(
        &self,
    ) -> impl Fn() -> futures::future::BoxFuture<'static, ()> + Send + Sync + Clone + 'static {
        let service = self.monitoring_service.clone();
        let exchange = self.exchange.clone();

        move || {
            let service = service.clone();
            let exchange = exchange.clone();
            Box::pin(async move {
                if let Err(e) = service.fetch_and_save_symbols(&exchange).await {
                    tracing::error!("Symbol fetch failed: {}", e);
                }
            })
        }
    }

    pub fn create_tickers_job(
        &self,
    ) -> impl Fn() -> futures::future::BoxFuture<'static, ()> + Send + Sync + Clone + 'static {
        let service = self.monitoring_service.clone();
        let exchange = self.exchange.clone();

        move || {
            let service = service.clone();
            let exchange = exchange.clone();
            Box::pin(async move {
                if let Err(e) = service.fetch_and_save_tickers(&exchange).await {
                    tracing::error!("Ticker fetch failed: {}", e);
                }
            })
        }
    }
}
