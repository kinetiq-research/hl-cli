use hl_rs::UpdateLeverage;

use crate::cli::{LeverageAction, MarginMode, Network};
use crate::client::{exchange_client, resolve_asset_index};
use crate::error::CliError;
use crate::output::print_json;

pub async fn run(
    network: &Network,
    json: bool,
    action: LeverageAction,
) -> Result<(), CliError> {
    let LeverageAction::Set {
        coin,
        leverage,
        mode,
    } = action;

    let asset_index = resolve_asset_index(network, &coin).await?;

    let update = match mode {
        MarginMode::Cross => UpdateLeverage::cross(asset_index, leverage),
        MarginMode::Isolated => UpdateLeverage::isolated(asset_index, leverage),
    };

    let client = exchange_client(network)?;
    let response = client.send_action(update).await?;

    if json {
        print_json(&response)?;
    } else {
        let mode_str = match mode {
            MarginMode::Cross => "cross",
            MarginMode::Isolated => "isolated",
        };
        println!("Leverage set to {leverage}x ({mode_str}) for {coin}");
    }
    Ok(())
}
