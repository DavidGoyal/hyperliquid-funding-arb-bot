use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub name: String,
    pub mid_px: String,
    pub mark_px: String,
    pub prev_day_px: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Balances {
    pub balances: Vec<Balance>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin: String,
    pub token: u8,
    pub hold: String,
    pub total: String,
    pub entry_ntl: String,
}
