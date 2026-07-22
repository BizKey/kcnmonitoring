use crate::api::models::{
    ApiV1MarketAllTickers, ApiV2Symbols, ApiV3Currencies, Currencies, Symbol, TickerData,
};
use crate::api::tools::get_env;
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, Method, Response};
use sha2::Sha256;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct KuCoinClient {
    client: Client,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    base_url: String,
}

impl KuCoinClient {
    pub fn new() -> Result<Self, String> {
        let base_url: String = get_env("KUCOIN_BASE_URL")?;
        let api_key: String = get_env("KUCOIN_KEY")?;
        let api_secret: String = get_env("KUCOIN_SECRET")?;
        let api_passphrase: String = get_env("KUCOIN_PASS")?;

        Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .map(|client| Self {
                client,
                api_key,
                api_secret,
                api_passphrase,
                base_url,
            })
            .map_err(|e| {
                let msg: String = format!("Get error on Client::builder:{}", e);
                error!("{}", msg);
                msg
            })
    }

    fn get_system_timestamp_ms(&self) -> Result<u64, String> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                let msg: String = format!("Get error get UNIX_EPOCH:{e}");
                error!("{}", msg);
                msg
            })?
            .as_millis() as u64)
    }

    async fn api_v3_currencies_get(&self) -> Result<String, String> {
        let system_timestamp_ms: u64 = self.get_system_timestamp_ms().map_err(|e| {
            error!("{}", e);
            e
        })?;

        let response: Response = self
            .make_request(
                Method::GET,
                "/api/v3/currencies",
                String::new(),
                String::new(),
                false,
                system_timestamp_ms,
            )
            .await?;

        let status: reqwest::StatusCode = response.status();

        let response_string: String = response.text().await.map_err(|e| {
            let msg: String = format!("Fail read text from response:{e}");
            error!("{}", msg);
            msg
        })?;

        match status.as_u16() {
            200 => Ok(response_string),
            status_code => {
                let msg: String = format!(
                    "API returned error status {}: {}",
                    status_code, response_string
                );
                error!("{}", msg);
                Err(msg)
            }
        }
    }
    async fn api_v1_market_all_tickers_get(&self) -> Result<String, String> {
        let system_timestamp_ms: u64 = self.get_system_timestamp_ms().map_err(|e| {
            error!("{}", e);
            e
        })?;

        let response: Response = self
            .make_request(
                Method::GET,
                "/api/v1/market/allTickers",
                String::new(),
                String::new(),
                false,
                system_timestamp_ms,
            )
            .await?;

        let status: reqwest::StatusCode = response.status();

        let response_string: String = response.text().await.map_err(|e| {
            let msg: String = format!("Fail read text from response:{e}");
            error!("{}", msg);
            msg
        })?;

        match status.as_u16() {
            200 => Ok(response_string),
            status_code => {
                let msg: String = format!(
                    "API returned error status {}: {}",
                    status_code, response_string
                );
                error!("{}", msg);
                Err(msg)
            }
        }
    }
    async fn api_v2_symbols_get(&self) -> Result<String, String> {
        let system_timestamp_ms: u64 = self.get_system_timestamp_ms().map_err(|e| {
            error!("{}", e);
            e
        })?;

        let response: Response = self
            .make_request(
                Method::GET,
                "/api/v2/symbols",
                String::new(),
                String::new(),
                false,
                system_timestamp_ms,
            )
            .await?;

        let status: reqwest::StatusCode = response.status();

        let response_string: String = response.text().await.map_err(|e| {
            let msg: String = format!("Fail read text from response:{e}");
            error!("{}", msg);
            msg
        })?;

        match status.as_u16() {
            200 => Ok(response_string),
            status_code => {
                let msg: String = format!(
                    "API returned error status {}: {}",
                    status_code, response_string
                );
                error!("{}", msg);
                Err(msg)
            }
        }
    }

    fn generate_signature(&self, to_sign: &[u8]) -> Result<String, String> {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes()).map_err(|e| {
            let msg = format!("Fail get api secret:{e}");
            error!("{}", msg);
            msg
        })?;

        mac.update(to_sign);
        Ok(base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
    }

    async fn make_request(
        &self,
        method: Method,
        endpoint: &str,
        query_string: String,
        body_str: String,
        authenticated: bool,
        timestamp: u64,
    ) -> Result<Response, String> {
        let url: String = if !query_string.is_empty() {
            format!("{}{}?{}", self.base_url, endpoint, query_string)
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let mut request_builder: reqwest::RequestBuilder =
            self.client.request(method.clone(), &url);

        if authenticated {
            let mut str_to_sign: String = format!(
                "{}{}{}",
                timestamp,
                method.as_ref().to_uppercase(),
                endpoint
            );

            if !&query_string.is_empty() {
                str_to_sign.push('?');
                str_to_sign.push_str(&query_string);
            }
            if !&body_str.is_empty() {
                str_to_sign.push_str(&body_str);
            }

            let kc_api_sign: String = self.generate_signature(str_to_sign.as_bytes())?;

            let kc_api_passphrase: String =
                self.generate_signature(self.api_passphrase.as_bytes())?;

            request_builder = request_builder
                .header("KC-API-KEY", &self.api_key)
                .header("KC-API-SIGN", kc_api_sign)
                .header("KC-API-TIMESTAMP", timestamp.to_string())
                .header("KC-API-PASSPHRASE", kc_api_passphrase)
                .header("KC-API-KEY-VERSION", "2");

            if !body_str.is_empty() {
                request_builder = request_builder
                    .header("Content-Type", "application/json")
                    .body(body_str);
            }
        }
        Ok(request_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                let msg: String = format!("Timeout {url}: {e}");
                error!("{}", msg);
                msg
            } else if e.is_connect() {
                let msg: String = format!("Error connection {url}: {e}");
                error!("{}", msg);
                msg
            } else if e.is_request() {
                let msg: String = format!("Error prepare request {url}: {e}");
                error!("{}", msg);
                msg
            } else if e.is_body() {
                let msg: String = format!("Error in body {url}: {e}");
                error!("{}", msg);
                msg
            } else {
                let msg: String = format!("Unexpected error {url}: {e}");
                error!("{}", msg);
                msg
            }
        })?)
    }
}

static KUCLIENT: OnceLock<Result<KuCoinClient, String>> = OnceLock::new();

fn get_client() -> Result<&'static KuCoinClient, String> {
    Ok(KUCLIENT
        .get_or_init(|| KuCoinClient::new())
        .as_ref()
        .map_err(|e| {
            let msg: String = format!("Fail get or init KuCoinClient:{e}");
            error!("{}", msg);
            msg
        })?)
}

pub async fn api_v1_market_all_tickers_get() -> Result<Option<TickerData>, String> {
    let client: &KuCoinClient = get_client()?;

    let response_string: String = client.api_v1_market_all_tickers_get().await?;

    let response: ApiV1MarketAllTickers =
        serde_json::from_str::<ApiV1MarketAllTickers>(&response_string).map_err(|e| {
            let msg: String = format!(
                "Failed to deserialize response '{}' as {}: {}",
                response_string,
                stringify!(ApiV1MarketAllTickers),
                e
            );
            error!("{}", msg);
            msg
        })?;

    match response.code.as_str() {
        "200000" => Ok(response.data),
        _ => {
            let msg: String = format!(
                "KuCoin API error: code={}, msg={:?}, data={:?}",
                response.code, response.msg, response.data
            );
            error!("{}", msg);
            Err(msg)
        }
    }
}

pub async fn api_v2_symbols_get() -> Result<Option<Vec<Symbol>>, String> {
    let client: &KuCoinClient = get_client()?;

    let response_string: String = client.api_v2_symbols_get().await?;

    let response: ApiV2Symbols =
        serde_json::from_str::<ApiV2Symbols>(&response_string).map_err(|e| {
            let msg: String = format!(
                "Failed to deserialize response '{response_string}' as (ApiV2Symbols): {e}"
            );
            error!("{}", msg);
            msg
        })?;

    match response.code.as_str() {
        "200000" => Ok(response.data),
        _ => {
            let msg: String = format!(
                "KuCoin API error: code={}, msg={:?}, data={:?}",
                response.code, response.msg, response.data
            );
            error!("{}", msg);
            Err(msg)
        }
    }
}

pub async fn api_v3_currencies_get() -> Result<Option<Vec<Currencies>>, String> {
    let client: &KuCoinClient = get_client()?;

    let response_string: String = client.api_v3_currencies_get().await?;

    let response: ApiV3Currencies = serde_json::from_str::<ApiV3Currencies>(&response_string)
        .map_err(|e| {
            let msg: String = format!(
                "Failed to deserialize response '{}' as {}: {}",
                response_string,
                stringify!(ApiV3Currencies),
                e
            );
            error!("{}", msg);
            msg
        })?;

    match response.code.as_str() {
        "200000" => Ok(response.data),
        _ => {
            let msg: String = format!(
                "KuCoin API error: code={}, msg={:?}, data={:?}",
                response.code, response.msg, response.data
            );
            error!("{}", msg);
            Err(msg)
        }
    }
}
