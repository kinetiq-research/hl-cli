use hl_rs::{BatchOrder, LimitOrderType, OrderType, OrderWire, Tif};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::cli::{Network, Side};
use crate::client::{exchange_client, resolve_asset_index, send_info_request, resolve_coin_and_dex};
use crate::confirm::confirm_action;
use crate::error::CliError;
use crate::output::print_json;

/// Fetch the best bid/ask from L2 book and return (best_bid, best_ask).
async fn get_best_prices(
    network: &Network,
    coin: &str,
    dex: Option<&str>,
) -> Result<(Decimal, Decimal), CliError> {
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

    let bids = levels[0]
        .as_array()
        .ok_or_else(|| CliError::Api("No bids".into()))?;
    let asks = levels[1]
        .as_array()
        .ok_or_else(|| CliError::Api("No asks".into()))?;

    let best_bid = bids
        .first()
        .and_then(|b| b.get("px"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .ok_or_else(|| CliError::Api("No bid price available".into()))?;

    let best_ask = asks
        .first()
        .and_then(|a| a.get("px"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .ok_or_else(|| CliError::Api("No ask price available".into()))?;

    Ok((best_bid, best_ask))
}

/// Fetch size decimals from metadata so we can round the size properly.
async fn get_size_decimals(
    network: &Network,
    dex: Option<&str>,
    coin: &str,
) -> Result<u32, CliError> {
    let (dex_name, full_coin) = resolve_coin_and_dex(dex, coin);

    if let Some(d) = &dex_name {
        let request = serde_json::json!({ "type": "metaAndAssetCtxs", "dex": d });
        let response: serde_json::Value = send_info_request(network, &request).await?;
        if let Some(universe) = response
            .as_array()
            .and_then(|a| a.first())
            .and_then(|m| m.get("universe"))
            .and_then(|u| u.as_array())
        {
            for asset in universe {
                let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.eq_ignore_ascii_case(&full_coin) {
                    return Ok(asset
                        .get("szDecimals")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(4) as u32);
                }
            }
        }
        return Ok(4); // default
    }

    let request = serde_json::json!({ "type": "meta" });
    let response: serde_json::Value = send_info_request(network, &request).await?;
    if let Some(universe) = response.get("universe").and_then(|v| v.as_array()) {
        for asset in universe {
            let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.eq_ignore_ascii_case(coin) {
                return Ok(asset
                    .get("szDecimals")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(4) as u32);
            }
        }
    }
    Ok(4) // default
}

fn round_size(size: Decimal, decimals: u32) -> Decimal {
    size.round_dp(decimals)
}

pub async fn run(
    network: &Network,
    json: bool,
    yes: bool,
    coin: &str,
    side: &Side,
    size: Option<Decimal>,
    amount: Option<Decimal>,
    slippage: Decimal,
    reduce_only: bool,
    dex: Option<&str>,
) -> Result<(), CliError> {
    if size.is_none() && amount.is_none() {
        return Err(CliError::InvalidArg(
            "Either --size or --amount is required".into(),
        ));
    }

    let (best_bid, best_ask) = get_best_prices(network, coin, dex).await?;
    let is_buy = matches!(side, Side::Buy);
    let slippage_mult = if is_buy {
        dec!(1) + slippage / dec!(100)
    } else {
        dec!(1) - slippage / dec!(100)
    };

    let ref_price = if is_buy { best_ask } else { best_bid };
    let limit_price = (ref_price * slippage_mult).round_dp(6);

    let order_size = if let Some(s) = size {
        s
    } else {
        let usd = amount.unwrap();
        let sz_decimals = get_size_decimals(network, dex, coin).await?;
        round_size(usd / ref_price, sz_decimals)
    };

    if order_size <= dec!(0) {
        return Err(CliError::InvalidArg("Calculated size is zero or negative".into()));
    }

    let side_str = if is_buy { "BUY" } else { "SELL" };
    confirm_action(
        &format!(
            "Market {side_str} {order_size} {coin} @ ~{limit_price} (ref: {ref_price}, slippage: {slippage}%)"
        ),
        yes,
    )?;

    let asset_index = resolve_asset_index(network, dex, coin).await?;

    let order = OrderWire {
        a: asset_index,
        b: is_buy,
        p: limit_price,
        s: order_size,
        r: reduce_only,
        t: OrderType::Limit(LimitOrderType { tif: Tif::Ioc }),
        c: None,
    };

    let batch = BatchOrder::new(vec![order]);
    let client = exchange_client(network)?;
    let response = client.send_action(batch).await?;

    if json {
        print_json(&response)?;
    } else {
        println!("Market order submitted: {side_str} {order_size} {coin} @ {limit_price} (IOC)");
    }
    Ok(())
}
