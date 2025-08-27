use crate::api::common::{ListTickers, Ticker};

pub async fn get_tickers() -> Result<Vec<Ticker>, Box<dyn std::error::Error>> {
    let body = reqwest::get("https://api.kucoin.com/api/v1/market/allTickers")
        .await?
        .text()
        .await?;

    let response: ListTickers = serde_json::from_str(&body)?;

    if response.code == "200000" {
        Ok(response.data.ticker)
    } else {
        Err(format!("API error: code {}", response.code).into())
    }
}
