use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Currency {
    pub currency: String,
    pub name: String,
    pub full_name: String,
    pub precision: i16,
    pub is_margin_enabled: bool,
    pub is_debit_enabled: bool,
}

impl Currency {
    pub fn new(
        currency: String,
        name: String,
        full_name: String,
        precision: i16,
        is_margin_enabled: bool,
        is_debit_enabled: bool,
    ) -> Self {
        Self {
            currency,
            name,
            full_name,
            precision,
            is_margin_enabled,
            is_debit_enabled,
        }
    }
}
