use hl_rs::{BatchCancel, CancelByCloid};

use crate::cli::{Network, OrderAction};
use crate::client::{exchange_client, resolve_asset_index};
use crate::confirm::confirm_action;
use crate::error::CliError;
use crate::output::print_json;

pub async fn run_cancel(
    network: &Network,
    json: bool,
    yes: bool,
    action: OrderAction,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let client = exchange_client(network)?;

    let response = match action {
        OrderAction::Cancel { ref coin, oid } => {
            confirm_action(&format!("Cancel order {oid} for {coin}"), yes)?;
            let asset_index = resolve_asset_index(network, dex, coin).await?;
            let cancel = BatchCancel::single(asset_index, oid);
            client.send_action(cancel).await?
        }
        OrderAction::CancelByCloid { ref coin, ref cloid } => {
            confirm_action(&format!("Cancel order (cloid: {cloid}) for {coin}"), yes)?;
            let asset_index = resolve_asset_index(network, dex, coin).await?;
            let cancel = CancelByCloid::single(asset_index, cloid.clone());
            client.send_action(cancel).await?
        }
        _ => unreachable!(),
    };

    if json {
        print_json(&response)?;
    } else {
        println!("Cancel request submitted.");
        print_json(&response)?;
    }
    Ok(())
}
