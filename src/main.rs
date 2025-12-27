use dotenvy::dotenv;
use std::{collections::HashMap, time::Duration};
use tokio::time::interval;

use crate::{
    perps::{
        get_open_positions::get_open_positions,
        get_token_info::get_token_info as get_perp_token_info, structs::Position,
    },
    place_order::place_order,
    spot::{
        get_token_info::get_token_info as get_spot_token_info,
        get_user_balances::get_user_balances, structs::Balance,
    },
};

mod perps;
mod place_order;
mod sign_action;
mod spot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let spot_tokens = vec![String::from("0x0d01dc56dcaaca66ad901c959b4011ec")];
    let perp_tokens = vec![String::from("HYPE")];
    let perp_to_spot_mapping = HashMap::from([(
        String::from("HYPE"),
        String::from("0x0d01dc56dcaaca66ad901c959b4011ec"),
    )]);
    let user_address = std::env::var("WALLET_ADDRESS").expect("WALLET_ADDRESS must be set");
    let user_private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");

    let mut interval = interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        let spot_user_balances = get_user_balances(&user_address).await?;
        let perp_open_positions = get_open_positions(&user_address).await?;

        let usdc_balance = spot_user_balances
            .balances
            .iter()
            .find(|balance| balance.coin == "USDC");

        let mut spot_balance = 0.0;
        if usdc_balance.is_some() {
            spot_balance = usdc_balance.unwrap().total.parse::<f64>().unwrap();
        }

        let perp_balance = perp_open_positions.withdrawable.parse::<f64>().unwrap();

        let available_to_trade = if spot_balance > perp_balance {
            perp_balance
        } else {
            spot_balance
        };
        if available_to_trade < 11.0 {
            println!("Available to trade is less than 11.0");
            continue;
        }

        for i in 0..perp_open_positions.asset_positions.len() {
            let perp_token = &perp_open_positions.asset_positions[i].position.coin;
            let spot_token = perp_to_spot_mapping.get(perp_token).unwrap();

            let perp_position = &perp_open_positions.asset_positions[i].position;
            let spot_position = spot_user_balances
                .balances
                .iter()
                .find(|balance| balance.coin == *perp_token)
                .unwrap();

            close_wrong_arb(
                &spot_token,
                &perp_token,
                &user_address,
                &spot_position,
                &perp_position,
                &user_private_key,
            )
            .await?;
        }

        for i in 0..spot_tokens.len() {
            check_arb_opportunities(
                &spot_tokens[i],
                &perp_tokens[i],
                &available_to_trade,
                &user_address,
                &user_private_key,
            )
            .await?;
        }
    }
}

async fn close_wrong_arb(
    spot_token: &str,
    perp_token: &str,
    user_address: &str,
    spot_position: &Balance,
    perp_position: &Position,
    user_private_key: &str,
) -> anyhow::Result<()> {
    let spot_token_info = get_spot_token_info(spot_token).await?;
    let perp_token_info = get_perp_token_info(perp_token, user_address).await?;

    let perp_mark_px = perp_token_info.mark_px.parse::<f64>().unwrap();
    let spot_mark_px = spot_token_info.mark_px.parse::<f64>().unwrap();

    if perp_mark_px > spot_mark_px {
        // long on perp, sell on spot
    }

    Ok(())
}

async fn check_arb_opportunities(
    spot_token: &str,
    perp_token: &str,
    available_to_trade: &f64,
    user_address: &str,
    user_private_key: &str,
) -> anyhow::Result<()> {
    let spot_token_info = get_spot_token_info(spot_token).await?;
    let perp_token_info = get_perp_token_info(perp_token, user_address).await?;

    let perp_mark_px = perp_token_info.mark_px.parse::<f64>().unwrap();
    let spot_mark_px = spot_token_info.mark_px.parse::<f64>().unwrap();

    // if perp_mark_px > spot_mark_px {
    //     println!("Short on perp, long on spot");

    place_order(user_private_key, perp_mark_px, "buy", 0.43, 159, 2, 3, true).await?;
    // }

    Ok(())
}
