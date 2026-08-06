use crate::db::models::{CurrenciesDb, SymbolDb, TickerDb};
use anyhow::{Context, Result};
use tracing::info;

pub async fn insert_tickers_to_db(
    pool: &sqlx::PgPool,
    exchange: &str,
    tickers: Vec<TickerDb>,
) -> Result<()> {
    let now = chrono::Utc::now();
    let total = tickers.len();

    for (index, ticker) in tickers.into_iter().enumerate() {
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
        .execute(pool)
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

pub async fn insert_currencies_to_db(
    pool: &sqlx::PgPool,
    exchange: &str,
    currencies: Vec<CurrenciesDb>,
) -> Result<()> {
    if currencies.is_empty() {
        info!("No currencies to insert");
        return Ok(());
    }

    let now = chrono::Utc::now();
    let total = currencies.len();

    for (index, currency) in currencies.into_iter().enumerate() {
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
        .execute(pool)
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

pub async fn insert_symbols_to_db(
    pool: &sqlx::PgPool,
    exchange: &str,
    symbols: Vec<SymbolDb>,
) -> Result<()> {
    let now = chrono::Utc::now();
    let total = symbols.len();

    for (index, symbol) in symbols.into_iter().enumerate() {
        let result = sqlx::query(
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
            "#
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
        .execute(pool)
        .await
        .with_context(|| {
            format!(
                "Failed to insert/update symbol at index {} with symbol '{}'",
                index, symbol.symbol
            )
        })?;

        if (index + 1) % 500 == 0 || index + 1 == total {
            info!(
                "Progress: {}/{} symbols processed ({} rows affected)",
                index + 1,
                total,
                result.rows_affected()
            );
        }
    }

    info!(
        "Successfully processed {} symbols for exchange '{}'",
        total, exchange
    );
    Ok(())
}
