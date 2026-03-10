use std::str::FromStr;

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use hl_rs::{BaseUrl, ExchangeClient};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::cli::Network;
use crate::error::CliError;

fn base_url(network: &Network) -> BaseUrl {
    match network {
        Network::Mainnet => BaseUrl::Mainnet,
        Network::Testnet => BaseUrl::Testnet,
    }
}

fn api_url(network: &Network) -> &'static str {
    match network {
        Network::Mainnet => "https://api.hyperliquid.xyz",
        Network::Testnet => "https://api.hyperliquid-testnet.xyz",
    }
}

/// Build an ExchangeClient with signing capability.
pub fn exchange_client(network: &Network) -> Result<ExchangeClient, CliError> {
    let key = std::env::var("HL_PRIVATE_KEY")
        .map_err(|_| CliError::Auth("HL_PRIVATE_KEY required for write operations".into()))?;
    let signer = PrivateKeySigner::from_str(&key)
        .map_err(|e| CliError::Auth(format!("Invalid HL_PRIVATE_KEY: {e}")))?;
    Ok(ExchangeClient::new(base_url(network)).with_signer(signer))
}

/// Load user address from HL_PRIVATE_KEY (derive) or HL_ADDRESS.
pub fn user_address() -> Result<Address, CliError> {
    if let Ok(key) = std::env::var("HL_PRIVATE_KEY") {
        let signer = PrivateKeySigner::from_str(&key)
            .map_err(|e| CliError::Auth(format!("Invalid HL_PRIVATE_KEY: {e}")))?;
        return Ok(signer.address());
    }
    if let Ok(addr) = std::env::var("HL_ADDRESS") {
        return Address::from_str(&addr)
            .map_err(|e| CliError::Auth(format!("Invalid HL_ADDRESS: {e}")));
    }
    Err(CliError::Auth(
        "Set HL_PRIVATE_KEY or HL_ADDRESS environment variable".into(),
    ))
}

/// Send a raw info request to the Hyperliquid /info endpoint.
pub async fn send_info_request<Req: Serialize, Resp: DeserializeOwned>(
    network: &Network,
    request: &Req,
) -> Result<Resp, CliError> {
    let url = format!("{}/info", api_url(network));
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(request).send().await?;
    let text = resp.text().await?;
    serde_json::from_str(&text)
        .map_err(|e| CliError::Api(format!("Failed to parse response: {e}\nBody: {text}")))
}

/// Resolve a coin name to its perp asset index by fetching metadata.
pub async fn resolve_asset_index(network: &Network, coin: &str) -> Result<u32, CliError> {
    #[derive(serde::Deserialize)]
    struct Meta {
        universe: Vec<AssetMeta>,
    }
    #[derive(serde::Deserialize)]
    struct AssetMeta {
        name: String,
    }

    let request = serde_json::json!({ "type": "meta" });
    let meta: Meta = send_info_request(network, &request).await?;

    for (i, asset) in meta.universe.iter().enumerate() {
        if asset.name.eq_ignore_ascii_case(coin) {
            return Ok(i as u32);
        }
    }
    Err(CliError::AssetNotFound(format!(
        "Unknown coin: {coin}. Use `hl meta` to list available assets."
    )))
}
