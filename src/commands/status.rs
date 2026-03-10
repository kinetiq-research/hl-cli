use crate::cli::Network;
use crate::client::send_info_request;
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run(network: &Network, json: bool) -> Result<(), CliError> {
    let start = std::time::Instant::now();

    // Try fetching meta as a health check
    let request = serde_json::json!({ "type": "meta" });
    let result: Result<serde_json::Value, _> = send_info_request(network, &request).await;
    let latency_ms = start.elapsed().as_millis();

    let (status, asset_count) = match &result {
        Ok(resp) => {
            let count = resp
                .get("universe")
                .and_then(|u| u.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            ("OK", count)
        }
        Err(e) => {
            if !json {
                eprintln!("API error: {e}");
            }
            ("ERROR", 0)
        }
    };

    let network_str = format!("{:?}", network).to_lowercase();

    if json {
        let info = serde_json::json!({
            "status": status,
            "network": network_str,
            "latency_ms": latency_ms,
            "assets": asset_count,
        });
        print_json(&info)?;
    } else {
        let mut table = new_table(&["Check", "Result"]);
        table.add_row(vec!["Status", status]);
        table.add_row(vec!["Network", &network_str]);
        table.add_row(vec!["Latency", &format!("{latency_ms}ms")]);
        table.add_row(vec!["Assets", &asset_count.to_string()]);
        print_table(table);
    }
    Ok(())
}
