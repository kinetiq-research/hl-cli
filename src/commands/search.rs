use crate::cli::Network;
use crate::client::send_info_request;
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run(
    network: &Network,
    json: bool,
    query: &str,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let query_lower = query.to_lowercase();
    let mut results: Vec<serde_json::Value> = Vec::new();

    // Search perp assets
    let perp_request = if let Some(d) = dex {
        serde_json::json!({ "type": "metaAndAssetCtxs", "dex": d })
    } else {
        serde_json::json!({ "type": "meta" })
    };

    let perp_response: serde_json::Value = send_info_request(network, &perp_request).await?;

    let universe = if dex.is_some() {
        perp_response
            .as_array()
            .and_then(|a| a.first())
            .and_then(|m| m.get("universe"))
            .and_then(|u| u.as_array())
    } else {
        perp_response.get("universe").and_then(|u| u.as_array())
    };

    if let Some(assets) = universe {
        for (i, asset) in assets.iter().enumerate() {
            let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.to_lowercase().contains(&query_lower) {
                let max_lev = asset
                    .get("maxLeverage")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let sz_dec = asset
                    .get("szDecimals")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                results.push(serde_json::json!({
                    "index": i,
                    "name": name,
                    "type": "perp",
                    "max_leverage": max_lev,
                    "sz_decimals": sz_dec,
                }));
            }
        }
    }

    // Also search spot if no dex filter
    if dex.is_none() {
        let spot_request = serde_json::json!({ "type": "spotMeta" });
        if let Ok(spot_response) = send_info_request::<_, serde_json::Value>(network, &spot_request).await {
            if let Some(tokens) = spot_response.get("tokens").and_then(|v| v.as_array()) {
                for token in tokens {
                    let name = token.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if name.to_lowercase().contains(&query_lower) {
                        let index = token.get("index").map(|v| v.to_string()).unwrap_or_default();
                        results.push(serde_json::json!({
                            "index": index,
                            "name": name,
                            "type": "spot",
                        }));
                    }
                }
            }
        }
    }

    if json {
        return print_json(&results);
    }

    if results.is_empty() {
        println!("No assets matching '{query}'");
        return Ok(());
    }

    let mut table = new_table(&["Index", "Name", "Type", "Max Leverage", "Sz Decimals"]);
    for r in &results {
        let index_str = r
            .get("index")
            .map(|v| match v.as_str() {
                Some(s) => s.to_string(),
                None => v.to_string(),
            })
            .unwrap_or_else(|| "-".to_string());
        table.add_row(vec![
            index_str.as_str(),
            r.get("name").and_then(|v| v.as_str()).unwrap_or("-"),
            r.get("type").and_then(|v| v.as_str()).unwrap_or("-"),
            r.get("max_leverage").and_then(|v| v.as_str()).unwrap_or("-"),
            r.get("sz_decimals").and_then(|v| v.as_str()).unwrap_or("-"),
        ]);
    }
    print_table(table);
    Ok(())
}
