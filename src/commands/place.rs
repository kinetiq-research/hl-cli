use hl_rs::{
    BatchOrder, LimitOrderType, OrderType, OrderWire, Tif, TpSl, TriggerOrderType,
};

use crate::cli::{Network, OrderAction, Side, TifArg, TriggerType};
use crate::client::{exchange_client, resolve_asset_index};
use crate::error::CliError;
use crate::output::print_json;

pub async fn run(network: &Network, json: bool, action: OrderAction) -> Result<(), CliError> {
    let OrderAction::Place {
        coin,
        side,
        size,
        price,
        tif,
        reduce_only,
        cloid,
        trigger_price,
        trigger_type,
        trigger_is_market,
    } = action
    else {
        unreachable!()
    };

    let asset_index = resolve_asset_index(network, &coin).await?;
    let is_buy = matches!(side, Side::Buy);

    let order_type = if let Some(tp) = trigger_price {
        let tpsl = match trigger_type {
            Some(TriggerType::Tp) => TpSl::Tp,
            Some(TriggerType::Sl) => TpSl::Sl,
            None => {
                return Err(CliError::InvalidArg(
                    "--trigger-type (tp or sl) is required when --trigger-price is set".into(),
                ))
            }
        };
        OrderType::Trigger(TriggerOrderType {
            trigger_px: tp,
            is_market: trigger_is_market,
            tpsl,
        })
    } else {
        let tif_val = match tif {
            TifArg::Gtc => Tif::Gtc,
            TifArg::Ioc => Tif::Ioc,
            TifArg::Alo => Tif::Alo,
        };
        OrderType::Limit(LimitOrderType { tif: tif_val })
    };

    let order = OrderWire {
        a: asset_index,
        b: is_buy,
        p: price,
        s: size,
        r: reduce_only,
        t: order_type,
        c: cloid,
    };

    let batch = BatchOrder::new(vec![order]);

    let client = exchange_client(network)?;
    let response = client.send_action(batch).await?;

    if json {
        print_json(&response)?;
    } else {
        let side_str = if is_buy { "BUY" } else { "SELL" };
        println!("Order submitted: {side_str} {size} {coin} @ {price}");
        println!("Response: {:?}", response);
    }
    Ok(())
}
