use crate::cli::Network;
use crate::client::send_info_request;
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run_book(network: &Network, json: bool, coin: &str) -> Result<(), CliError> {
    let request = serde_json::json!({
        "type": "l2Book",
        "coin": coin,
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let levels = response.get("levels").and_then(|v| v.as_array());
    if let Some(levels) = levels {
        // levels[0] = bids, levels[1] = asks
        if levels.len() >= 2 {
            let bids = levels[0].as_array();
            let asks = levels[1].as_array();

            println!("=== {coin} Order Book ===\n");

            // Print asks (reversed so best ask at bottom)
            if let Some(asks) = asks {
                let mut table = new_table(&["Ask Price", "Ask Size"]);
                let display_asks: Vec<&serde_json::Value> =
                    asks.iter().take(10).rev().collect();
                for ask in display_asks {
                    let px = ask.get("px").and_then(|v| v.as_str()).unwrap_or("-");
                    let sz = ask.get("sz").and_then(|v| v.as_str()).unwrap_or("-");
                    table.add_row(vec![px, sz]);
                }
                print_table(table);
            }

            println!("  ---");

            // Print bids
            if let Some(bids) = bids {
                let mut table = new_table(&["Bid Price", "Bid Size"]);
                for bid in bids.iter().take(10) {
                    let px = bid.get("px").and_then(|v| v.as_str()).unwrap_or("-");
                    let sz = bid.get("sz").and_then(|v| v.as_str()).unwrap_or("-");
                    table.add_row(vec![px, sz]);
                }
                print_table(table);
            }
        }
    } else {
        print_json(&response)?;
    }

    Ok(())
}

pub async fn run_mids(network: &Network, json: bool) -> Result<(), CliError> {
    let request = serde_json::json!({ "type": "allMids" });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    if let Some(mids) = response.as_object() {
        let mut table = new_table(&["Coin", "Mid Price"]);
        let mut entries: Vec<(&String, &serde_json::Value)> = mids.iter().collect();
        entries.sort_by_key(|(k, _)| k.to_string());
        for (coin, price) in entries {
            let px_str = price.as_str().map(String::from).unwrap_or_else(|| price.to_string());
            table.add_row(vec![coin.as_str(), &px_str]);
        }
        print_table(table);
    } else {
        print_json(&response)?;
    }

    Ok(())
}

pub async fn run_meta(network: &Network, json: bool) -> Result<(), CliError> {
    let request = serde_json::json!({ "type": "meta" });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    if let Some(universe) = response.get("universe").and_then(|v| v.as_array()) {
        let mut table = new_table(&["Index", "Name", "Max Leverage", "Sz Decimals"]);
        for (i, asset) in universe.iter().enumerate() {
            let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("-");
            let max_lev = asset
                .get("maxLeverage")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let sz_dec = asset
                .get("szDecimals")
                .map(|v| v.to_string())
                .unwrap_or_default();
            table.add_row(vec![&i.to_string(), name, &max_lev, &sz_dec]);
        }
        print_table(table);
    } else {
        print_json(&response)?;
    }

    Ok(())
}

pub async fn run_spot_meta(network: &Network, json: bool) -> Result<(), CliError> {
    let request = serde_json::json!({ "type": "spotMeta" });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    // Spot meta has tokens and universe arrays
    if let Some(tokens) = response.get("tokens").and_then(|v| v.as_array()) {
        let mut table = new_table(&["Index", "Name", "Token ID", "Decimals"]);
        for token in tokens {
            let index = token.get("index").map(|v| v.to_string()).unwrap_or_default();
            let name = token.get("name").and_then(|v| v.as_str()).unwrap_or("-");
            let token_id = token
                .get("tokenId")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let weis = token
                .get("weiDecimals")
                .map(|v| v.to_string())
                .unwrap_or_default();
            table.add_row(vec![&index, name, token_id, &weis]);
        }
        print_table(table);
    } else {
        print_json(&response)?;
    }

    Ok(())
}

pub async fn run_candles(
    network: &Network,
    json: bool,
    coin: &str,
    interval: &str,
    start: u64,
    end: u64,
) -> Result<(), CliError> {
    let request = serde_json::json!({
        "type": "candleSnapshot",
        "req": {
            "coin": coin,
            "interval": interval,
            "startTime": start,
            "endTime": end,
        }
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let candles = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if candles.is_empty() {
        println!("No candle data");
        return Ok(());
    }

    let mut table = new_table(&["Time", "Open", "High", "Low", "Close", "Volume"]);
    for candle in candles {
        let t = candle.get("t").map(|v| v.to_string()).unwrap_or_default();
        let o = candle.get("o").and_then(|v| v.as_str()).unwrap_or("-");
        let h = candle.get("h").and_then(|v| v.as_str()).unwrap_or("-");
        let l = candle.get("l").and_then(|v| v.as_str()).unwrap_or("-");
        let c = candle.get("c").and_then(|v| v.as_str()).unwrap_or("-");
        let v = candle.get("v").and_then(|v| v.as_str()).unwrap_or("-");
        table.add_row(vec![&t, o, h, l, c, v]);
    }
    print_table(table);
    Ok(())
}

pub async fn run_trades(network: &Network, json: bool, coin: &str) -> Result<(), CliError> {
    let request = serde_json::json!({
        "type": "recentTrades",
        "coin": coin,
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let trades = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if trades.is_empty() {
        println!("No recent trades");
        return Ok(());
    }

    let mut table = new_table(&["Coin", "Side", "Price", "Size", "Time", "TID"]);
    for trade in trades.iter().take(50) {
        let coin = trade.get("coin").and_then(|v| v.as_str()).unwrap_or("-");
        let side = trade.get("side").and_then(|v| v.as_str()).unwrap_or("-");
        let px = trade.get("px").and_then(|v| v.as_str()).unwrap_or("-");
        let sz = trade.get("sz").and_then(|v| v.as_str()).unwrap_or("-");
        let time = trade.get("time").map(|v| v.to_string()).unwrap_or_default();
        let tid = trade.get("tid").map(|v| v.to_string()).unwrap_or_default();
        table.add_row(vec![coin, side, px, sz, &time, &tid]);
    }
    print_table(table);
    Ok(())
}
