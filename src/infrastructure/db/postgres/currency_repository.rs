use anyhow::Context;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::info;

use crate::domain::{
    entities::currency::Currency, repositories::currency_repository::CurrencyRepository,
};

pub struct PostgresCurrencyRepository {
    pool: PgPool,
}

impl PostgresCurrencyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurrencyRepository for PostgresCurrencyRepository {
    async fn save(
        &self,
        exchange: &str,
        currencies: &[Currency],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now();
        let total = currencies.len();

        for (index, currency) in currencies.iter().enumerate() {
            let result = sqlx::query(
                r#"
                INSERT INTO currency (
                    exchange, currency, currency_name, full_name, 
                    precision, is_margin_enabled, is_debit_enabled, 
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (exchange, currency)
                DO UPDATE SET
                    currency_name = EXCLUDED.currency_name,
                    full_name = EXCLUDED.full_name,
                    precision = EXCLUDED.precision,
                    is_margin_enabled = EXCLUDED.is_margin_enabled,
                    is_debit_enabled = EXCLUDED.is_debit_enabled,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(exchange)
            .bind(&currency.currency)
            .bind(&currency.name)
            .bind(&currency.full_name)
            .bind(currency.precision)
            .bind(currency.is_margin_enabled)
            .bind(currency.is_debit_enabled)
            .bind(now)
            .execute(&self.pool)
            .await
            .with_context(|| {
                format!(
                    "Failed to insert/update currency at index {} with currency '{}'",
                    index, currency.currency
                )
            })?;

            if (index + 1) % 500 == 0 || index + 1 == total {
                info!(
                    "Progress: {}/{} currencies processed ({} rows affected)",
                    index + 1,
                    total,
                    result.rows_affected()
                );
            }
        }

        info!(
            "Successfully processed {} currencies for exchange '{}'",
            total, exchange
        );
        Ok(())
    }
}
