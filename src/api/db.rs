use crate::api::models::{Currencies, Symbol, TickerData};
use sqlx::{Postgres, QueryBuilder};

pub async fn insert_tickers_to_db(
    pool: sqlx::PgPool,
    exchange: &str,
    tickers: TickerData,
) -> Result<(), String> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO ticker 
                (exchange, symbol, symbol_name, taker_fee_rate, maker_fee_rate, taker_coefficient, maker_coefficient, updated_at)",
    );

    query_builder.push_values(&tickers.ticker, |mut b, ticker| {
        b.push_bind(exchange)
            .push_bind(&ticker.symbol)
            .push_bind(&ticker.symbol_name)
            .push_bind(&ticker.taker_fee_rate)
            .push_bind(&ticker.maker_fee_rate)
            .push_bind(&ticker.taker_coefficient)
            .push_bind(&ticker.maker_coefficient)
            .push_bind(chrono::Utc::now());
    });

    query_builder.push(
        " ON CONFLICT (exchange, symbol)
                DO UPDATE SET
                    symbol_name = EXCLUDED.symbol_name,
                    taker_fee_rate = EXCLUDED.taker_fee_rate,
                    maker_fee_rate = EXCLUDED.maker_fee_rate,
                    taker_coefficient = EXCLUDED.taker_coefficient,
                    maker_coefficient = EXCLUDED.maker_coefficient,
                    updated_at = CURRENT_TIMESTAMP",
    );

    match query_builder.build().execute(&pool).await {
        Ok(_) => {
            log::info!("Success insert {} tickers", tickers.ticker.len());
            Ok(())
        }
        Err(e) => {
            let msg = format!("Error on bulk insert tickers to db: {}", e);
            log::error!("{}", msg);
            Err(msg)
        }
    }
}
pub async fn insert_symbols_to_db(
    pool: sqlx::PgPool,
    exchange: &str,
    symbols: Vec<Symbol>,
) -> Result<(), String> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO symbol
            (exchange, symbol, symbol_name, base_currency, quote_currency, fee_currency,
            market, base_min_size, quote_min_size, base_max_size, quote_max_size,
            base_increment, quote_increment, price_increment, price_limit_rate,
            min_funds, is_margin_enabled, enable_trading, fee_category,
            maker_fee_coefficient, taker_fee_coefficient, st, updated_at)",
    );

    query_builder.push_values(&symbols, |mut b, symbol| {
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
            .push_bind(chrono::Utc::now());
    });

    query_builder.push(
        " ON CONFLICT (exchange, symbol)
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
                updated_at = CURRENT_TIMESTAMP",
    );

    match query_builder.build().execute(&pool).await {
        Ok(_) => {
            log::info!("Success insert {} symbols", symbols.len());
            Ok(())
        }
        Err(e) => {
            let msg: String = format!("Error on bulk insert symbols to db: {}", e);
            log::error!("{}", msg);
            Err(msg)
        }
    }
}
pub async fn insert_currencies_to_db(
    pool: sqlx::PgPool,
    exchange: &str,
    currencies: Vec<Currencies>,
) -> Result<(), String> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                    "INSERT INTO currency
                    (exchange, currency, currency_name, full_name, is_margin_enabled, is_debit_enabled, updated_at)",
        );

    query_builder.push_values(&currencies, |mut b, currency| {
        b.push_bind(exchange)
            .push_bind(&currency.currency)
            .push_bind(&currency.name)
            .push_bind(&currency.full_name)
            .push_bind(currency.precision)
            .push_bind(currency.is_margin_enabled)
            .push_bind(currency.is_debit_enabled)
            .push_bind(chrono::Utc::now());
    });

    query_builder.push(
        " ON CONFLICT (exchange, currency)
            DO UPDATE SET
                currency_name = EXCLUDED.currency_name,
                full_name = EXCLUDED.full_name,
                precision = EXCLUDED.precision,
                is_margin_enabled = EXCLUDED.is_margin_enabled,
                is_debit_enabled = EXCLUDED.is_debit_enabled,
                updated_at = CURRENT_TIMESTAMP",
    );

    match query_builder.build().execute(&pool).await {
        Ok(_) => {
            log::info!("Success insert {} currencies", currencies.len());
            Ok(())
        }
        Err(e) => {
            let msg: String = format!("Error on bulk insert currencies to db: {}", e);
            log::error!("{}", msg);
            Err(msg)
        }
    }
}
