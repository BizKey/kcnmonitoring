use crate::domain::entities::ticker::Ticker;
use crate::domain::repositories::ticker_repository::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::info;

pub struct PostgresTickerRepository {
    pool: PgPool,
}

impl PostgresTickerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TickerReadRepository for PostgresTickerRepository {}

#[async_trait]
impl TickerWriteRepository for PostgresTickerRepository {
    async fn save(&self, exchange: &str, tickers: &[Ticker]) -> Result<()> {
        let now = chrono::Utc::now();
        let total = tickers.len();

        for (index, ticker) in tickers.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO ticker (
                    exchange, symbol, symbol_name, 
                    taker_fee_rate, maker_fee_rate, 
                    taker_coefficient, maker_coefficient, 
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (exchange, symbol)
                DO UPDATE SET
                    symbol_name = EXCLUDED.symbol_name,
                    taker_fee_rate = EXCLUDED.taker_fee_rate,
                    maker_fee_rate = EXCLUDED.maker_fee_rate,
                    taker_coefficient = EXCLUDED.taker_coefficient,
                    maker_coefficient = EXCLUDED.maker_coefficient,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(exchange)
            .bind(&ticker.symbol)
            .bind(&ticker.symbol_name)
            .bind(&ticker.taker_fee_rate)
            .bind(&ticker.maker_fee_rate)
            .bind(&ticker.taker_coefficient)
            .bind(&ticker.maker_coefficient)
            .bind(now)
            .execute(&self.pool)
            .await
            .with_context(|| {
                format!(
                    "Failed to insert/update ticker at index {} with symbol '{}'",
                    index, ticker.symbol
                )
            })?;

            if (index + 1) % 500 == 0 || index + 1 == total {
                info!("Progress: {}/{} tickers processed", index + 1, total);
            }
        }

        info!(
            "Successfully processed {} tickers for exchange '{}'",
            total, exchange
        );
        Ok(())
    }
}
