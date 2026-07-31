use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub symbol_name: String,
    pub taker_fee_rate: String,
    pub maker_fee_rate: String,
    pub taker_coefficient: String,
    pub maker_coefficient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub currency: String,
    pub name: String,
    pub full_name: String,
    pub precision: i16,
    pub is_margin_enabled: bool,
    pub is_debit_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SyncStats {
    pub tickers_processed: usize,
    pub symbols_processed: usize,
    pub currencies_processed: usize,
    pub timestamp: DateTime<Utc>,
}

impl SyncStats {
    pub fn new() -> Self {
        Self {
            tickers_processed: 0,
            symbols_processed: 0,
            currencies_processed: 0,
            timestamp: Utc::now(),
        }
    }
}
