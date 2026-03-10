use hl_rs::{BatchModify, LimitOrderType, OrderType, OrderWire, Tif};

use crate::cli::{Network, OrderAction, Side, TifArg};
use crate::client::{exchange_client, resolve_asset_index};
use crate::error::CliError;
use crate::output::print_json;

pub async fn run(network: &Network, json: bool, action: OrderAction) -> Result<(), CliError> {
    let OrderAction::Modify {
        oid,
        coin,
        side,
        size,
        price,
        tif,
        reduce_only,
        cloid,
    } = action
    else {
        unreachable!()
    };

    let asset_index = resolve_asset_index(network, &coin).await?;
    let is_buy = matches!(side, Side::Buy);

    let tif_val = match tif {
        TifArg::Gtc => Tif::Gtc,
        TifArg::Ioc => Tif::Ioc,
        TifArg::Alo => Tif::Alo,
    };

    let order = OrderWire {
        a: asset_index,
        b: is_buy,
        p: price,
        s: size,
        r: reduce_only,
        t: OrderType::Limit(LimitOrderType { tif: tif_val }),
        c: cloid,
    };

    let modify = BatchModify::single(oid, order);

    let client = exchange_client(network)?;
    let response = client.send_action(modify).await?;

    if json {
        print_json(&response)?;
    } else {
        let side_str = if is_buy { "BUY" } else { "SELL" };
        println!("Order {oid} modified: {side_str} {size} {coin} @ {price}");
        println!("Response: {:?}", response);
    }
    Ok(())
}
