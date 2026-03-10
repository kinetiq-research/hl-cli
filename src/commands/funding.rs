use crate::cli::Network;
use crate::client::{send_info_request, user_address};
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run_funding(
    network: &Network,
    json: bool,
    coin: &str,
    start: u64,
    end: Option<u64>,
) -> Result<(), CliError> {
    let mut request = serde_json::json!({
        "type": "fundingHistory",
        "coin": coin,
        "startTime": start,
    });
    if let Some(end_time) = end {
        request["endTime"] = serde_json::json!(end_time);
    }
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let entries = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if entries.is_empty() {
        println!("No funding history");
        return Ok(());
    }

    let mut table = new_table(&["Coin", "Funding Rate", "Premium", "Time"]);
    for entry in entries {
        let coin = entry.get("coin").and_then(|v| v.as_str()).unwrap_or("-");
        let rate = entry
            .get("fundingRate")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let premium = entry.get("premium").and_then(|v| v.as_str()).unwrap_or("-");
        let time = entry.get("time").map(|v| v.to_string()).unwrap_or_default();
        table.add_row(vec![coin, rate, premium, &time]);
    }
    print_table(table);
    Ok(())
}

pub async fn run_user_funding(
    network: &Network,
    json: bool,
    start: u64,
    end: Option<u64>,
) -> Result<(), CliError> {
    let address = user_address()?;
    let mut request = serde_json::json!({
        "type": "userFunding",
        "user": format!("{:?}", address),
        "startTime": start,
    });
    if let Some(end_time) = end {
        request["endTime"] = serde_json::json!(end_time);
    }
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let entries = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if entries.is_empty() {
        println!("No user funding history");
        return Ok(());
    }

    let mut table = new_table(&["Time", "Coin", "USD Delta", "Type"]);
    for entry in entries {
        let time = entry.get("time").map(|v| v.to_string()).unwrap_or_default();
        let delta = entry
            .get("delta")
            .and_then(|d| d.get("coin"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let usd_delta = entry
            .get("delta")
            .and_then(|d| d.get("usdc"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let funding_type = entry
            .get("delta")
            .and_then(|d| d.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        table.add_row(vec![&time, delta, usd_delta, funding_type]);
    }
    print_table(table);
    Ok(())
}
