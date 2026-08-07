use crate::infrastructure::config::Config;
use anyhow::{Context, Result};
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, Method, Response};
use sha2::Sha256;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, serde::Deserialize)]
pub struct ApiV1MarketAllTickers {
    pub code: String,
    pub msg: Option<String>,
    pub data: Option<TickerData>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TickerData {
    pub ticker: Vec<TickerApi>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TickerApi {
    pub symbol: String,
    #[serde(rename = "symbolName")]
    pub symbol_name: String,
    #[serde(rename = "takerFeeRate")]
    pub taker_fee_rate: String,
    #[serde(rename = "makerFeeRate")]
    pub maker_fee_rate: String,
    #[serde(rename = "takerCoefficient")]
    pub taker_coefficient: String,
    #[serde(rename = "makerCoefficient")]
    pub maker_coefficient: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiV3Currencies {
    pub code: String,
    pub msg: Option<String>,
    pub data: Option<Vec<CurrenciesApi>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CurrenciesApi {
    pub currency: String,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub precision: i16,
    #[serde(rename = "isMarginEnabled")]
    pub is_margin_enabled: bool,
    #[serde(rename = "isDebitEnabled")]
    pub is_debit_enabled: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiV2Symbols {
    pub code: String,
    pub msg: Option<String>,
    pub data: Option<Vec<SymbolApi>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SymbolApi {
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
}

#[derive(Debug, Clone)]
pub struct KuCoinClient {
    client: Client,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    base_url: String,
}

static KUCLIENT: OnceLock<Result<KuCoinClient>> = OnceLock::new();

pub fn init_client(config: &Config) -> Result<()> {
    KUCLIENT.get_or_init(|| KuCoinClient::new(config));
    Ok(())
}

pub fn get_client() -> Result<&'static KuCoinClient> {
    KUCLIENT
        .get()
        .ok_or_else(|| anyhow::anyhow!("KuCoin client not initialized"))?
        .as_ref()
        .map_err(|e| anyhow::anyhow!("KuCoin client error: {}", e))
}

impl KuCoinClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_key: config.kucoin_key.clone(),
            api_secret: config.kucoin_secret.clone(),
            api_passphrase: config.kucoin_passphrase.clone(),
            base_url: config.kucoin_base_url.clone(),
        })
    }

    fn get_system_timestamp_ms(&self) -> Result<u64> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get UNIX_EPOCH")?
            .as_millis() as u64)
    }

    fn generate_signature(&self, to_sign: &[u8]) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .context("Failed to create HMAC-SHA256")?;
        mac.update(to_sign);
        Ok(base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
    }

    async fn make_request(
        &self,
        method: Method,
        endpoint: &str,
        query_string: &str,
        body_str: &str,
        authenticated: bool,
    ) -> Result<Response> {
        let timestamp = self.get_system_timestamp_ms()?;
        let url = if !query_string.is_empty() {
            format!("{}{}?{}", self.base_url, endpoint, query_string)
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let mut request_builder = self.client.request(method.clone(), &url);

        if authenticated {
            let mut str_to_sign = format!(
                "{}{}{}",
                timestamp,
                method.as_ref().to_uppercase(),
                endpoint
            );

            if !query_string.is_empty() {
                str_to_sign.push('?');
                str_to_sign.push_str(&query_string);
            }
            if !body_str.is_empty() {
                str_to_sign.push_str(body_str);
            }

            let kc_api_sign = self
                .generate_signature(str_to_sign.as_bytes())
                .context("Failed to generate signature")?;

            let kc_api_passphrase = self.generate_signature(self.api_passphrase.as_bytes())?;

            request_builder = request_builder
                .header("KC-API-KEY", &self.api_key)
                .header("KC-API-SIGN", kc_api_sign)
                .header("KC-API-TIMESTAMP", timestamp.to_string())
                .header("KC-API-PASSPHRASE", kc_api_passphrase)
                .header("KC-API-KEY-VERSION", "2");

            if !body_str.is_empty() {
                request_builder = request_builder
                    .header("Content-Type", "application/json")
                    .body(body_str.to_string());
            }
        }

        match request_builder.send().await {
            Ok(response) => Ok(response),
            Err(e) => {
                if e.is_timeout() {
                    anyhow::bail!("Timeout {}: {}", url, e)
                } else if e.is_connect() {
                    anyhow::bail!("Error connection {}: {}", url, e)
                } else if e.is_request() {
                    anyhow::bail!("Error prepare request {}: {}", url, e)
                } else if e.is_body() {
                    anyhow::bail!("Error in body {}: {}", url, e)
                } else {
                    anyhow::bail!("Unexpected error {}: {}", url, e)
                }
            }
        }
    }

    async fn read_response(&self, response: Response) -> Result<String> {
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        match status {
            200 => Ok(body),
            _ => anyhow::bail!("API returned error status {}: {}", status, body),
        }
    }

    pub async fn api_v3_currencies_get(&self) -> Result<Option<Vec<CurrenciesApi>>> {
        let response = self
            .make_request(Method::GET, "/api/v3/currencies", "", "", false)
            .await?;

        let body = self.read_response(response).await?;
        let response_data = serde_json::from_str::<ApiV3Currencies>(&body)
            .context("Failed to deserialize currencies response")?;

        if response_data.code == "200000" {
            Ok(response_data.data)
        } else {
            anyhow::bail!(
                "KuCoin API error: code={}, msg={:?}",
                response_data.code,
                response_data.msg
            )
        }
    }

    pub async fn api_v1_market_all_tickers_get(&self) -> Result<Option<TickerData>> {
        let response = self
            .make_request(Method::GET, "/api/v1/market/allTickers", "", "", false)
            .await?;

        let body = self.read_response(response).await?;
        let response_data = serde_json::from_str::<ApiV1MarketAllTickers>(&body)
            .context("Failed to deserialize tickers response")?;

        if response_data.code == "200000" {
            Ok(response_data.data)
        } else {
            anyhow::bail!(
                "KuCoin API error: code={}, msg={:?}",
                response_data.code,
                response_data.msg
            )
        }
    }

    pub async fn api_v2_symbols_get(&self) -> Result<Option<Vec<SymbolApi>>> {
        let response = self
            .make_request(Method::GET, "/api/v2/symbols", "", "", false)
            .await?;

        let body = self.read_response(response).await?;
        let response_data = serde_json::from_str::<ApiV2Symbols>(&body)
            .context("Failed to deserialize symbols response")?;

        if response_data.code == "200000" {
            Ok(response_data.data)
        } else {
            anyhow::bail!(
                "KuCoin API error: code={}, msg={:?}",
                response_data.code,
                response_data.msg
            )
        }
    }
}
