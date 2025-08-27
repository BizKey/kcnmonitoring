use crate::api::common::{Currencies, ListCurrencies};

pub async fn get_currencies() -> Result<Vec<Currencies>, Box<dyn std::error::Error>> {
    let body = reqwest::get("https://api.kucoin.com/api/v3/currencies")
        .await?
        .text()
        .await?;

    let response: ListCurrencies = serde_json::from_str(&body)?;

    if response.code == "200000" {
        Ok(response.data)
    } else {
        Err(format!("API error: code {}", response.code).into())
    }
}
