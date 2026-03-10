use crate::cli::Network;
use crate::client::{resolve_coin_and_dex, send_info_request};
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run(
    network: &Network,
    json: bool,
    coin: Option<&str>,
    dex: Option<&str>,
) -> Result<(), CliError> {
    // Fetch metaAndAssetCtxs which includes OI data
    let request = if let Some(d) = dex {
        serde_json::json!({ "type": "metaAndAssetCtxs", "dex": d })
    } else {
        serde_json::json!({ "type": "metaAndAssetCtxs" })
    };

    let response: serde_json::Value = send_info_request(network, &request).await?;

    // Response is [meta, assetCtxs] tuple
    let arr = response
        .as_array()
        .ok_or_else(|| CliError::Api("Unexpected response format".into()))?;

    if arr.len() < 2 {
        return Err(CliError::Api("Incomplete metaAndAssetCtxs response".into()));
    }

    let universe = arr[0]
        .get("universe")
        .and_then(|u| u.as_array())
        .ok_or_else(|| CliError::Api("No universe in meta".into()))?;

    let ctxs = arr[1]
        .as_array()
        .ok_or_else(|| CliError::Api("No asset contexts".into()))?;

    // Build list of (name, open_interest, funding_rate, mark_px)
    let mut entries: Vec<(String, String, String, String)> = Vec::new();

    for (i, asset) in universe.iter().enumerate() {
        let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("-");
        if let Some(ctx) = ctxs.get(i) {
            let oi = ctx
                .get("openInterest")
                .or_else(|| ctx.get("oi"))
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            let funding = ctx
                .get("funding")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let mark = ctx
                .get("markPx")
                .and_then(|v| v.as_str())
                .unwrap_or("-");

            // Filter by coin if specified
            if let Some(filter) = coin {
                let (_, full_name) = resolve_coin_and_dex(dex, filter);
                if !name.eq_ignore_ascii_case(&full_name)
                    && !name.eq_ignore_ascii_case(filter)
                {
                    continue;
                }
            }

            entries.push((
                name.to_string(),
                oi.to_string(),
                funding.to_string(),
                mark.to_string(),
            ));
        }
    }

    if json {
        let json_entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|(name, oi, funding, mark)| {
                serde_json::json!({
                    "coin": name,
                    "open_interest": oi,
                    "funding": funding,
                    "mark_px": mark,
                })
            })
            .collect();
        return print_json(&json_entries);
    }

    if entries.is_empty() {
        println!("No data found");
        return Ok(());
    }

    let mut table = new_table(&["Coin", "Open Interest", "Funding Rate", "Mark Price"]);
    for (name, oi, funding, mark) in &entries {
        table.add_row(vec![name.as_str(), oi.as_str(), funding.as_str(), mark.as_str()]);
    }
    print_table(table);
    Ok(())
}
