use crate::spot::structs::TokenInfo;

pub async fn get_token_info(token_id: &str) -> anyhow::Result<TokenInfo> {
    let client = reqwest::Client::new();

    let res = client
        .post("https://api.hyperliquid.xyz/info")
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "type":"tokenDetails",
                "tokenId": token_id.to_string()
            })
            .to_string(),
        )
        .send()
        .await?
        .text()
        .await?;

    let token_info: TokenInfo = serde_json::from_str(&res)?;

    Ok(token_info)
}
