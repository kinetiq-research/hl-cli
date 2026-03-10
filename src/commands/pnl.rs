use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::cli::Network;
use crate::client::{send_info_request, user_address};
use crate::commands::state::UserStateResponse;
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run(
    network: &Network,
    json: bool,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let address = user_address()?;
    let mut request = serde_json::json!({
        "type": "clearinghouseState",
        "user": address.to_string(),
    });
    if let Some(d) = dex {
        request["dex"] = serde_json::json!(d);
    }
    let response: UserStateResponse = send_info_request(network, &request).await?;

    let positions: Vec<_> = response
        .asset_positions
        .iter()
        .filter(|p| p.position.szi != "0.0" && p.position.szi != "0")
        .collect();

    let mut total_unrealized = dec!(0);
    let mut total_notional = dec!(0);
    let mut per_coin: Vec<serde_json::Value> = Vec::new();

    for ap in &positions {
        let p = &ap.position;
        let upnl = p
            .unrealized_pnl
            .as_deref()
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(dec!(0));
        let notional = p
            .position_value
            .as_deref()
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(dec!(0));

        total_unrealized += upnl;
        total_notional += notional.abs();

        per_coin.push(serde_json::json!({
            "coin": p.coin,
            "size": p.szi,
            "entry_px": p.entry_px,
            "unrealized_pnl": upnl.to_string(),
            "roe": p.return_on_equity,
        }));
    }

    let account_value: Decimal = response
        .margin_summary
        .account_value
        .parse()
        .unwrap_or(dec!(0));

    if json {
        let summary = serde_json::json!({
            "account_value": account_value.to_string(),
            "total_unrealized_pnl": total_unrealized.to_string(),
            "total_notional": total_notional.to_string(),
            "positions": per_coin,
        });
        return print_json(&summary);
    }

    if positions.is_empty() {
        println!("No open positions");
        return Ok(());
    }

    println!("=== PnL Summary ===\n");

    let mut table = new_table(&["Coin", "Size", "Entry Px", "Unrealized PnL", "ROE"]);
    for ap in &positions {
        let p = &ap.position;
        let upnl = p.unrealized_pnl.as_deref().unwrap_or("0");
        table.add_row(vec![
            &p.coin,
            &p.szi,
            p.entry_px.as_deref().unwrap_or("-"),
            upnl,
            p.return_on_equity.as_deref().unwrap_or("-"),
        ]);
    }
    print_table(table);

    println!();
    let mut summary = new_table(&["Metric", "Value"]);
    summary.add_row(vec!["Account Value", &account_value.to_string()]);
    summary.add_row(vec!["Total Unrealized PnL", &total_unrealized.to_string()]);
    summary.add_row(vec!["Total Notional", &total_notional.to_string()]);
    print_table(summary);

    Ok(())
}
