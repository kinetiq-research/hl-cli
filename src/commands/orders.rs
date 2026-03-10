use crate::cli::Network;
use crate::client::{send_info_request, user_address};
use crate::error::CliError;
use crate::output::{new_table, print_json, print_table};

pub async fn run_orders(network: &Network, json: bool) -> Result<(), CliError> {
    let address = user_address()?;
    let request = serde_json::json!({
        "type": "openOrders",
        "user": format!("{:?}", address),
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let orders = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if orders.is_empty() {
        println!("No open orders");
        return Ok(());
    }

    let mut table = new_table(&["Coin", "Side", "Price", "Size", "Order ID", "Cloid", "Timestamp"]);
    for order in orders {
        let coin = order.get("coin").and_then(|v| v.as_str()).unwrap_or("-");
        let side = order.get("side").and_then(|v| v.as_str()).unwrap_or("-");
        let limit_px = order.get("limitPx").and_then(|v| v.as_str()).unwrap_or("-");
        let sz = order.get("sz").and_then(|v| v.as_str()).unwrap_or("-");
        let oid = order
            .get("oid")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let cloid = order.get("cloid").and_then(|v| v.as_str()).unwrap_or("-");
        let timestamp = order
            .get("timestamp")
            .map(|v| v.to_string())
            .unwrap_or_default();

        table.add_row(vec![coin, side, limit_px, sz, &oid, cloid, &timestamp]);
    }
    print_table(table);
    Ok(())
}

pub async fn run_order_status(
    network: &Network,
    json: bool,
    oid: u64,
) -> Result<(), CliError> {
    let address = user_address()?;
    let request = serde_json::json!({
        "type": "orderStatus",
        "user": format!("{:?}", address),
        "oid": oid,
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    // Pretty-print the order status
    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    println!("Order {oid}: {status}");

    if let Some(order) = response.get("order") {
        let coin = order.get("coin").and_then(|v| v.as_str()).unwrap_or("-");
        let side = order.get("side").and_then(|v| v.as_str()).unwrap_or("-");
        let limit_px = order.get("limitPx").and_then(|v| v.as_str()).unwrap_or("-");
        let sz = order.get("sz").and_then(|v| v.as_str()).unwrap_or("-");
        let orig_sz = order.get("origSz").and_then(|v| v.as_str()).unwrap_or("-");
        println!("  Coin: {coin}  Side: {side}  Price: {limit_px}  Size: {sz}  Orig Size: {orig_sz}");
    }

    Ok(())
}

pub async fn run_fills(network: &Network, json: bool) -> Result<(), CliError> {
    let address = user_address()?;
    let request = serde_json::json!({
        "type": "userFills",
        "user": format!("{:?}", address),
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let fills = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if fills.is_empty() {
        println!("No recent fills");
        return Ok(());
    }

    let mut table = new_table(&[
        "Coin", "Side", "Price", "Size", "Fee", "Realized PnL", "Time", "OID",
    ]);
    for fill in fills {
        let coin = fill.get("coin").and_then(|v| v.as_str()).unwrap_or("-");
        let side = fill.get("side").and_then(|v| v.as_str()).unwrap_or("-");
        let px = fill.get("px").and_then(|v| v.as_str()).unwrap_or("-");
        let sz = fill.get("sz").and_then(|v| v.as_str()).unwrap_or("-");
        let fee = fill.get("fee").and_then(|v| v.as_str()).unwrap_or("-");
        let pnl = fill
            .get("closedPnl")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let time = fill.get("time").map(|v| v.to_string()).unwrap_or_default();
        let oid = fill.get("oid").map(|v| v.to_string()).unwrap_or_default();

        table.add_row(vec![coin, side, px, sz, fee, pnl, &time, &oid]);
    }
    print_table(table);
    Ok(())
}

pub async fn run_historical_orders(network: &Network, json: bool) -> Result<(), CliError> {
    let address = user_address()?;
    let request = serde_json::json!({
        "type": "historicalOrders",
        "user": format!("{:?}", address),
    });
    let response: serde_json::Value = send_info_request(network, &request).await?;

    if json {
        return print_json(&response);
    }

    let orders = response.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if orders.is_empty() {
        println!("No historical orders");
        return Ok(());
    }

    let mut table = new_table(&[
        "Coin", "Side", "Price", "Size", "Status", "Order ID", "Timestamp",
    ]);
    for order in orders {
        let coin = order
            .get("order")
            .and_then(|o| o.get("coin"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let side = order
            .get("order")
            .and_then(|o| o.get("side"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let limit_px = order
            .get("order")
            .and_then(|o| o.get("limitPx"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let sz = order
            .get("order")
            .and_then(|o| o.get("sz"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let status = order
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let oid = order
            .get("order")
            .and_then(|o| o.get("oid"))
            .map(|v| v.to_string())
            .unwrap_or_default();
        let timestamp = order
            .get("order")
            .and_then(|o| o.get("timestamp"))
            .map(|v| v.to_string())
            .unwrap_or_default();

        table.add_row(vec![coin, side, limit_px, sz, status, &oid, &timestamp]);
    }
    print_table(table);
    Ok(())
}
