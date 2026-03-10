use hl_rs::{BatchCancel, CancelWire};

use crate::cli::Network;
use crate::client::{exchange_client, resolve_asset_index, send_info_request, user_address};
use crate::confirm::confirm_action;
use crate::error::CliError;
use crate::output::print_json;

pub async fn run(
    network: &Network,
    json: bool,
    yes: bool,
    coin: Option<&str>,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let address = user_address()?;
    let request = serde_json::json!({
        "type": "openOrders",
        "user": address.to_string(),
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    let orders = response.as_array().ok_or_else(|| {
        CliError::Api("Unexpected response format for open orders".into())
    })?;

    if orders.is_empty() {
        if !json {
            println!("No open orders to cancel");
        }
        return Ok(());
    }

    // Filter by coin if specified
    let filtered: Vec<&serde_json::Value> = if let Some(filter_coin) = coin {
        orders
            .iter()
            .filter(|o| {
                o.get("coin")
                    .and_then(|v| v.as_str())
                    .map(|c| c.eq_ignore_ascii_case(filter_coin))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        orders.iter().collect()
    };

    if filtered.is_empty() {
        if !json {
            println!("No matching orders to cancel");
        }
        return Ok(());
    }

    let desc = if let Some(c) = coin {
        format!("Cancel {} open orders for {c}", filtered.len())
    } else {
        format!("Cancel ALL {} open orders", filtered.len())
    };
    confirm_action(&desc, yes)?;

    let mut cancels = Vec::new();
    for order in &filtered {
        let order_coin = order.get("coin").and_then(|v| v.as_str()).unwrap_or("");
        let oid = order.get("oid").and_then(|v| v.as_u64()).unwrap_or(0);
        if oid == 0 {
            continue;
        }
        let asset_index = resolve_asset_index(network, dex, order_coin).await?;
        cancels.push(CancelWire { a: asset_index, o: oid });
    }

    if cancels.is_empty() {
        if !json {
            println!("No valid orders to cancel");
        }
        return Ok(());
    }

    let batch = BatchCancel::new(cancels);
    let client = exchange_client(network)?;
    let response = client.send_action(batch).await?;

    if json {
        print_json(&response)?;
    } else {
        println!("Cancelled {} orders.", filtered.len());
    }
    Ok(())
}
