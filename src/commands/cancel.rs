use hl_rs::{BatchCancel, CancelByCloid};

use crate::cli::{Network, OrderAction};
use crate::client::{exchange_client, resolve_asset_index};
use crate::error::CliError;
use crate::output::print_json;

pub async fn run_cancel(network: &Network, json: bool, action: OrderAction) -> Result<(), CliError> {
    let client = exchange_client(network)?;

    let response = match action {
        OrderAction::Cancel { coin, oid } => {
            let asset_index = resolve_asset_index(network, &coin).await?;
            let cancel = BatchCancel::single(asset_index, oid);
            client.send_action(cancel).await?
        }
        OrderAction::CancelByCloid { coin, cloid } => {
            let asset_index = resolve_asset_index(network, &coin).await?;
            let cancel = CancelByCloid::single(asset_index, cloid);
            client.send_action(cancel).await?
        }
        _ => unreachable!(),
    };

    if json {
        print_json(&response)?;
    } else {
        println!("Cancel result: {:?}", response);
    }
    Ok(())
}
