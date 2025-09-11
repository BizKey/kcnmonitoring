use crate::api::common::{
    ApiV1Timestamp, Currencies, ListCurrencies, ListLoanMarket, ListSymbols, ListTickers,
    LoanMarket, Symbol, Ticker,
};
use log::error;
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KuCoinClient {
    client: Client,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    base_url: String,
}

impl KuCoinClient {
    pub fn new(
        api_key: String,
        api_secret: String,
        api_passphrase: String,
        base_url: String,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_secret,
            api_passphrase,
            base_url,
        }
    }

    pub async fn api_v3_project_list(
        &self,
    ) -> Result<Vec<LoanMarket>, Box<dyn std::error::Error + Send + Sync>> {
        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v3/project/list",
                None,
                None,
                false,
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
                        Err(e) => {
                            error!("Ошибка десериализации JSON: {}", e);
                            return Err(e.into());
                        }
                    },
                    Err(e) => {
                        error!("Ошибка при получении текста ответа: {}", e);
                        return Err(e.into());
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
                    Err(_) => {
                        return Err(format!("Wrong HTTP status: '{}'", status).into());
                    }
                },
            },
            Err(e) => {
                error!("ошибка при получении HTTP-запроса: {}", e);
                return Err(e.into());
            }
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
                        Err(e) => {
                            error!("Ошибка десериализации JSON: {}", e);
                            return Err(e.into());
                        }
                    },
                    Err(e) => {
                        error!("Ошибка при получении текста ответа: {}", e);
                        return Err(e.into());
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
                    Err(_) => {
                        return Err(format!("Wrong HTTP status: '{}'", status).into());
                    }
                },
            },
            Err(e) => {
                error!("ошибка при получении HTTP-запроса: {}", e);
                return Err(e.into());
            }
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
                        Err(e) => {
                            error!("Ошибка десериализации JSON: {}", e);
                            return Err(e.into());
                        }
                    },
                    Err(e) => {
                        error!("Ошибка при получении текста ответа: {}", e);
                        return Err(e.into());
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
                    Err(_) => {
                        return Err(format!("Wrong HTTP status: '{}'", status).into());
                    }
                },
            },
            Err(e) => {
                error!("ошибка при получении HTTP-запроса: {}", e);
                return Err(e.into());
            }
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
                        Err(e) => {
                            error!("Ошибка десериализации JSON: {}", e);
                            return Err(e.into());
                        }
                    },
                    Err(e) => {
                        error!("Ошибка при получении текста ответа: {}", e);
                        return Err(e.into());
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
                    Err(_) => {
                        return Err(format!("Wrong HTTP status: '{}'", status).into());
                    }
                },
            },
            Err(e) => {
                error!("ошибка при получении HTTP-запроса: {}", e);
                return Err(e.into());
            }
        };
    }

    pub async fn api_v1_timestamp(
        &self,
    ) -> Result<ApiV1Timestamp, Box<dyn std::error::Error + Send + Sync>> {
        return match self
            .make_request(reqwest::Method::GET, "/api/v1/timestamp", None, None, false)
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ApiV1Timestamp>(&text) {
                        Ok(r) => match r.code.as_str() {
                            "200000" => Ok(r),
                            _ => Err(format!("API error: code {}", r.code).into()),
                        },
                        Err(e) => {
                            error!("Ошибка десериализации JSON: {}", e);
                            return Err(e.into());
                        }
                    },
                    Err(e) => {
                        error!("Ошибка при получении текста ответа: {}", e);
                        return Err(e.into());
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
                    Err(_) => {
                        return Err(format!("Wrong HTTP status: '{}'", status).into());
                    }
                },
            },
            Err(e) => {
                error!("ошибка при получении HTTP-запроса: {}", e);
                return Err(e.into());
            }
        };
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

        let response = request_builder.send().await?;

        Ok(response)
    }
}
