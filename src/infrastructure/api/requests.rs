// Переносим существующий код из старого api/requests.rs с небольшими изменениями
use crate::infrastructure::config::Config;
use anyhow::{Context, Result};
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, Method, Response};
use sha2::Sha256;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub struct KuCoinClient {
    client: Client,
    api_key: String,
    api_secret: String,
    api_passphrase: String,
    base_url: String,
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

    // ... Остальные методы остаются такими же как в исходном коде
    // (get_system_timestamp_ms, generate_signature, make_request)
    
    pub async fn api_v3_currencies_get(&self) -> Result<String> {
        // как в исходном коде
    }

    pub async fn api_v1_market_all_tickers_get(&self) -> Result<String> {
        // как в исходном коде
    }

    pub async fn api_v2_symbols_get(&self) -> Result<String> {
        // как в исходном коде
    }
}

static KUCLIENT: OnceLock<Result<KuCoinClient>> = OnceLock::new();

pub fn init_client(config: &Config) -> Result<()> {
    KUCLIENT.get_or_init(|| KuCoinClient::new(config));
    Ok(())
}

fn get_client() -> Result<&'static KuCoinClient> {
    KUCLIENT
        .get()
        .ok_or_else(|| anyhow::anyhow!("KuCoin client not initialized"))?
        .as_ref()
        .map_err(|e| anyhow::anyhow!("KuCoin client error: {}", e))
}