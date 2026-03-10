use hl_rs::UpdateLeverage;

use crate::cli::{LeverageAction, MarginMode, Network};
use crate::client::{exchange_client, resolve_asset_index};
use crate::confirm::confirm_action;
use crate::error::CliError;
use crate::output::print_json;

pub async fn run(
    network: &Network,
    json: bool,
    yes: bool,
    action: LeverageAction,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let LeverageAction::Set {
        coin,
        leverage,
        mode,
    } = action;

    let asset_index = resolve_asset_index(network, dex, &coin).await?;
    let mode_str = match mode {
        MarginMode::Cross => "cross",
        MarginMode::Isolated => "isolated",
    };

    confirm_action(
        &format!("Set leverage for {coin} to {leverage}x ({mode_str})"),
        yes,
    )?;

    let update = match mode {
        MarginMode::Cross => UpdateLeverage::cross(asset_index, leverage),
        MarginMode::Isolated => UpdateLeverage::isolated(asset_index, leverage),
    };

    let client = exchange_client(network)?;
    let response = client.send_action(update).await?;

    if json {
        print_json(&response)?;
    } else {
        println!("Leverage set to {leverage}x ({mode_str}) for {coin}");
    }
    Ok(())
}
