use crate::domain::entities::symbol::Symbol;
use crate::domain::repositories::symbol_repository::{SymbolReadRepository, SymbolWriteRepository};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::info;

pub struct PostgresSymbolRepository {
    pool: PgPool,
}

impl PostgresSymbolRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SymbolReadRepository for PostgresSymbolRepository {}

#[async_trait]
impl SymbolWriteRepository for PostgresSymbolRepository {
    async fn save(&self, exchange: &str, symbols: &[Symbol]) -> Result<()> {
        let now = chrono::Utc::now();
        let total = symbols.len();

        for (index, symbol) in symbols.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO symbol (
                    exchange, symbol, symbol_name, base_currency, quote_currency, fee_currency,
                    market, base_min_size, quote_min_size, base_max_size, quote_max_size,
                    base_increment, quote_increment, price_increment, price_limit_rate,
                    min_funds, is_margin_enabled, enable_trading, fee_category,
                    maker_fee_coefficient, taker_fee_coefficient, st, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
                ON CONFLICT (exchange, symbol)
                DO UPDATE SET
                    symbol_name = EXCLUDED.symbol_name,
                    base_currency = EXCLUDED.base_currency,
                    quote_currency = EXCLUDED.quote_currency,
                    fee_currency = EXCLUDED.fee_currency,
                    market = EXCLUDED.market,
                    base_min_size = EXCLUDED.base_min_size,
                    quote_min_size = EXCLUDED.quote_min_size,
                    base_max_size = EXCLUDED.base_max_size,
                    quote_max_size = EXCLUDED.quote_max_size,
                    base_increment = EXCLUDED.base_increment,
                    quote_increment = EXCLUDED.quote_increment,
                    price_increment = EXCLUDED.price_increment,
                    price_limit_rate = EXCLUDED.price_limit_rate,
                    min_funds = EXCLUDED.min_funds,
                    is_margin_enabled = EXCLUDED.is_margin_enabled,
                    enable_trading = EXCLUDED.enable_trading,
                    fee_category = EXCLUDED.fee_category,
                    maker_fee_coefficient = EXCLUDED.maker_fee_coefficient,
                    taker_fee_coefficient = EXCLUDED.taker_fee_coefficient,
                    st = EXCLUDED.st,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(exchange)
            .bind(&symbol.symbol)
            .bind(&symbol.name)
            .bind(&symbol.base_currency)
            .bind(&symbol.quote_currency)
            .bind(&symbol.fee_currency)
            .bind(&symbol.market)
            .bind(&symbol.base_min_size)
            .bind(&symbol.quote_min_size)
            .bind(&symbol.base_max_size)
            .bind(&symbol.quote_max_size)
            .bind(&symbol.base_increment)
            .bind(&symbol.quote_increment)
            .bind(&symbol.price_increment)
            .bind(&symbol.price_limit_rate)
            .bind(&symbol.min_funds)
            .bind(symbol.is_margin_enabled)
            .bind(symbol.enable_trading)
            .bind(&symbol.fee_category)
            .bind(&symbol.maker_fee_coefficient)
            .bind(&symbol.taker_fee_coefficient)
            .bind(symbol.st)
            .bind(now)
            .execute(&self.pool)
            .await
            .with_context(|| {
                format!(
                    "Failed to insert/update symbol at index {} with symbol '{}'",
                    index, symbol.symbol
                )
            })?;

            if (index + 1) % 500 == 0 || index + 1 == total {
                info!("Progress: {}/{} symbols processed", index + 1, total);
            }
        }

        info!(
            "Successfully processed {} symbols for exchange '{}'",
            total, exchange
        );
        Ok(())
    }
}
