use crate::api::common::{ListSymbols, Symbol};
use log::error;
pub async fn get_symbols() -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
    let body = match reqwest::get("https://api.kucoin.com/api/v2/symbols").await {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(e) => {
                error!("Ошибка при получении текста ответа: {}", e);
                return Err(e.into());
            }
        },
        Err(e) => {
            error!("Ошибка при выполнении HTTP-запроса: {}", e);
            return Err(e.into());
        }
    };

    let response: ListSymbols = serde_json::from_str(&body)?;

    if response.code == "200000" {
        Ok(response.data)
    } else {
        Err(format!("API error: code {}", response.code).into())
    }
}
