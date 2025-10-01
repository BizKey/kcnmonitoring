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
    pub precision: i16,
    pub confirms: Option<i16>,

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

    #[serde(rename = "quoteCurrency")]
    pub quote_currency: String,

    #[serde(rename = "feeCurrency")]
    pub fee_currency: String,

    pub market: String,

    #[serde(rename = "baseMinSize")]
    pub base_min_size: String,

    #[serde(rename = "quoteMinSize")]
    pub quote_min_size: String,

    #[serde(rename = "baseMaxSize")]
    pub base_max_size: String,

    #[serde(rename = "quoteMaxSize")]
    pub quote_max_size: String,

    #[serde(rename = "baseIncrement")]
    pub base_increment: String,

    #[serde(rename = "quoteIncrement")]
    pub quote_increment: String,

    #[serde(rename = "priceIncrement")]
    pub price_increment: String,

    #[serde(rename = "priceLimitRate")]
    pub price_limit_rate: String,

    #[serde(rename = "minFunds")]
    pub min_funds: Option<String>,

    #[serde(rename = "isMarginEnabled")]
    pub is_margin_enabled: bool,

    #[serde(rename = "enableTrading")]
    pub enable_trading: bool,

    #[serde(rename = "feeCategory")]
    pub fee_category: i16,

    #[serde(rename = "makerFeeCoefficient")]
    pub maker_fee_coefficient: String,

    #[serde(rename = "takerFeeCoefficient")]
    pub taker_fee_coefficient: String,

    pub st: bool,

    #[serde(rename = "callauctionIsEnabled")]
    pub callauction_is_enabled: bool,

    #[serde(rename = "callauctionPriceFloor")]
    pub callauction_price_floor: Option<String>,

    #[serde(rename = "callauctionPriceCeiling")]
    pub callauction_price_ceiling: Option<String>,

    #[serde(rename = "callauctionFirstStageStartTime")]
    pub callauction_first_stage_start_time: Option<i64>,

    #[serde(rename = "callauctionSecondStageStartTime")]
    pub callauction_second_stage_start_time: Option<i64>,

    #[serde(rename = "callauctionThirdStageStartTime")]
    pub callauction_third_stage_start_time: Option<i64>,

    #[serde(rename = "tradingStartTime")]
    pub trading_start_time: Option<i64>,
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
pub struct ApiV3MarginBorrowRateDataItem {
    pub currency: String,

    #[serde(rename = "hourlyBorrowRate")]
    pub hourly_borrow_rate: String,

    #[serde(rename = "annualizedBorrowRate")]
    pub annualized_borrow_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiV3MarginBorrowRateData {
    pub items: Vec<ApiV3MarginBorrowRateDataItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApiV3MarginBorrowRate {
    pub code: String,
    pub data: ApiV3MarginBorrowRateData,
}

#[derive(Debug, Deserialize)]
pub struct Candle {
    pub timestamp: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub quote_volume: String,
}

#[derive(Debug, Deserialize)]
pub struct ListCandle {
    pub code: String,
    pub data: Vec<Vec<String>>,
}
impl ListCandle {
    pub fn into_candles(self) -> Result<Vec<Candle>, &'static str> {
        self.data
            .into_iter()
            .map(|inner| {
                if inner.len() != 7 {
                    return Err("Invalid candle data length");
                }
                Ok(Candle {
                    timestamp: inner[0].clone(),
                    open: inner[1].clone(),
                    high: inner[2].clone(),
                    low: inner[3].clone(),
                    close: inner[4].clone(),
                    volume: inner[5].clone(),
                    quote_volume: inner[6].clone(),
                })
            })
            .collect()
    }
}
