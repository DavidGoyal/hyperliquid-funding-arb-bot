use dotenvy::dotenv;
use std::time::Duration;
use tokio::time::interval;

use crate::{
    perps::{
        get_open_positions::get_open_positions,
        get_token_info::get_token_info as get_perp_token_info,
        structs::{Position, TokenDetails as PerpTokenDetails},
    },
    place_order::place_order,
    spot::{
        get_token_info::get_token_info as get_spot_token_info,
        get_user_balances::get_user_balances,
        structs::{Balance, TokenDetails as SpotTokenDetails},
    },
};

mod perps;
mod place_order;
mod sign_action;
mod spot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let spot_tokens = vec![
        SpotTokenDetails {
            name: String::from("HYPE"),
            symbol: String::from("HYPE"),
            asset_id: 107,
            sz_decimals: 2,
            hex_asset_address: String::from("0x0d01dc56dcaaca66ad901c959b4011ec"),
        },
        SpotTokenDetails {
            name: String::from("ETH"),
            symbol: String::from("UETH"),
            asset_id: 151,
            sz_decimals: 4,
            hex_asset_address: String::from("0xe1edd30daaf5caac3fe63569e24748da"),
        },
        SpotTokenDetails {
            name: String::from("SOL"),
            symbol: String::from("USOL"),
            asset_id: 156,
            sz_decimals: 3,
            hex_asset_address: String::from("0x49b67c39f5566535de22b29b0e51e685"),
        },
        SpotTokenDetails {
            name: String::from("kBONK"),
            symbol: String::from("UBONK"),
            asset_id: 194,
            sz_decimals: 0,
            hex_asset_address: String::from("0xb113d34e351cf195733c98442530c099"),
        },
        SpotTokenDetails {
            name: String::from("FARTCOIN"),
            symbol: String::from("UFART"),
            asset_id: 162,
            sz_decimals: 1,
            hex_asset_address: String::from("0x7650808198966e4285687d3deb556ccc"),
        },
    ];
    let perp_tokens = vec![
        PerpTokenDetails {
            name: String::from("HYPE"),
            asset_id: 159,
            sz_decimals: 2,
        },
        PerpTokenDetails {
            name: String::from("ETH"),
            asset_id: 1,
            sz_decimals: 4,
        },
        PerpTokenDetails {
            name: String::from("SOL"),
            asset_id: 5,
            sz_decimals: 2,
        },
        PerpTokenDetails {
            name: String::from("kBONK"),
            asset_id: 85,
            sz_decimals: 0,
        },
        PerpTokenDetails {
            name: String::from("FARTCOIN"),
            asset_id: 165,
            sz_decimals: 1,
        },
    ];
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

        let mut available_to_trade = if spot_balance > perp_balance {
            perp_balance
        } else {
            spot_balance
        };

        for i in 0..perp_open_positions.asset_positions.len() {
            let perp_token = &perp_open_positions.asset_positions[i].position.coin;
            let perp_token_details = perp_tokens
                .iter()
                .find(|token| token.name == *perp_token)
                .unwrap();
            let spot_token_details = spot_tokens
                .iter()
                .find(|token| token.name == *perp_token)
                .unwrap();

            let perp_position = &perp_open_positions.asset_positions[i].position;
            let spot_position = spot_user_balances
                .balances
                .iter()
                .find(|balance| balance.coin == spot_token_details.symbol);

            // Skip if there's no corresponding spot position
            let Some(spot_position) = spot_position else {
                continue;
            };

            close_wrong_arb(
                &spot_token_details,
                &perp_token_details,
                &user_address,
                &spot_position,
                &perp_position,
                &user_private_key,
            )
            .await?;
        }

        if available_to_trade < 11.0 {
            println!("Available to trade is less than 11.0");
            continue;
        }

        for i in 0..spot_tokens.len() {
            check_arb_opportunities(
                &spot_tokens[i],
                &perp_tokens[i],
                &mut available_to_trade,
                &user_address,
                &user_private_key,
            )
            .await?;
        }
    }
}

async fn close_wrong_arb(
    spot_token: &SpotTokenDetails,
    perp_token: &PerpTokenDetails,
    user_address: &str,
    spot_position: &Balance,
    perp_position: &Position,
    user_private_key: &str,
) -> anyhow::Result<()> {
    let spot_token_info = get_spot_token_info(&spot_token.hex_asset_address).await?;
    let perp_token_info = get_perp_token_info(&perp_token.name, user_address).await?;

    let perp_mark_px = perp_token_info.mark_px.parse::<f64>().unwrap();
    let spot_mark_px = spot_token_info.mark_px.parse::<f64>().unwrap();

    if perp_mark_px < spot_mark_px {
        // long on perp, sell on spot
        println!(
            "Closing wrong arb for {} and {}",
            spot_token.name, perp_token.name
        );
        place_order(
            user_private_key,
            perp_mark_px,
            "buy",
            perp_position.szi.parse::<f64>().unwrap().abs(),
            perp_token.asset_id,
            perp_token.sz_decimals,
            6 - perp_token.sz_decimals,
            true,
        )
        .await?;

        place_order(
            user_private_key,
            spot_mark_px,
            "sell",
            spot_position.total.parse::<f64>().unwrap(),
            10000 + spot_token.asset_id,
            spot_token.sz_decimals,
            8 - spot_token.sz_decimals,
            true,
        )
        .await?;
    }

    Ok(())
}

async fn check_arb_opportunities(
    spot_token: &SpotTokenDetails,
    perp_token: &PerpTokenDetails,
    available_to_trade: &mut f64,
    user_address: &str,
    user_private_key: &str,
) -> anyhow::Result<()> {
    if *available_to_trade < 11.0 {
        println!("Available to trade is less than 11.0");
        return Ok(());
    }

    let spot_token_info = get_spot_token_info(&spot_token.hex_asset_address).await?;
    let perp_token_info = get_perp_token_info(&perp_token.name, user_address).await?;

    let perp_mark_px = perp_token_info.mark_px.parse::<f64>().unwrap();
    let spot_mark_px = spot_token_info.mark_px.parse::<f64>().unwrap();

    if perp_mark_px > spot_mark_px {
        // short on perp, long on spot
        place_order(
            user_private_key,
            perp_mark_px,
            "sell",
            11.0,
            perp_token.asset_id,
            perp_token.sz_decimals,
            6 - perp_token.sz_decimals,
            false,
        )
        .await?;

        place_order(
            user_private_key,
            spot_mark_px,
            "buy",
            11.0,
            10000 + spot_token.asset_id,
            spot_token.sz_decimals,
            8 - spot_token.sz_decimals,
            false,
        )
        .await?;

        *available_to_trade -= 11.0;
    }

    Ok(())
}
