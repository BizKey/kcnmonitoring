// src/infrastructure/db/postgres_repositories.rs
use crate::domain::{
    Currency as DomainCurrency, CurrencyRepository, DomainResult, Symbol as DomainSymbol,
    SymbolRepository, Ticker as DomainTicker, TickerRepository,
};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::info;

#[derive(sqlx::FromRow)]
struct SymbolRecord {
    symbol: String,
    name: String,
    base_currency: String,
    quote_currency: String,
    fee_currency: String,
    market: String,
    base_min_size: String,
    quote_min_size: String,
    base_max_size: String,
    quote_max_size: String,
    base_increment: String,
    quote_increment: String,
    price_increment: String,
    price_limit_rate: String,
    min_funds: Option<String>,
    is_margin_enabled: bool,
    enable_trading: bool,
    fee_category: i16,
    maker_fee_coefficient: String,
    taker_fee_coefficient: String,
    st: bool,
}

// Вспомогательная структура для запроса Currency
#[derive(sqlx::FromRow)]
struct CurrencyRecord {
    currency: String,
    name: String,
    full_name: String,
    precision: i16,
    is_margin_enabled: bool,
    is_debit_enabled: bool,
}

// Вспомогательная структура для запроса Ticker
#[derive(sqlx::FromRow)]
struct TickerRecord {
    symbol: String,
    symbol_name: String,
    taker_fee_rate: String,
    maker_fee_rate: String,
    taker_coefficient: String,
    maker_coefficient: String,
}

pub struct PostgresTickerRepository {
    pool: PgPool,
}

impl PostgresTickerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TickerRepository for PostgresTickerRepository {
    async fn save_tickers(
        &self,
        exchange: &str,
        tickers: Vec<DomainTicker>,
    ) -> DomainResult<usize> {
        if tickers.is_empty() {
            return Ok(0);
        }

        let now = chrono::Utc::now();
        let total = tickers.len();

        for chunk in tickers.chunks(500) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO ticker (exchange, symbol, symbol_name, taker_fee_rate, maker_fee_rate, taker_coefficient, maker_coefficient, updated_at) ",
            );

            query_builder.push_values(chunk, |mut b, ticker| {
                b.push_bind(exchange)
                    .push_bind(&ticker.symbol)
                    .push_bind(&ticker.symbol_name)
                    .push_bind(&ticker.taker_fee_rate)
                    .push_bind(&ticker.maker_fee_rate)
                    .push_bind(&ticker.taker_coefficient)
                    .push_bind(&ticker.maker_coefficient)
                    .push_bind(now);
            });

            query_builder.push(
                " ON CONFLICT (exchange, symbol) DO UPDATE SET \
                 symbol_name = EXCLUDED.symbol_name, \
                 taker_fee_rate = EXCLUDED.taker_fee_rate, \
                 maker_fee_rate = EXCLUDED.maker_fee_rate, \
                 taker_coefficient = EXCLUDED.taker_coefficient, \
                 maker_coefficient = EXCLUDED.maker_coefficient, \
                 updated_at = CURRENT_TIMESTAMP",
            );

            query_builder.build().execute(&self.pool).await?;
        }

        info!("Saved {} tickers for exchange '{}'", total, exchange);
        Ok(total)
    }

    async fn get_ticker(&self, exchange: &str, symbol: &str) -> DomainResult<Option<DomainTicker>> {
        let row: Option<(String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT symbol, symbol_name, taker_fee_rate, maker_fee_rate, 
                   taker_coefficient, maker_coefficient
            FROM ticker
            WHERE exchange = $1 AND symbol = $2
            "#,
        )
        .bind(exchange)
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                symbol,
                symbol_name,
                taker_fee_rate,
                maker_fee_rate,
                taker_coefficient,
                maker_coefficient,
            )| DomainTicker {
                symbol,
                symbol_name,
                taker_fee_rate,
                maker_fee_rate,
                taker_coefficient,
                maker_coefficient,
            },
        ))
    }

    async fn get_all_tickers(&self, exchange: &str) -> DomainResult<Vec<DomainTicker>> {
        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT symbol, symbol_name, taker_fee_rate, maker_fee_rate,
                   taker_coefficient, maker_coefficient
            FROM ticker
            WHERE exchange = $1
            "#,
        )
        .bind(exchange)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    symbol,
                    symbol_name,
                    taker_fee_rate,
                    maker_fee_rate,
                    taker_coefficient,
                    maker_coefficient,
                )| DomainTicker {
                    symbol,
                    symbol_name,
                    taker_fee_rate,
                    maker_fee_rate,
                    taker_coefficient,
                    maker_coefficient,
                },
            )
            .collect())
    }

    async fn delete_old_tickers(
        &self,
        exchange: &str,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> DomainResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM ticker
            WHERE exchange = $1 AND updated_at < $2
            "#,
        )
        .bind(exchange)
        .bind(older_than)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }
}

// Реализация для Symbol
pub struct PostgresSymbolRepository {
    pool: PgPool,
}

impl PostgresSymbolRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SymbolRepository for PostgresSymbolRepository {
    async fn save_symbols(
        &self,
        exchange: &str,
        symbols: Vec<DomainSymbol>,
    ) -> DomainResult<usize> {
        if symbols.is_empty() {
            return Ok(0);
        }

        let now = chrono::Utc::now();
        let total = symbols.len();

        for chunk in symbols.chunks(500) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO symbol (exchange, symbol, symbol_name, base_currency, quote_currency, fee_currency, market, base_min_size, quote_min_size, base_max_size, quote_max_size, base_increment, quote_increment, price_increment, price_limit_rate, min_funds, is_margin_enabled, enable_trading, fee_category, maker_fee_coefficient, taker_fee_coefficient, st, updated_at) ",
            );

            query_builder.push_values(chunk, |mut b, symbol| {
                b.push_bind(exchange)
                    .push_bind(&symbol.symbol)
                    .push_bind(&symbol.name)
                    .push_bind(&symbol.base_currency)
                    .push_bind(&symbol.quote_currency)
                    .push_bind(&symbol.fee_currency)
                    .push_bind(&symbol.market)
                    .push_bind(&symbol.base_min_size)
                    .push_bind(&symbol.quote_min_size)
                    .push_bind(&symbol.base_max_size)
                    .push_bind(&symbol.quote_max_size)
                    .push_bind(&symbol.base_increment)
                    .push_bind(&symbol.quote_increment)
                    .push_bind(&symbol.price_increment)
                    .push_bind(&symbol.price_limit_rate)
                    .push_bind(&symbol.min_funds)
                    .push_bind(symbol.is_margin_enabled)
                    .push_bind(symbol.enable_trading)
                    .push_bind(symbol.fee_category)
                    .push_bind(&symbol.maker_fee_coefficient)
                    .push_bind(&symbol.taker_fee_coefficient)
                    .push_bind(symbol.st)
                    .push_bind(now);
            });

            query_builder.push(
                " ON CONFLICT (exchange, symbol) DO UPDATE SET \
                 symbol_name = EXCLUDED.symbol_name, \
                 base_currency = EXCLUDED.base_currency, \
                 quote_currency = EXCLUDED.quote_currency, \
                 fee_currency = EXCLUDED.fee_currency, \
                 market = EXCLUDED.market, \
                 base_min_size = EXCLUDED.base_min_size, \
                 quote_min_size = EXCLUDED.quote_min_size, \
                 base_max_size = EXCLUDED.base_max_size, \
                 quote_max_size = EXCLUDED.quote_max_size, \
                 base_increment = EXCLUDED.base_increment, \
                 quote_increment = EXCLUDED.quote_increment, \
                 price_increment = EXCLUDED.price_increment, \
                 price_limit_rate = EXCLUDED.price_limit_rate, \
                 min_funds = EXCLUDED.min_funds, \
                 is_margin_enabled = EXCLUDED.is_margin_enabled, \
                 enable_trading = EXCLUDED.enable_trading, \
                 fee_category = EXCLUDED.fee_category, \
                 maker_fee_coefficient = EXCLUDED.maker_fee_coefficient, \
                 taker_fee_coefficient = EXCLUDED.taker_fee_coefficient, \
                 st = EXCLUDED.st, \
                 updated_at = CURRENT_TIMESTAMP",
            );

            query_builder.build().execute(&self.pool).await?;
        }

        info!("Saved {} symbols for exchange '{}'", total, exchange);
        Ok(total)
    }

    async fn get_symbol(&self, exchange: &str, symbol: &str) -> DomainResult<Option<DomainSymbol>> {
        let record: Option<SymbolRecord> = sqlx::query_as(
            r#"
            SELECT symbol, name, base_currency, quote_currency, fee_currency,
                   market, base_min_size, quote_min_size, base_max_size, quote_max_size,
                   base_increment, quote_increment, price_increment, price_limit_rate,
                   min_funds, is_margin_enabled, enable_trading, fee_category,
                   maker_fee_coefficient, taker_fee_coefficient, st
            FROM symbol
            WHERE exchange = $1 AND symbol = $2
            "#,
        )
        .bind(exchange)
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| DomainSymbol {
            symbol: r.symbol,
            name: r.name,
            base_currency: r.base_currency,
            quote_currency: r.quote_currency,
            fee_currency: r.fee_currency,
            market: r.market,
            base_min_size: r.base_min_size,
            quote_min_size: r.quote_min_size,
            base_max_size: r.base_max_size,
            quote_max_size: r.quote_max_size,
            base_increment: r.base_increment,
            quote_increment: r.quote_increment,
            price_increment: r.price_increment,
            price_limit_rate: r.price_limit_rate,
            min_funds: r.min_funds,
            is_margin_enabled: r.is_margin_enabled,
            enable_trading: r.enable_trading,
            fee_category: r.fee_category,
            maker_fee_coefficient: r.maker_fee_coefficient,
            taker_fee_coefficient: r.taker_fee_coefficient,
            st: r.st,
        }))
    }

    async fn get_all_symbols(&self, exchange: &str) -> DomainResult<Vec<DomainSymbol>> {
        let records: Vec<SymbolRecord> = sqlx::query_as(
            r#"
            SELECT symbol, name, base_currency, quote_currency, fee_currency,
                   market, base_min_size, quote_min_size, base_max_size, quote_max_size,
                   base_increment, quote_increment, price_increment, price_limit_rate,
                   min_funds, is_margin_enabled, enable_trading, fee_category,
                   maker_fee_coefficient, taker_fee_coefficient, st
            FROM symbol
            WHERE exchange = $1
            "#,
        )
        .bind(exchange)
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| DomainSymbol {
                symbol: r.symbol,
                name: r.name,
                base_currency: r.base_currency,
                quote_currency: r.quote_currency,
                fee_currency: r.fee_currency,
                market: r.market,
                base_min_size: r.base_min_size,
                quote_min_size: r.quote_min_size,
                base_max_size: r.base_max_size,
                quote_max_size: r.quote_max_size,
                base_increment: r.base_increment,
                quote_increment: r.quote_increment,
                price_increment: r.price_increment,
                price_limit_rate: r.price_limit_rate,
                min_funds: r.min_funds,
                is_margin_enabled: r.is_margin_enabled,
                enable_trading: r.enable_trading,
                fee_category: r.fee_category,
                maker_fee_coefficient: r.maker_fee_coefficient,
                taker_fee_coefficient: r.taker_fee_coefficient,
                st: r.st,
            })
            .collect())
    }
}

// Реализация для Currency
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
    async fn save_currencies(
        &self,
        exchange: &str,
        currencies: Vec<DomainCurrency>,
    ) -> DomainResult<usize> {
        if currencies.is_empty() {
            return Ok(0);
        }

        let now = chrono::Utc::now();
        let total = currencies.len();

        for chunk in currencies.chunks(500) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO currency (exchange, currency, currency_name, full_name, precision, is_margin_enabled, is_debit_enabled, updated_at) ",
            );

            query_builder.push_values(chunk, |mut b, currency| {
                b.push_bind(exchange)
                    .push_bind(&currency.currency)
                    .push_bind(&currency.name)
                    .push_bind(&currency.full_name)
                    .push_bind(currency.precision)
                    .push_bind(currency.is_margin_enabled)
                    .push_bind(currency.is_debit_enabled)
                    .push_bind(now);
            });

            query_builder.push(
                " ON CONFLICT (exchange, currency) DO UPDATE SET \
                 currency_name = EXCLUDED.currency_name, \
                 full_name = EXCLUDED.full_name, \
                 precision = EXCLUDED.precision, \
                 is_margin_enabled = EXCLUDED.is_margin_enabled, \
                 is_debit_enabled = EXCLUDED.is_debit_enabled, \
                 updated_at = CURRENT_TIMESTAMP",
            );

            query_builder.build().execute(&self.pool).await?;
        }

        info!("Saved {} currencies for exchange '{}'", total, exchange);
        Ok(total)
    }

    async fn get_currency(
        &self,
        exchange: &str,
        currency: &str,
    ) -> DomainResult<Option<DomainCurrency>> {
        let record: Option<CurrencyRecord> = sqlx::query_as(
            r#"
            SELECT currency, name, full_name, precision,
                   is_margin_enabled, is_debit_enabled
            FROM currency
            WHERE exchange = $1 AND currency = $2
            "#,
        )
        .bind(exchange)
        .bind(currency)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| DomainCurrency {
            currency: r.currency,
            name: r.name,
            full_name: r.full_name,
            precision: r.precision,
            is_margin_enabled: r.is_margin_enabled,
            is_debit_enabled: r.is_debit_enabled,
        }))
    }

    async fn get_all_currencies(&self, exchange: &str) -> DomainResult<Vec<DomainCurrency>> {
        let records: Vec<CurrencyRecord> = sqlx::query_as(
            r#"
            SELECT currency, name, full_name, precision,
                   is_margin_enabled, is_debit_enabled
            FROM currency
            WHERE exchange = $1
            "#,
        )
        .bind(exchange)
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| DomainCurrency {
                currency: r.currency,
                name: r.name,
                full_name: r.full_name,
                precision: r.precision,
                is_margin_enabled: r.is_margin_enabled,
                is_debit_enabled: r.is_debit_enabled,
            })
            .collect())
    }
}
