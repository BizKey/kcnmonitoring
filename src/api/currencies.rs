use crate::api::common::{Currencies, ListCurrencies};
use log::error;
pub async fn get_currencies() -> Result<Vec<Currencies>, Box<dyn std::error::Error>> {
    let body = match reqwest::get("https://api.kucoin.com/api/v3/currencies").await {
        Ok(response) => match response.status().as_u16() {
            200 => match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    error!("Ошибка при получении текста ответа: {}", e);
                    return Err(e.into());
                }
            },
            status => {
                error!("Получен неуспешный HTTP статус: {}", status);
                return Err(format!("HTTP ошибка: статус {}", status).into());
            }
        },
        Err(e) => {
            error!("ошибка при получении HTTP-запроса: {}", e);
            return Err(e.into());
        }
    };

    let response: ListCurrencies = serde_json::from_str(&body)?;

    if response.code == "200000" {
        Ok(response.data)
    } else {
        Err(format!("API error: code {}", response.code).into())
    }
}
