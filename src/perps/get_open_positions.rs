use crate::perps::structs::Positions;

pub async fn get_open_positions(user_address: &str) -> anyhow::Result<Positions> {
    let client = reqwest::Client::new();

    let res = client
        .post("https://api.hyperliquid.xyz/info")
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "type": "clearinghouseState",
                "user": user_address.to_string()
            })
            .to_string(),
        )
        .send()
        .await?
        .text()
        .await?;

    let positions: Positions = serde_json::from_str(&res)?;

    Ok(positions)
}
