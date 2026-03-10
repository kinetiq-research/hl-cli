use hl_rs::{BatchOrder, LimitOrderType, OrderType, OrderWire, Tif};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

use crate::cli::Network;
use crate::client::{exchange_client, resolve_asset_index};
use crate::confirm::confirm_action;
use crate::error::CliError;
use crate::output::print_json;

#[derive(Deserialize)]
struct OrderSpec {
    coin: String,
    side: String,
    size: String,
    price: String,
    #[serde(default = "default_tif")]
    tif: String,
    #[serde(default)]
    reduce_only: bool,
    #[serde(default)]
    cloid: Option<String>,
}

fn default_tif() -> String {
    "gtc".to_string()
}

pub async fn run(
    network: &Network,
    json: bool,
    yes: bool,
    orders_json: &str,
    dex: Option<&str>,
) -> Result<(), CliError> {
    // Load JSON from string or file (@ prefix)
    let json_str = if let Some(path) = orders_json.strip_prefix('@') {
        std::fs::read_to_string(path)
            .map_err(|e| CliError::InvalidArg(format!("Cannot read file {path}: {e}")))?
    } else {
        orders_json.to_string()
    };

    let specs: Vec<OrderSpec> = serde_json::from_str(&json_str)
        .map_err(|e| CliError::InvalidArg(format!("Invalid JSON: {e}")))?;

    if specs.is_empty() {
        return Err(CliError::InvalidArg("Empty order list".into()));
    }

    confirm_action(
        &format!("Place {} orders in batch", specs.len()),
        yes,
    )?;

    let mut wires = Vec::with_capacity(specs.len());
    for spec in &specs {
        let asset_index = resolve_asset_index(network, dex, &spec.coin).await?;
        let is_buy = spec.side.eq_ignore_ascii_case("buy");
        let price = Decimal::from_str(&spec.price)
            .map_err(|e| CliError::InvalidArg(format!("Bad price '{}': {e}", spec.price)))?;
        let size = Decimal::from_str(&spec.size)
            .map_err(|e| CliError::InvalidArg(format!("Bad size '{}': {e}", spec.size)))?;

        let tif = match spec.tif.to_lowercase().as_str() {
            "gtc" => Tif::Gtc,
            "ioc" => Tif::Ioc,
            "alo" => Tif::Alo,
            other => return Err(CliError::InvalidArg(format!("Unknown tif: {other}"))),
        };

        wires.push(OrderWire {
            a: asset_index,
            b: is_buy,
            p: price,
            s: size,
            r: spec.reduce_only,
            t: OrderType::Limit(LimitOrderType { tif }),
            c: spec.cloid.clone(),
        });
    }

    let batch = BatchOrder::new(wires);
    let client = exchange_client(network)?;
    let response = client.send_action(batch).await?;

    if json {
        print_json(&response)?;
    } else {
        println!("Batch of {} orders submitted.", specs.len());
        print_json(&response)?;
    }
    Ok(())
}
