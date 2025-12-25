use chrono::Utc;

use crate::sign_action::{
    Action, ExchangePayload, Limit, Order, OrderType, SignAction, sign_action,
};

pub async fn place_order(
    private_key: &str,
    mark_px: f64,
    side: &str,
    amount: f64,
    asset_id: u32,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let current_time = Utc::now().timestamp_millis();
    let limit_px = if matches!(side, "buy") {
        mark_px * 1.01
    } else {
        mark_px * 0.99
    };

    let nonce = current_time as u64;
    let expires_after = (current_time + 10000) as u64;

    let action = Action {
        action: SignAction {
            type_: "order".to_string(),
            orders: vec![Order {
                a: asset_id,
                b: matches!(side, "buy"),
                p: format!("{:.8}", limit_px),
                s: format!("{:.8}", amount / limit_px),
                r: false,
                t: OrderType {
                    limit: Limit {
                        tif: "Ioc".to_string(),
                    },
                },
            }],
            grouping: "na".to_string(),
        },
    };

    let signature = sign_action(private_key, &action, nonce, expires_after).await?;

    let payload = ExchangePayload {
        action: action,
        signature,
        nonce,
    };

    let payload_json = serde_json::to_string(&payload)?;

    let res = client
        .post("https://api.hyperliquid.xyz/exchange")
        .header("Content-Type", "application/json")
        .body(payload_json)
        .send()
        .await?
        .text()
        .await?;

    println!("{}", res);

    Ok(())
}
