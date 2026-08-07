use anyhow::{Context, Result};
use std::env;

pub struct Config {
    pub kucoin_base_url: String,
    pub kucoin_key: String,
    pub kucoin_secret: String,
    pub kucoin_passphrase: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            kucoin_base_url: get_env("KUCOIN_BASE_URL")
                .unwrap_or_else(|_| "https://api.kucoin.com".to_string()),
            kucoin_key: get_env("KUCOIN_KEY")?,
            kucoin_secret: get_env("KUCOIN_SECRET")?,
            kucoin_passphrase: get_env("KUCOIN_PASS")?,
            database_url: get_env("DATABASE_URL").context("DATABASE_URL not set")?,
        })
    }
}

fn get_env(key: &str) -> Result<String> {
    Ok(env::var(key)?.trim().to_string())
}
