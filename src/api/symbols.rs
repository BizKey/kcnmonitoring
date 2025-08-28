use crate::api::common::{ListSymbols, Symbol};

pub async fn get_symbols() -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
    let body = reqwest::get("https://api.kucoin.com/api/v2/symbols")
        .await?
        .text()
        .await?;

    let response: ListSymbols = serde_json::from_str(&body)?;

    if response.code == "200000" {
        Ok(response.data)
    } else {
        Err(format!("API error: code {}", response.code).into())
    }
}
