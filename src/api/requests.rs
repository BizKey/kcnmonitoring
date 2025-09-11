use crate::api::models::{
    ApiV1Timestamp, ApiV3MarginBorrowRate, ApiV3MarginBorrowRateData, Currencies, ListCurrencies,
    ListLoanMarket, ListSymbols, ListTickers, LoanMarket, Symbol, Ticker,
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use log::error;
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
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
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        };

        let api_key = match env::var("KUCOIN_KEY") {
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        };

        let api_secret = match env::var("KUCOIN_SECRET") {
            Ok(val) => val,
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

    pub async fn api_v3_margin_borrowrate(
        &self,
    ) -> Result<ApiV3MarginBorrowRateData, Box<dyn std::error::Error + Send + Sync>> {
        // Query the borrowing interest rate through this interface.
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v3/margin/borrowRate",
                None,
                None,
                true,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ApiV3MarginBorrowRate>(&text) {
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

    pub async fn api_v3_project_list(
        &self,
    ) -> Result<Vec<LoanMarket>, Box<dyn std::error::Error + Send + Sync>> {
        // This API endpoint is used to get the information about the currencies available for lending.
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v3/project/list",
                None,
                None,
                true,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ListLoanMarket>(&text) {
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
    pub async fn api_v3_currencies(
        &self,
    ) -> Result<Vec<Currencies>, Box<dyn std::error::Error + Send + Sync>> {
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v3/currencies",
                None,
                None,
                false,
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
    ) -> Result<Vec<Ticker>, Box<dyn std::error::Error + Send + Sync>> {
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v1/market/allTickers",
                None,
                None,
                false,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ListTickers>(&text) {
                        Ok(r) => match r.code.as_str() {
                            "200000" => Ok(r.data.ticker),
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
        return match self
            .make_request(reqwest::Method::GET, "/api/v2/symbols", None, None, false)
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

    pub async fn api_v1_timestamp(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        return match self
            .make_request(reqwest::Method::GET, "/api/v1/timestamp", None, None, false)
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ApiV1Timestamp>(&text) {
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
        let string_to_sign = format!("{}{}{}{}", timestamp, method, endpoint, query_string);
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        base64::encode(result.into_bytes())
    }

    fn generate_passphrase_signature(&self) -> String {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(self.api_passphrase.as_bytes());
        let result = mac.finalize();
        base64::encode(result.into_bytes())
    }
    async fn make_request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        query_params: Option<HashMap<&str, &str>>,
        body: Option<HashMap<&str, &str>>,
        authenticated: bool,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request_builder = self.client.request(method.clone(), &url);

        if let Some(params) = &query_params {
            request_builder = request_builder.query(&params);
        }

        if let Some(body_data) = &body {
            request_builder = request_builder.json(&body_data);
        }
        if authenticated {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let query_string = query_params
                .as_ref()
                .map(|params| {
                    let mut pairs: Vec<String> =
                        params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                    pairs.sort();
                    pairs.join("&")
                })
                .unwrap_or_default();

            let body_str = body
                .as_ref()
                .map(|b| serde_json::to_string(b).unwrap())
                .unwrap_or_default();

            let signature = self.generate_signature(
                timestamp,
                &method.to_string(),
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
        }

        let response = request_builder.send().await?;

        Ok(response)
    }
}
