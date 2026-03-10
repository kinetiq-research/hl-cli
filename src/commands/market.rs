use rust_decimal::Decimal;

use crate::cli::Network;
use crate::client::{resolve_coin_and_dex, resolve_spot_names, send_info_request, DexInfo};
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

/// Render a depth bar using Unicode block chars.
fn depth_bar(fraction: f64, max_width: usize) -> String {
    let filled = (fraction * max_width as f64).round() as usize;
    let filled = filled.min(max_width);
    "\u{2588}".repeat(filled)
}

pub async fn run_book(
    network: &Network,
    json: bool,
    coin: &str,
    dex: Option<&str>,
    levels: usize,
) -> Result<(), CliError> {
    let (_, full_coin) = resolve_coin_and_dex(dex, coin);
    let request = serde_json::json!({
        "type": "l2Book",
        "coin": full_coin,
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let book_levels = response.get("levels").and_then(|v| v.as_array());
    if let Some(book_levels) = book_levels {
        if book_levels.len() >= 2 {
            let bids = book_levels[0].as_array();
            let asks = book_levels[1].as_array();

            println!("=== {full_coin} Order Book ===\n");

            // Parse asks and compute depth bars
            if let Some(asks) = asks {
                let ask_entries: Vec<(String, String, Decimal)> = asks
                    .iter()
                    .take(levels)
                    .filter_map(|ask| {
                        let px = ask.get("px").and_then(|v| v.as_str())?.to_string();
                        let sz = ask.get("sz").and_then(|v| v.as_str())?.to_string();
                        let sz_dec: Decimal = sz.parse().ok()?;
                        Some((px, sz, sz_dec))
                    })
                    .collect();

                let max_ask_sz = ask_entries
                    .iter()
                    .map(|(_, _, s)| *s)
                    .max()
                    .unwrap_or(Decimal::ONE);

                let mut table = new_table(&["Ask Price", "Ask Size", "Depth"]);
                for (px, sz, sz_dec) in ask_entries.iter().rev() {
                    let frac = if max_ask_sz > Decimal::ZERO {
                        sz_dec.to_string().parse::<f64>().unwrap_or(0.0)
                            / max_ask_sz.to_string().parse::<f64>().unwrap_or(1.0)
                    } else {
                        0.0
                    };
                    table.add_row(vec![px.as_str(), sz.as_str(), &depth_bar(frac, 20)]);
                }
                print_table(table);
            }

            // Spread line
            if let (Some(bids), Some(asks)) = (
                book_levels[0].as_array(),
                book_levels[1].as_array(),
            ) {
                let best_bid = bids
                    .first()
                    .and_then(|b| b.get("px"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<Decimal>().ok());
                let best_ask = asks
                    .first()
                    .and_then(|a| a.get("px"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<Decimal>().ok());
                if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
                    let spread = ask - bid;
                    println!("  --- spread: {spread} ---");
                }
            }

            // Print bids with depth bars
            if let Some(bids) = bids {
                let bid_entries: Vec<(String, String, Decimal)> = bids
                    .iter()
                    .take(levels)
                    .filter_map(|bid| {
                        let px = bid.get("px").and_then(|v| v.as_str())?.to_string();
                        let sz = bid.get("sz").and_then(|v| v.as_str())?.to_string();
                        let sz_dec: Decimal = sz.parse().ok()?;
                        Some((px, sz, sz_dec))
                    })
                    .collect();

                let max_bid_sz = bid_entries
                    .iter()
                    .map(|(_, _, s)| *s)
                    .max()
                    .unwrap_or(Decimal::ONE);

                let mut table = new_table(&["Bid Price", "Bid Size", "Depth"]);
                for (px, sz, sz_dec) in &bid_entries {
                    let frac = if max_bid_sz > Decimal::ZERO {
                        sz_dec.to_string().parse::<f64>().unwrap_or(0.0)
                            / max_bid_sz.to_string().parse::<f64>().unwrap_or(1.0)
                    } else {
                        0.0
                    };
                    table.add_row(vec![px.as_str(), sz.as_str(), &depth_bar(frac, 20)]);
                }
                print_table(table);
            }
        }
    } else {
        print_json(&response)?;
    }

    Ok(())
}

pub async fn run_mids(
    network: &Network,
    json: bool,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let mut request = serde_json::json!({ "type": "allMids" });
    if let Some(d) = dex {
        request["dex"] = serde_json::json!(d);
    }
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    if let Some(mids) = response.as_object() {
        let spot_names = if dex.is_none() {
            resolve_spot_names(network).await.unwrap_or_default()
        } else {
            Default::default()
        };

        let mut table = new_table(&["Coin", "Mid Price"]);
        let mut entries: Vec<(&String, &serde_json::Value)> = mids.iter().collect();
        entries.sort_by_key(|(k, _)| k.to_string());
        for (coin, price) in entries {
            let display_name = spot_names.get(coin.as_str()).unwrap_or(coin);
            let px_str = price
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| price.to_string());
            table.add_row(vec![display_name.as_str(), &px_str]);
        }
        print_table(table);
    } else {
        print_json(&response)?;
    }

    Ok(())
}

pub async fn run_meta(
    network: &Network,
    json: bool,
    dex: Option<&str>,
) -> Result<(), CliError> {
    if let Some(d) = dex {
        return run_dex_meta(network, json, d).await;
    }

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

async fn run_dex_meta(network: &Network, json: bool, dex: &str) -> Result<(), CliError> {
    let request = serde_json::json!({ "type": "metaAndAssetCtxs", "dex": dex });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let meta = response
        .as_array()
        .and_then(|a| a.first())
        .and_then(|m| m.get("universe"))
        .and_then(|u| u.as_array());

    if let Some(universe) = meta {
        println!("=== {dex} dex assets ===\n");
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

pub async fn run_dexes(network: &Network, json: bool) -> Result<(), CliError> {
    let response: Vec<Option<DexInfo>> =
        send_info_request(network, &serde_json::json!({ "type": "perpDexs" })).await?;

    let dexes: Vec<&DexInfo> = response.iter().filter_map(|d| d.as_ref()).collect();

    if json {
        return print_json(&dexes);
    }

    if dexes.is_empty() {
        println!("No HIP-3 dexes found");
        return Ok(());
    }

    let mut table = new_table(&["Name", "Full Name", "Deployer"]);
    for dex in &dexes {
        table.add_row(vec![&dex.name, &dex.full_name, &dex.deployer]);
    }
    print_table(table);
    Ok(())
}

pub async fn run_spot_meta(network: &Network, json: bool) -> Result<(), CliError> {
    let request = serde_json::json!({ "type": "spotMeta" });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

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
    dex: Option<&str>,
) -> Result<(), CliError> {
    let (_, full_coin) = resolve_coin_and_dex(dex, coin);
    let request = serde_json::json!({
        "type": "candleSnapshot",
        "req": {
            "coin": full_coin,
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

pub async fn run_trades(
    network: &Network,
    json: bool,
    coin: &str,
    dex: Option<&str>,
) -> Result<(), CliError> {
    let (_, full_coin) = resolve_coin_and_dex(dex, coin);
    let request = serde_json::json!({
        "type": "recentTrades",
        "coin": full_coin,
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
