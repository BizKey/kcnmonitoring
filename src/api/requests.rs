use crate::api::models::{
    Currencies, ListCurrencies, ListSymbols, ListTickers, Symbol, TickerData,
};
use base64::Engine;
use hmac::{Hmac, Mac};
use urlencoding::encode;

use reqwest::{Client, Response};

use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub fn new(base_url: String) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let api_passphrase = match env::var("KUCOIN_PASS") {
            Ok(api_passphrase) => api_passphrase,
            Err(e) => return Err(e.into()),
        };

        let api_key = match env::var("KUCOIN_KEY") {
            Ok(api_key) => api_key,
            Err(e) => return Err(e.into()),
        };

        let api_secret = match env::var("KUCOIN_SECRET") {
            Ok(api_secret) => api_secret,
            Err(e) => return Err(e.into()),
        };

        Ok(Self {
            client: Client::new(),
            api_key,
            api_secret,
            api_passphrase,
            base_url,
        })
    }

    fn get_system_timestamp_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub async fn api_v3_currencies(
        &self,
    ) -> Result<Vec<Currencies>, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp: u64 = self.get_system_timestamp_ms();
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v3/currencies",
                None,
                None,
                false,
                timestamp,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ListCurrencies>(&text) {
                        Ok(r) => match r.code.as_str() {
                            "200000" => Ok(r.data),
                            _ => Err(format!("API error: code {}", r.code).into()),
                        },
                        Err(e) => Err(format!(
                            "Error JSON deserialize:'{}' with data: '{}'",
                            e, text
                        )
                        .into()),
                    },
                    Err(e) => {
                        return Err(format!("Error get text response from HTTP:'{}'", e).into());
                    }
                },
                status => match response.text().await {
                    Ok(text) => {
                        Err(format!("Wrong HTTP status: '{}' with body: '{}'", status, text).into())
                    }
                    Err(_) => Err(format!("Wrong HTTP status: '{}'", status).into()),
                },
            },
            Err(e) => return Err(format!("Error HTTP:'{}'", e).into()),
        };
    }
    pub async fn api_v1_market_alltickers(
        &self,
    ) -> Result<TickerData, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp: u64 = self.get_system_timestamp_ms();
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v1/market/allTickers",
                None,
                None,
                false,
                timestamp,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ListTickers>(&text) {
                        Ok(r) => match r.code.as_str() {
                            "200000" => Ok(r.data),
                            _ => Err(format!("API error: code {}", r.code).into()),
                        },
                        Err(e) => Err(format!(
                            "Error JSON deserialize:'{}' with data: '{}'",
                            e, text
                        )
                        .into()),
                    },
                    Err(e) => {
                        return Err(format!("Error get text response from HTTP:'{}'", e).into());
                    }
                },
                status => match response.text().await {
                    Ok(text) => {
                        return Err(format!(
                            "Wrong HTTP status: '{}' with body: '{}'",
                            status, text
                        )
                        .into());
                    }
                    Err(_) => Err(format!("Wrong HTTP status: '{}'", status).into()),
                },
            },
            Err(e) => return Err(format!("Error HTTP:'{}'", e).into()),
        };
    }

    pub async fn api_v2_symbols(
        &self,
    ) -> Result<Vec<Symbol>, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp: u64 = self.get_system_timestamp_ms();
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v2/symbols",
                None,
                None,
                false,
                timestamp,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ListSymbols>(&text) {
                        Ok(r) => match r.code.as_str() {
                            "200000" => Ok(r.data),
                            _ => Err(format!("API error: code {}", r.code).into()),
                        },
                        Err(e) => Err(format!(
                            "Error JSON deserialize:'{}' with data: '{}'",
                            e, text
                        )
                        .into()),
                    },
                    Err(e) => {
                        return Err(format!("Error get text response from HTTP:'{}'", e).into());
                    }
                },
                status => match response.text().await {
                    Ok(text) => {
                        return Err(format!(
                            "Wrong HTTP status: '{}' with body: '{}'",
                            status, text
                        )
                        .into());
                    }
                    Err(_) => Err(format!("Wrong HTTP status: '{}'", status).into()),
                },
            },
            Err(e) => return Err(format!("Error HTTP:'{}'", e).into()),
        };
    }

    fn generate_signature(
        &self,
        timestamp: u64,
        method: &str,
        endpoint: &str,
        query_string: &str,
        body: &str,
    ) -> String {
        let string_to_sign: String = format!("{}{}{}{}", timestamp, method, endpoint, query_string);
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }

    fn generate_passphrase_signature(&self) -> String {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(self.api_passphrase.as_bytes());
        let result = mac.finalize();
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }
    async fn make_request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        query_params: Option<HashMap<&str, &str>>,
        body: Option<HashMap<&str, &str>>,
        authenticated: bool,
        timestamp: u64,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let query_string: String = query_params
            .as_ref()
            .map(|params| {
                let mut pairs: Vec<_> = params.iter().collect();
                pairs.sort_by(|a, b| a.0.cmp(b.0));
                pairs
                    .iter()
                    .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
                    .collect::<Vec<_>>()
                    .join("&")
            })
            .unwrap_or_default();

        let url = if !query_string.is_empty() {
            format!("{}{}?{}", self.base_url, endpoint, query_string)
        } else {
            format!("{}{}", self.base_url, endpoint)
        };

        let mut request_builder = self.client.request(method.clone(), &url);

        if authenticated {
            let body_str = body
                .as_ref()
                .map(|b| {
                    serde_json::to_string(b).map_err(|e| format!("JSON serialization error: {}", e))
                })
                .transpose()?
                .unwrap_or_default();

            let signature = self.generate_signature(
                timestamp,
                method.as_ref(),
                endpoint,
                &query_string,
                &body_str,
            );

            let passphrase_signature = self.generate_passphrase_signature();

            request_builder = request_builder
                .header("KC-API-KEY", &self.api_key)
                .header("KC-API-SIGN", signature)
                .header("KC-API-TIMESTAMP", timestamp.to_string())
                .header("KC-API-PASSPHRASE", passphrase_signature)
                .header("KC-API-KEY-VERSION", "2");

            if !body_str.is_empty() {
                request_builder = request_builder
                    .header("Content-Type", "application/json")
                    .body(body_str);
            }
        }

        match request_builder.send().await {
            Ok(response) => Ok(response),
            Err(e) => {
                {
                    if e.is_timeout() {
                        let msg: String = format!("Timeout {}: {}", url, e);
                        log::error!("{}", msg);
                    } else if e.is_connect() {
                        let msg: String = format!("Error connection {}: {}", url, e);
                        log::error!("{}", msg);
                    } else if e.is_request() {
                        let msg: String = format!("Error prepare request {}: {}", url, e);
                        log::error!("{}", msg);
                    } else if e.is_body() {
                        let msg: String = format!("Error in body {}: {}", url, e);
                        log::error!("{}", msg);
                    } else {
                        let msg: String = format!("Unexpected error {}: {}", url, e);
                        log::error!("{}", msg);
                    }
                }
                Err(e.into())
            }
        }
    }
}
