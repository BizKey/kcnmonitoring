use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub symbol_name: String,
    pub taker_fee_rate: String,
    pub maker_fee_rate: String,
    pub taker_coefficient: String,
    pub maker_coefficient: String,
}

impl Ticker {
    pub fn new(
        symbol: String,
        symbol_name: String,
        taker_fee_rate: String,
        maker_fee_rate: String,
        taker_coefficient: String,
        maker_coefficient: String,
    ) -> Self {
        Self {
            symbol,
            symbol_name,
            taker_fee_rate,
            maker_fee_rate,
            taker_coefficient,
            maker_coefficient,
        }
    }
}
