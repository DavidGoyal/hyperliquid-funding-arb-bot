use serde::Deserialize;

pub struct TokenDetails {
    pub name: String,
    pub asset_id: u32,
    pub sz_decimals: u8,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Positions {
    pub asset_positions: Vec<AssetPosition>,
    pub withdrawable: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
    pub position: Position,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub coin: String,
    pub szi: String,
    pub entry_px: String,
    pub liquidation_px: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub leverage: Leverage,
    pub mark_px: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    #[serde(rename = "type")]
    pub leverage_type: String,
    pub value: u8,
}
