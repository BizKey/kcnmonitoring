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

#[derive(Debug, Deserialize)]
pub struct LoanMarket {
    pub currency: String,

    #[serde(rename = "purchaseEnable")]
    pub purchase_enable: bool,

    #[serde(rename = "redeemEnable")]
    pub redeem_enable: bool,

    pub increment: String,

    #[serde(rename = "minPurchaseSize")]
    pub min_purchase_size: String,

    #[serde(rename = "maxPurchaseSize")]
    pub max_purchase_size: String,

    #[serde(rename = "interestIncrement")]
    pub interest_increment: String,

    #[serde(rename = "minInterestRate")]
    pub min_interest_rate: String,

    #[serde(rename = "marketInterestRate")]
    pub market_interest_rate: String,

    #[serde(rename = "maxInterestRate")]
    pub max_interest_rate: String,

    #[serde(rename = "autoPurchaseEnable")]
    pub auto_purchase_enable: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListLoanMarket {
    pub code: String,
    pub data: Vec<LoanMarket>,
}

#[derive(Debug, Deserialize)]
pub struct ApiV1Timestamp {
    pub code: String,
    pub data: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApiV3MarginBorrowRateDataItem {
    pub currency: String,

    #[serde(rename = "hourlyBorrowRate")]
    pub hourly_borrow_rate: String,

    #[serde(rename = "annualizedBorrowRate")]
    pub annualized_borrow_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiV3MarginBorrowRateData {
    #[serde(rename = "vipLevel")]
    pub vip_level: u8,

    pub items: Vec<ApiV3MarginBorrowRateDataItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApiV3MarginBorrowRate {
    pub code: String,
    pub data: ApiV3MarginBorrowRateData,
}
