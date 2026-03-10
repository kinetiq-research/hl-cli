use std::str::FromStr;

use alloy::primitives::Address;
use hl_rs::{UsdSend, Withdraw};
use rust_decimal::Decimal;

use crate::cli::Network;
use crate::client::exchange_client;
use crate::confirm::confirm_action;
use crate::error::CliError;
use crate::output::print_json;

pub async fn run_transfer(
    network: &Network,
    json: bool,
    yes: bool,
    to: &str,
    amount: Decimal,
) -> Result<(), CliError> {
    let destination = Address::from_str(to)
        .map_err(|e| CliError::InvalidArg(format!("Invalid address: {e}")))?;

    confirm_action(
        &format!("Transfer {amount} USDC to {to}"),
        yes,
    )?;

    let client = exchange_client(network)?;
    let action = UsdSend::new(destination, amount);
    let response = client.send_action(action).await?;

    if json {
        print_json(&response)?;
    } else {
        println!("Transferred {amount} USDC to {to}");
        print_json(&response)?;
    }
    Ok(())
}

pub async fn run_withdraw(
    network: &Network,
    json: bool,
    yes: bool,
    to: &str,
    amount: Decimal,
) -> Result<(), CliError> {
    let destination = Address::from_str(to)
        .map_err(|e| CliError::InvalidArg(format!("Invalid address: {e}")))?;

    confirm_action(
        &format!("Withdraw {amount} USDC to L1 address {to}"),
        yes,
    )?;

    let client = exchange_client(network)?;
    let action = Withdraw::new(destination, amount);
    let response = client.send_action(action).await?;

    if json {
        print_json(&response)?;
    } else {
        println!("Withdrawal of {amount} USDC to {to} submitted");
        print_json(&response)?;
    }
    Ok(())
}
