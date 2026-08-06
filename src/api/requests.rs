use crate::api::models::{
    ApiV1MarketAllTickers, ApiV2Symbols, ApiV3Currencies, CurrenciesApi, SymbolApi, TickerData,
};
use crate::tools::get_env;
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, Method, Response};
use sha2::Sha256;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
type HmacSha256 = Hmac<Sha256>;
use anyhow::{Context, Result};
#[derive(Debug, Clone)]
pub struct KuCoinClient {
    client: Client,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    base_url: String,
}

impl KuCoinClient {
    pub fn new() -> Result<Self> {
        let base_url = get_env("KUCOIN_BASE_URL")?;
        let api_key = get_env("KUCOIN_KEY")?;
        let api_secret = get_env("KUCOIN_SECRET")?;
        let api_passphrase = get_env("KUCOIN_PASS")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .context("Get error on Client::builder")?;
        Ok(Self {
            client,
            api_key,
            api_secret,
            api_passphrase,
            base_url,
        })
    }

    fn get_system_timestamp_ms(&self) -> Result<u64> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Get error get UNIX_EPOCH")?
            .as_millis() as u64)
    }

    async fn api_v3_currencies_get(&self) -> Result<String> {
        Ok(read_response(
            self.make_request(
                Method::GET,
                "/api/v3/currencies",
                &String::new(),
                &String::new(),
                false,
            )
            .await?,
        )
        .await?)
    }
    async fn api_v1_market_all_tickers_get(&self) -> Result<String> {
        Ok(read_response(
            self.make_request(
                Method::GET,
                "/api/v1/market/allTickers",
                &String::new(),
                &String::new(),
                false,
            )
            .await?,
        )
        .await?)
    }
    async fn api_v2_symbols_get(&self) -> Result<String> {
        Ok(read_response(
            self.make_request(
                Method::GET,
                "/api/v2/symbols",
                &String::new(),
                &String::new(),
                false,
            )
            .await?,
        )
        .await?)
    }

    fn generate_signature(&self, to_sign: &[u8]) -> Result<String> {
        let mut mac =
            HmacSha256::new_from_slice(self.api_secret.as_bytes()).context("Fail HmacSha256")?;
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
            if !&body_str.is_empty() {
                str_to_sign.push_str(body_str);
            }

            let kc_api_sign = self
                .generate_signature(str_to_sign.as_bytes())
                .context("Fail generate signature")?;

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
}

static KUCLIENT: OnceLock<Result<KuCoinClient>> = OnceLock::new();

fn get_client() -> Result<&'static KuCoinClient> {
    KUCLIENT
        .get_or_init(|| KuCoinClient::new())
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Fail get or init KuCoinClient: {e}"))
}

async fn read_response(response: Response) -> Result<String> {
    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .context("Failed to read response body from")?;

    match status {
        200 => Ok(body),
        _ => anyhow::bail!("API returned error status {}: {}", status, body),
    }
}

pub async fn api_v1_market_all_tickers_get() -> Result<Option<TickerData>> {
    let response_string = get_client()?.api_v1_market_all_tickers_get().await?;

    let response =
        serde_json::from_str::<ApiV1MarketAllTickers>(&response_string).with_context(|| {
            format!(
                "Failed to deserialize response '{}' as {}",
                response_string,
                stringify!(ApiV1MarketAllTickers),
            )
        })?;

    if response.code.as_str() == "200000" {
        Ok(response.data)
    } else {
        anyhow::bail!(
            "KuCoin API error: code={}, msg={:?}, data={:?}",
            response.code,
            response.msg,
            response.data
        )
    }
}

pub async fn api_v2_symbols_get() -> Result<Option<Vec<SymbolApi>>> {
    let response_string = get_client()?.api_v2_symbols_get().await?;

    let response = serde_json::from_str::<ApiV2Symbols>(&response_string).with_context(|| {
        format!("Failed to deserialize response '{response_string}' as (ApiV2Symbols)")
    })?;

    if response.code.as_str() == "200000" {
        Ok(response.data)
    } else {
        anyhow::bail!(
            "KuCoin API error: code={}, msg={:?}, data={:?}",
            response.code,
            response.msg,
            response.data
        )
    }
}

pub async fn api_v3_currencies_get() -> Result<Option<Vec<CurrenciesApi>>> {
    let response_string = get_client()?.api_v3_currencies_get().await?;

    let response =
        serde_json::from_str::<ApiV3Currencies>(&response_string).with_context(|| {
            format!(
                "Failed to deserialize response '{}' as {}",
                response_string,
                stringify!(ApiV3Currencies),
            )
        })?;

    if response.code.as_str() == "200000" {
        Ok(response.data)
    } else {
        anyhow::bail!(
            "KuCoin API error: code={}, msg={:?}, data={:?}",
            response.code,
            response.msg,
            response.data
        )
    }
}
