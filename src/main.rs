use std::time::Duration;
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
mod spot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let spot_tokens = vec![String::from("0x0d01dc56dcaaca66ad901c959b4011ec")];
    let perp_tokens = vec![String::from("HYPE")];
    let user_address = "0xC604589f651bfb2515a408bc1C1013dcb707702C";

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
                &perp_open_positions,
            )
            .await?;
        }
    }
}

async fn close_wrong_arb() {}

async fn check_arb_opportunities(
    spot_token: &str,
    perp_token: &str,
    user_address: &str,
    spot_user_balances: &Balances,
    perp_open_positions: &Positions,
) -> anyhow::Result<()> {
    let spot_token_info = get_spot_token_info(spot_token).await?;
    let perp_token_info = get_perp_token_info(perp_token, user_address).await?;

    Ok(())
}
