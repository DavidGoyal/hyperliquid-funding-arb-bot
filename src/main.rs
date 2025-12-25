use dotenvy::dotenv;
use std::{collections::HashMap, time::Duration};
use tokio::time::interval;

use crate::{
    perps::{
        get_open_positions::get_open_positions,
        get_token_info::get_token_info as get_perp_token_info, structs::Positions,
    },
    spot::{
        get_token_info::get_token_info as get_spot_token_info,
        get_user_balances::get_user_balances, structs::Balances,
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
    let spot_to_perp_mapping = HashMap::from([(
        String::from("0x0d01dc56dcaaca66ad901c959b4011ec"),
        String::from("HYPE"),
    )]);
    let user_address = std::env::var("USER_ADDRESS").expect("USER_ADDRESS must be set");

    let mut interval = interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        let spot_user_balances = get_user_balances(&user_address).await?;
        let perp_open_positions = get_open_positions(&user_address).await?;

        for i in 0..spot_tokens.len() {
            check_arb_opportunities(
                &spot_tokens[i],
                &perp_tokens[i],
                &user_address,
                &spot_user_balances,
            )
            .await?;
        }
    }
}

async fn close_wrong_arb(
    spot_token: &str,
    perp_token: &str,
    user_address: &str,
    spot_user_balances: &Balances,
    perp_open_positions: &Positions,
) -> anyhow::Result<()> {
    let spot_token_info = get_spot_token_info(spot_token).await?;
    let perp_token_info = get_perp_token_info(perp_token, user_address).await?;

    if perp_token_info.mark_px < spot_token_info.mark_px {
        println!("Long on perp, short on spot");

        // let present_in_perp_open_positions = perp_open_positions
        //     .asset_positions
        //     .iter()
        //     .any(|position| position == perp_token);

        // if present_in_perp_open_positions {
        //     println!("Present in perp open positions");
        // } else {
        //     println!("Not present in perp open positions");
        // }
    }

    Ok(())
}

async fn check_arb_opportunities(
    spot_token: &str,
    perp_token: &str,
    user_address: &str,
    spot_user_balances: &Balances,
) -> anyhow::Result<()> {
    let spot_token_info = get_spot_token_info(spot_token).await?;
    let perp_token_info = get_perp_token_info(perp_token, user_address).await?;

    if perp_token_info.mark_px > spot_token_info.mark_px {
        println!("Short on perp, long on spot");

        let present_in_spot_balances = spot_user_balances
            .balances
            .iter()
            .any(|balance| balance.coin == spot_token);

        if present_in_spot_balances {
            println!("Present in spot balances");
        } else {
            println!("Not present in spot balances");
        }
    }

    Ok(())
}
