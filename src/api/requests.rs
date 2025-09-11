use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub async fn get_server_time(&self) -> Result<String, Box<dyn std::error::Error>> {
        let response = self
            .make_request(reqwest::Method::GET, "/api/v1/timestamp", None, None, false)
            .await?;

        let timestamp: String = response.text().await?;
        Ok(timestamp)
    }
    async fn make_request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        query_params: Option<HashMap<&str, &str>>,
        body: Option<HashMap<&str, &str>>,
        authenticated: bool,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request_builder = self.client.request(method.clone(), &url);

        let response = request_builder.send().await?;

        Ok(response)
    }
}
