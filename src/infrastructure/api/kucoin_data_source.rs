use crate::api::requests as kucoin_api;
use crate::domain::{Currency, DomainResult, ExchangeDataSource, Symbol, Ticker};
use async_trait::async_trait;

pub struct KuCoinDataSource {
    exchange_name: String,
}

impl KuCoinDataSource {
    pub fn new() -> Self {
        Self {
            exchange_name: "kucoin".to_string(),
        }
    }
}

#[async_trait]
impl ExchangeDataSource for KuCoinDataSource {
    async fn get_tickers(&self) -> DomainResult<Vec<Ticker>> {
        let ticker_data = kucoin_api::api_v1_market_all_tickers_get()
            .await
            .map_err(|e| crate::domain::DomainError::Api(e.to_string()))?;

        let tickers = ticker_data
            .ok_or_else(|| crate::domain::DomainError::NotFound("No tickers data".to_string()))?
            .ticker
            .into_iter()
            .map(|t| Ticker {
                symbol: t.symbol,
                symbol_name: t.symbol_name,
                taker_fee_rate: t.taker_fee_rate,
                maker_fee_rate: t.maker_fee_rate,
                taker_coefficient: t.taker_coefficient,
                maker_coefficient: t.maker_coefficient,
            })
            .collect();

        Ok(tickers)
    }

    async fn get_symbols(&self) -> DomainResult<Vec<Symbol>> {
        let symbols = kucoin_api::api_v2_symbols_get()
            .await
            .map_err(|e| crate::domain::DomainError::Api(e.to_string()))?
            .ok_or_else(|| crate::domain::DomainError::NotFound("No symbols data".to_string()))?;

        Ok(symbols
            .into_iter()
            .map(|s| Symbol {
                symbol: s.symbol,
                name: s.name,
                base_currency: s.base_currency,
                quote_currency: s.quote_currency,
                fee_currency: s.fee_currency,
                market: s.market,
                base_min_size: s.base_min_size,
                quote_min_size: s.quote_min_size,
                base_max_size: s.base_max_size,
                quote_max_size: s.quote_max_size,
                base_increment: s.base_increment,
                quote_increment: s.quote_increment,
                price_increment: s.price_increment,
                price_limit_rate: s.price_limit_rate,
                min_funds: s.min_funds,
                is_margin_enabled: s.is_margin_enabled,
                enable_trading: s.enable_trading,
                fee_category: s.fee_category,
                maker_fee_coefficient: s.maker_fee_coefficient,
                taker_fee_coefficient: s.taker_fee_coefficient,
                st: s.st,
            })
            .collect())
    }

    async fn get_currencies(&self) -> DomainResult<Vec<Currency>> {
        let currencies = kucoin_api::api_v3_currencies_get()
            .await
            .map_err(|e| crate::domain::DomainError::Api(e.to_string()))?
            .ok_or_else(|| {
                crate::domain::DomainError::NotFound("No currencies data".to_string())
            })?;

        Ok(currencies
            .into_iter()
            .map(|c| Currency {
                currency: c.currency,
                name: c.name,
                full_name: c.full_name,
                precision: c.precision,
                is_margin_enabled: c.is_margin_enabled,
                is_debit_enabled: c.is_debit_enabled,
            })
            .collect())
    }

    async fn get_exchange_name(&self) -> &str {
        &self.exchange_name
    }
}
