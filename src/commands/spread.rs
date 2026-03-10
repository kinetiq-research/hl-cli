use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::cli::Network;
use crate::client::{resolve_coin_and_dex, send_info_request};
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run(
    network: &Network,
    json: bool,
    coin: &str,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let (_, full_coin) = resolve_coin_and_dex(dex, coin);
    let request = serde_json::json!({
        "type": "l2Book",
        "coin": full_coin,
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    let levels = response
        .get("levels")
        .and_then(|v| v.as_array())
        .ok_or_else(|| CliError::Api("No order book data".into()))?;

    if levels.len() < 2 {
        return Err(CliError::Api("Incomplete order book".into()));
    }

    let bids = levels[0].as_array();
    let asks = levels[1].as_array();

    let best_bid = bids
        .and_then(|b| b.first())
        .and_then(|b| b.get("px"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok());

    let best_ask = asks
        .and_then(|a| a.first())
        .and_then(|a| a.get("px"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok());

    let (bid, ask) = match (best_bid, best_ask) {
        (Some(b), Some(a)) => (b, a),
        _ => return Err(CliError::Api("Cannot determine bid/ask".into())),
    };

    let spread = ask - bid;
    let midpoint = (ask + bid) / dec!(2);
    let spread_bps = if midpoint > dec!(0) {
        (spread / midpoint * dec!(10000)).round_dp(2)
    } else {
        dec!(0)
    };

    if json {
        let info = serde_json::json!({
            "coin": full_coin,
            "best_bid": bid.to_string(),
            "best_ask": ask.to_string(),
            "spread": spread.to_string(),
            "spread_bps": spread_bps.to_string(),
            "midpoint": midpoint.to_string(),
        });
        print_json(&info)?;
    } else {
        let mut table = new_table(&["Metric", "Value"]);
        table.add_row(vec!["Coin", &full_coin]);
        table.add_row(vec!["Best Bid", &bid.to_string()]);
        table.add_row(vec!["Best Ask", &ask.to_string()]);
        table.add_row(vec!["Spread", &spread.to_string()]);
        table.add_row(vec!["Spread (bps)", &spread_bps.to_string()]);
        table.add_row(vec!["Midpoint", &midpoint.to_string()]);
        print_table(table);
    }
    Ok(())
}
