mod cancel;
mod funding;
mod leverage;
mod market;
mod modify;
mod orders;
mod place;
mod state;
mod transfer;

use crate::cli::{Cli, Command, LeverageAction, OrderAction};
use crate::error::CliError;

pub async fn dispatch(cli: Cli) -> Result<(), CliError> {
    let json = cli.json;
    let network = &cli.network;

    match cli.command {
        Command::State => state::run_state(network, json).await,
        Command::Positions => state::run_positions(network, json).await,
        Command::Balance => state::run_balance(network, json).await,
        Command::Orders => orders::run_orders(network, json).await,
        Command::OrderStatus { oid } => orders::run_order_status(network, json, oid).await,
        Command::Fills => orders::run_fills(network, json).await,
        Command::HistoricalOrders => orders::run_historical_orders(network, json).await,
        Command::Book { coin } => market::run_book(network, json, &coin).await,
        Command::Mids => market::run_mids(network, json).await,
        Command::Meta => market::run_meta(network, json).await,
        Command::SpotMeta => market::run_spot_meta(network, json).await,
        Command::Trades { coin } => market::run_trades(network, json, &coin).await,
        Command::Funding { coin, start, end } => {
            funding::run_funding(network, json, &coin, start, end).await
        }
        Command::UserFunding { start, end } => {
            funding::run_user_funding(network, json, start, end).await
        }
        Command::Candles {
            coin,
            interval,
            start,
            end,
        } => market::run_candles(network, json, &coin, &interval, start, end).await,
        Command::Order { action } => match action {
            OrderAction::Place { .. } => place::run(network, json, action).await,
            OrderAction::Cancel { .. } | OrderAction::CancelByCloid { .. } => {
                cancel::run_cancel(network, json, action).await
            }
            OrderAction::Modify { .. } => modify::run(network, json, action).await,
        },
        Command::Leverage { action } => match action {
            LeverageAction::Set { .. } => leverage::run(network, json, action).await,
        },
        Command::Transfer { to, amount } => {
            transfer::run_transfer(network, json, &to, amount).await
        }
        Command::Withdraw { to, amount } => {
            transfer::run_withdraw(network, json, &to, amount).await
        }
    }
}
