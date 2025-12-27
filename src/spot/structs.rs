use serde::Deserialize;

pub struct TokenDetails {
    pub name: String,
    pub symbol: String,
    pub asset_id: u32,
    pub sz_decimals: u8,
    pub hex_asset_address: String,
}

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
    pub token: u64,
    pub hold: String,
    pub total: String,
    pub entry_ntl: String,
}
