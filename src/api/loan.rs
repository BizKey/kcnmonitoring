use crate::api::common::{ListLoanMarket, LoanMarket};
use log::error;

pub async fn get_loan_market() -> Result<Vec<LoanMarket>, Box<dyn std::error::Error>> {
    return match reqwest::get("https://api.kucoin.com/api/v3/project/list").await {
        Ok(response) => match response.status().as_str() {
            "200" => match response.text().await {
                Ok(text) => match serde_json::from_str::<ListLoanMarket>(&text) {
                    Ok(r) => match r.code.as_str() {
                        "200000" => Ok(r.data),
                        _ => Err(format!("API error: code {}", r.code).into()),
                    },
                    Err(e) => {
                        error!("Ошибка десериализации JSON: {}", e);
                        return Err(e.into());
                    }
                },
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
}
