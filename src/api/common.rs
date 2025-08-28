use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Ticker {
    pub symbol: String,

    #[serde(rename = "symbolName")]
    pub symbol_name: String,
    pub buy: Option<String>,

    #[serde(rename = "bestBidSize")]
    pub best_bid_size: Option<String>,
    pub sell: Option<String>,

    #[serde(rename = "bestAskSize")]
    pub best_ask_size: Option<String>,

    #[serde(rename = "changeRate")]
    pub change_rate: Option<String>,

    #[serde(rename = "changePrice")]
    pub change_price: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub vol: Option<String>,

    #[serde(rename = "volValue")]
    pub vol_value: String,
    pub last: Option<String>,

    #[serde(rename = "averagePrice")]
    pub average_price: Option<String>,

    #[serde(rename = "takerFeeRate")]
    pub taker_fee_rate: String,

    #[serde(rename = "makerFeeRate")]
    pub maker_fee_rate: String,

    #[serde(rename = "takerCoefficient")]
    pub taker_coefficient: String,

    #[serde(rename = "makerCoefficient")]
    pub maker_coefficient: String,
}

#[derive(Debug, Deserialize)]
pub struct TickerData {
    pub time: u128,
    pub ticker: Vec<Ticker>,
}
#[derive(Debug, Deserialize)]
pub struct ListTickers {
    pub code: String,
    pub data: TickerData,
}

#[derive(Debug, Deserialize)]
pub struct Currencies {
    pub currency: String,
    pub name: String,

    #[serde(rename = "fullName")]
    pub full_name: String,
    pub precision: u8,
    pub confirms: Option<u8>,

    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,

    #[serde(rename = "isMarginEnabled")]
    pub is_margin_enabled: bool,

    #[serde(rename = "isDebitEnabled")]
    pub is_debit_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListCurrencies {
    pub code: String,
    pub data: Vec<Currencies>,
}

#[derive(Debug, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub name: String,

    #[serde(rename = "baseCurrency")]
    pub base_currency: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSymbols {
    pub code: String,
    pub data: Vec<Symbol>,
}
