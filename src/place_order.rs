use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::sign_action::{ExchangePayload, Limit, Order, OrderType, SignAction, sign_action};

fn float_to_wire(x: f64, decimals: u32) -> String {
    let rounded = format!("{:.prec$}", x, prec = decimals as usize);
    let decimal = Decimal::from_str(&rounded).expect("Failed to parse decimal");
    decimal.normalize().to_string()
}

pub async fn place_order(
    private_key: &str,
    mark_px: f64,
    side: &str,
    amount: f64,
    asset_id: u32,
    sz_decimals: u32,
    pz_decimals: u32,
    is_opposite: bool,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let current_time = Utc::now().timestamp_millis();
    let limit_px = if matches!(side, "buy") {
        mark_px * 1.01
    } else {
        mark_px * 0.99
    };

    let nonce = current_time as u64;
    let expires_after = nonce + 10000;

    let action = SignAction {
        type_: "order".to_string(),
        orders: vec![Order {
            a: asset_id,
            b: matches!(side, "buy"),
            p: float_to_wire(limit_px, pz_decimals),
            s: if is_opposite {
                float_to_wire(amount, sz_decimals)
            } else {
                float_to_wire(amount / limit_px, sz_decimals)
            },
            r: false,
            t: OrderType {
                limit: Limit {
                    tif: "Ioc".to_string(),
                },
            },
        }],
        grouping: "na".to_string(),
    };

    let signature = sign_action(private_key, &action, nonce, expires_after).await?;

    let payload = ExchangePayload {
        action: action,
        nonce,
        signature,
        vault_address: None,
        expires_after,
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
