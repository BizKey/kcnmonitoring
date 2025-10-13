use crate::api::models::{
    ApiV3MarginBorrowRate, ApiV3MarginBorrowRateData, Candle, Currencies, ListCandle,
    ListCurrencies, ListLoanMarket, ListSymbols, ListTickers, LoanMarket, Symbol, TickerData,
};
use base64::Engine;
use hmac::{Hmac, Mac};
use log::{error, info};

use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct KuCoinClient {
    client: Client,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct WebSocketSubscribe {
    id: u64,
    r#type: String,
    topic: String,
    privateChannel: bool,
    response: bool,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketKlineData {
    pub symbol: String,
    pub candles: Vec<String>, // [timestamp, open, close, high, low, volume, turnover]
    pub time: u64,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketKlineMessage {
    pub data: WebSocketKlineData,
    pub subject: String,
    pub topic: String,
    pub r#type: String,
}

pub struct KucoinWebSocketClient {
    base_url: String,
}

impl KucoinWebSocketClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn connect_and_subscribe(
        &self,
        symbols: Vec<String>,
        interval: String,
    ) -> Result<mpsc::Receiver<WebSocketKlineData>, Box<dyn std::error::Error>> {
        // Получаем WebSocket токен
        let token = self.get_websocket_token().await?;
        let ws_url = format!("{}/?token={}", self.base_url, token);

        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut write, read) = ws_stream.split();

        // Создаем канал для передачи данных
        let (tx, rx) = mpsc::channel(100);

        // Запускаем обработку сообщений в отдельной задаче
        tokio::spawn(async move {
            Self::handle_websocket_messages(read, write, symbols, interval, tx).await;
        });

        Ok(rx)
    }

    async fn handle_websocket_messages(
        mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
        mut write: impl SinkExt<Message> + Unpin,
        symbols: Vec<String>,
        interval: String,
        tx: mpsc::Sender<WebSocketKlineData>,
    ) {
        // Подписываемся на каждый символ
        for symbol in symbols {
            let topic = format!("/market/candles:{}_{}", symbol, interval);
            let subscribe_msg = WebSocketSubscribe {
                id: 1,
                r#type: "subscribe".to_string(),
                topic,
                privateChannel: false,
                response: true,
            };

            if let Ok(json_msg) = serde_json::to_string(&subscribe_msg) {
                if let Err(e) = write.send(Message::Text(json_msg)).await {
                    error!("Ошибка отправки подписки: {}", e);
                    continue;
                }
                info!("Подписались на: {}_{}", symbol, interval);
                tokio::time::sleep(Duration::from_millis(100)).await; // Небольшая задержка между подписками
            }
        }

        // Обрабатываем входящие сообщения
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(kline_msg) = serde_json::from_str::<WebSocketKlineMessage>(&text) {
                        if kline_msg.topic.contains("candles") {
                            // Отправляем данные через канал
                            if tx.send(kline_msg.data).await.is_err() {
                                break; // Получатель больше не слушает
                            }
                        }
                    }
                }
                Ok(Message::Ping(_)) => {
                    // Автоматически обрабатывается
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket соединение закрыто");
                    break;
                }
                Err(e) => {
                    error!("WebSocket ошибка: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    async fn get_websocket_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Используем ваш существующий KuCoinClient для получения токена
        let client = KuCoinClient::new("https://api.kucoin.com".to_string())?;

        // KuCoin требует POST запрос для получения токена WebSocket
        // Здесь нужно реализовать получение токена через API
        // Для демонстрации возвращаем заглушку
        Ok("your_websocket_token_here".to_string())
    }
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
    ) -> Result<TickerData, Box<dyn std::error::Error + Send + Sync>> {
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
    pub async fn api_v1_market_candles(
        &self,
        symbol_name: String,
        type_candles: String,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        let mut query_params = HashMap::new();

        query_params.insert("symbol", symbol_name.as_str());
        query_params.insert("type", type_candles.as_str());

        return match self
            .make_request(
                reqwest::Method::GET,
                "/api/v1/market/candles",
                Some(query_params),
                None,
                false,
            )
            .await
        {
            Ok(response) => match response.status().as_str() {
                "200" => match response.text().await {
                    Ok(text) => match serde_json::from_str::<ListCandle>(&text) {
                        Ok(r) => match r.code.as_str() {
                            "200000" => {
                                let candles = r
                                    .into_candles()
                                    .map_err(|e| format!("Failed to parse candles: {}", e))?;
                                // Теперь у вас есть Vec<Candle>
                                Ok(candles)
                            }
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
        }

        let response = request_builder.send().await.map_err(|e| {
            // Детальный анализ ошибки reqwest
            if e.is_timeout() {
                format!("Timeout {}: {}", url, e)
            } else if e.is_connect() {
                format!("Error connection {}: {}", url, e)
            } else if e.is_request() {
                format!("Error prepare request {}: {}", url, e)
            } else if e.is_body() {
                format!("Error in body {}: {}", url, e)
            } else {
                format!("Unexpected error {}: {}", url, e)
            }
        })?;

        Ok(response)
    }
}
