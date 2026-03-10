mod batch;
mod cancel;
mod cancel_all;
mod funding;
mod init;
mod install_skill;
mod leverage;
mod market;
mod market_order;
mod modify;
mod oi;
mod orders;
mod place;
mod pnl;
mod search;
pub mod shell;
mod spread;
pub mod state;
mod status;
mod transfer;
mod upgrade;

use crate::cli::{Cli, Command, LeverageAction, OrderAction};
use crate::error::CliError;

/// Dispatch a CLI invocation, consuming the Cli struct.
pub async fn dispatch(cli: Cli) -> Result<(), CliError> {
    let json = cli.json;
    let network = &cli.network;
    let dex = cli.dex.as_deref();
    let yes = cli.yes;

    match cli.command {
        Command::State => state::run_state(network, json, dex).await,
        Command::Positions => state::run_positions(network, json, dex).await,
        Command::Balance => state::run_balance(network, json).await,
        Command::Orders => orders::run_orders(network, json).await,
        Command::OrderStatus { oid } => orders::run_order_status(network, json, oid).await,
        Command::Fills => orders::run_fills(network, json).await,
        Command::HistoricalOrders => orders::run_historical_orders(network, json).await,
        Command::Book { coin, levels } => {
            market::run_book(network, json, &coin, dex, levels).await
        }
        Command::Mids => market::run_mids(network, json, dex).await,
        Command::Meta => market::run_meta(network, json, dex).await,
        Command::SpotMeta => market::run_spot_meta(network, json).await,
        Command::Dexes => market::run_dexes(network, json).await,
        Command::Trades { coin } => market::run_trades(network, json, &coin, dex).await,
        Command::Funding { coin, start, end } => {
            funding::run_funding(network, json, &coin, start, end, dex).await
        }
        Command::UserFunding { start, end } => {
            funding::run_user_funding(network, json, start, end).await
        }
        Command::Candles {
            coin,
            interval,
            start,
            end,
        } => market::run_candles(network, json, &coin, &interval, start, end, dex).await,
        Command::Order { action } => match action {
            OrderAction::Place { .. } => place::run(network, json, yes, action, dex).await,
            OrderAction::Market {
                ref coin,
                ref side,
                size,
                amount,
                slippage,
                reduce_only,
            } => {
                market_order::run(
                    network,
                    json,
                    yes,
                    coin,
                    side,
                    size,
                    amount,
                    slippage,
                    reduce_only,
                    dex,
                )
                .await
            }
            OrderAction::Batch { ref orders_json } => {
                batch::run(network, json, yes, orders_json, dex).await
            }
            OrderAction::Cancel { .. } | OrderAction::CancelByCloid { .. } => {
                cancel::run_cancel(network, json, yes, action, dex).await
            }
            OrderAction::CancelAll { ref coin } => {
                cancel_all::run(network, json, yes, coin.as_deref(), dex).await
            }
            OrderAction::Modify { .. } => modify::run(network, json, yes, action, dex).await,
        },
        Command::Leverage { action } => match action {
            LeverageAction::Set { .. } => leverage::run(network, json, yes, action, dex).await,
        },
        Command::Transfer { to, amount } => {
            transfer::run_transfer(network, json, yes, &to, amount).await
        }
        Command::Withdraw { to, amount } => {
            transfer::run_withdraw(network, json, yes, &to, amount).await
        }
        Command::InstallSkill { project, force } => install_skill::run(project, force),
        Command::Init {
            private_key,
            address,
            network: net,
            force,
            no_skill,
        } => init::run(private_key, address, net, force, no_skill),

        // New commands
        Command::Status => status::run(network, json).await,
        Command::Spread { coin } => spread::run(network, json, &coin, dex).await,
        Command::Pnl => pnl::run(network, json, dex).await,
        Command::Oi { coin } => oi::run(network, json, coin.as_deref(), dex).await,
        Command::Search { query } => search::run(network, json, &query, dex).await,
        Command::Shell => shell::run().await,
        Command::Upgrade => upgrade::run(),
    }
}

/// Dispatch without consuming Cli (for watch mode). Borrows Cli.
pub async fn dispatch_once(cli: &Cli) -> Result<(), CliError> {
    let json = cli.json;
    let network = &cli.network;
    let dex = cli.dex.as_deref();

    // Watch mode only supports read commands
    match &cli.command {
        Command::State => state::run_state(network, json, dex).await,
        Command::Positions => state::run_positions(network, json, dex).await,
        Command::Balance => state::run_balance(network, json).await,
        Command::Orders => orders::run_orders(network, json).await,
        Command::Fills => orders::run_fills(network, json).await,
        Command::HistoricalOrders => orders::run_historical_orders(network, json).await,
        Command::Book { coin, levels } => {
            market::run_book(network, json, coin, dex, *levels).await
        }
        Command::Mids => market::run_mids(network, json, dex).await,
        Command::Meta => market::run_meta(network, json, dex).await,
        Command::SpotMeta => market::run_spot_meta(network, json).await,
        Command::Dexes => market::run_dexes(network, json).await,
        Command::Trades { coin } => market::run_trades(network, json, coin, dex).await,
        Command::Spread { coin } => spread::run(network, json, coin, dex).await,
        Command::Pnl => pnl::run(network, json, dex).await,
        Command::Oi { coin } => oi::run(network, json, coin.as_deref(), dex).await,
        Command::Status => status::run(network, json).await,
        _ => Err(CliError::InvalidArg(
            "Watch mode only supports read commands".into(),
        )),
    }
}
