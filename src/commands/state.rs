use serde::Deserialize;

use crate::cli::Network;
use crate::client::{send_info_request, user_address};
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

#[derive(Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStateResponse {
    pub asset_positions: Vec<AssetPosition>,
    pub margin_summary: MarginSummary,
    pub withdrawable: String,
    #[serde(default)]
    pub cross_margin_summary: Option<MarginSummary>,
}

#[derive(Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
}

#[derive(Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
    pub position: PositionData,
    #[serde(rename = "type")]
    pub position_type: Option<String>,
}

#[derive(Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    pub coin: String,
    pub szi: String,
    pub entry_px: Option<String>,
    pub position_value: Option<String>,
    pub unrealized_pnl: Option<String>,
    pub return_on_equity: Option<String>,
    pub liquidation_px: Option<String>,
    pub margin_used: Option<String>,
    pub leverage: Option<Leverage>,
    pub cum_funding: Option<CumFunding>,
}

#[derive(Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    #[serde(rename = "type")]
    pub leverage_type: String,
    pub value: u32,
    pub raw_usd: Option<String>,
}

#[derive(Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CumFunding {
    pub all_time: String,
    pub since_open: String,
    pub since_change: String,
}

pub async fn run_state(
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

    if json {
        return print_json(&response);
    }

    let ms = &response.margin_summary;
    let mut table = new_table(&["Metric", "Value"]);
    table.add_row(vec!["Account Value", &ms.account_value]);
    table.add_row(vec!["Total Margin Used", &ms.total_margin_used]);
    table.add_row(vec!["Total Notional Pos", &ms.total_ntl_pos]);
    table.add_row(vec!["Total Raw USD", &ms.total_raw_usd]);
    table.add_row(vec!["Withdrawable", &response.withdrawable]);
    print_table(table);
    Ok(())
}

pub async fn run_positions(
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

    // Filter to non-zero positions
    let positions: Vec<&AssetPosition> = response
        .asset_positions
        .iter()
        .filter(|p| p.position.szi != "0.0" && p.position.szi != "0")
        .collect();

    if json {
        return print_json(&positions);
    }

    if positions.is_empty() {
        println!("No open positions");
        return Ok(());
    }

    let mut table = new_table(&[
        "Coin",
        "Size",
        "Entry Px",
        "Pos Value",
        "Unrealized PnL",
        "ROE",
        "Leverage",
        "Liq Px",
        "Margin Used",
    ]);

    for ap in &positions {
        let p = &ap.position;
        let lev = p
            .leverage
            .as_ref()
            .map(|l| format!("{}x {}", l.value, l.leverage_type))
            .unwrap_or_default();

        table.add_row(vec![
            &p.coin,
            &p.szi,
            p.entry_px.as_deref().unwrap_or("-"),
            p.position_value.as_deref().unwrap_or("-"),
            p.unrealized_pnl.as_deref().unwrap_or("-"),
            p.return_on_equity.as_deref().unwrap_or("-"),
            &lev,
            p.liquidation_px.as_deref().unwrap_or("-"),
            p.margin_used.as_deref().unwrap_or("-"),
        ]);
    }

    print_table(table);
    Ok(())
}

pub async fn run_balance(network: &Network, json: bool) -> Result<(), CliError> {
    let address = user_address()?;
    let request = serde_json::json!({
        "type": "spotClearinghouseState",
        "user": address.to_string(),
    });

    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    // Try to parse balances from the response
    if let Some(balances) = response.get("balances").and_then(|b| b.as_array()) {
        if balances.is_empty() {
            println!("No token balances");
            return Ok(());
        }

        let mut table = new_table(&["Token", "Hold", "Total"]);
        for bal in balances {
            let coin = bal.get("coin").and_then(|v| v.as_str()).unwrap_or("-");
            let hold = bal.get("hold").and_then(|v| v.as_str()).unwrap_or("0");
            let total = bal.get("total").and_then(|v| v.as_str()).unwrap_or("0");
            table.add_row(vec![coin, hold, total]);
        }
        print_table(table);
    } else {
        print_json(&response)?;
    }

    Ok(())
}
