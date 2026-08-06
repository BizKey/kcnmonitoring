use crate::api::models::{CurrenciesApi, SymbolApi, TickerApi};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SymbolDb {
    pub symbol: String,
    pub name: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub fee_currency: String,
    pub market: String,
    pub base_min_size: String,
    pub quote_min_size: String,
    pub base_max_size: String,
    pub quote_max_size: String,
    pub base_increment: String,
    pub quote_increment: String,
    pub price_increment: String,
    pub price_limit_rate: String,
    pub min_funds: Option<String>,
    pub is_margin_enabled: bool,
    pub enable_trading: bool,
    pub fee_category: i16,
    pub maker_fee_coefficient: String,
    pub taker_fee_coefficient: String,
    pub st: bool,
}

impl From<SymbolApi> for SymbolDb {
    fn from(symbol: SymbolApi) -> SymbolDb {
        Self {
            symbol: symbol.symbol,
            name: symbol.name,
            base_currency: symbol.base_currency,
            quote_currency: symbol.quote_currency,
            fee_currency: symbol.fee_currency,
            market: symbol.market,
            base_min_size: symbol.base_min_size,
            quote_min_size: symbol.quote_min_size,
            base_max_size: symbol.base_max_size,
            quote_max_size: symbol.quote_max_size,
            base_increment: symbol.base_increment,
            quote_increment: symbol.quote_increment,
            price_increment: symbol.price_increment,
            price_limit_rate: symbol.price_limit_rate,
            min_funds: symbol.min_funds,
            is_margin_enabled: symbol.is_margin_enabled,
            enable_trading: symbol.enable_trading,
            fee_category: symbol.fee_category,
            maker_fee_coefficient: symbol.maker_fee_coefficient,
            taker_fee_coefficient: symbol.taker_fee_coefficient,
            st: symbol.st,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CurrenciesDb {
    pub currency: String,
    pub name: String,
    pub full_name: String,
    pub precision: i16,
    pub is_margin_enabled: bool,
    pub is_debit_enabled: bool,
}

impl From<CurrenciesApi> for CurrenciesDb {
    fn from(currencies: CurrenciesApi) -> CurrenciesDb {
        Self {
            currency: currencies.currency,
            name: currencies.name,
            full_name: currencies.full_name,
            precision: currencies.precision,
            is_margin_enabled: currencies.is_margin_enabled,
            is_debit_enabled: currencies.is_debit_enabled,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TickerDb {
    pub symbol: String,
    pub symbol_name: String,
    pub taker_fee_rate: String,
    pub maker_fee_rate: String,
    pub taker_coefficient: String,
    pub maker_coefficient: String,
}

impl From<TickerApi> for TickerDb {
    fn from(ticker: TickerApi) -> TickerDb {
        Self {
            symbol: ticker.symbol,
            symbol_name: ticker.symbol_name,
            taker_fee_rate: ticker.taker_fee_rate,
            maker_fee_rate: ticker.maker_fee_rate,
            taker_coefficient: ticker.taker_coefficient,
            maker_coefficient: ticker.maker_coefficient,
        }
    }
}
