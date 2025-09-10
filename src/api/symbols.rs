use crate::api::common::{ListSymbols, Symbol};
use log::error;

pub async fn get_symbols() -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
    let body = match reqwest::get("https://api.kucoin.com/api/v2/symbols").await {
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
            error!("Ошибка при выполнении HTTP-запроса: {}", e);
            return Err(e.into());
        }
    };

    return match serde_json::from_str::<ListSymbols>(&body) {
        Ok(r) => match r.code.as_str() {
            "200000" => Ok(r.data),
            _ => Err(format!("API error: code {}", r.code).into()),
        },
        Err(e) => {
            error!("Ошибка десериализации JSON: {}", e);
            return Err(e.into());
        }
    };
}
