use crate::spot::structs::Balances;

pub async fn get_user_balances(user_address: &str) -> anyhow::Result<Balances> {
    let client = reqwest::Client::new();

    let res = client
        .post("https://api.hyperliquid.xyz/info")
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "type": "spotClearinghouseState",
                "user": user_address.to_string()
            })
            .to_string(),
        )
        .send()
        .await?
        .text()
        .await?;

    let balances: Balances = serde_json::from_str(&res)?;

    Ok(balances)
}
