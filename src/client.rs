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

/// Build a mapping of @N spot indices to human-readable "BASE/QUOTE" names.
pub async fn resolve_spot_names(
    network: &Network,
) -> Result<std::collections::HashMap<String, String>, CliError> {
    #[derive(serde::Deserialize)]
    struct SpotMeta {
        tokens: Vec<SpotToken>,
        universe: Vec<SpotPair>,
    }
    #[derive(serde::Deserialize)]
    struct SpotToken {
        index: u32,
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct SpotPair {
        index: u32,
        tokens: Vec<u32>,
    }

    let request = serde_json::json!({ "type": "spotMeta" });
    let meta: SpotMeta = send_info_request(network, &request).await?;

    let token_names: std::collections::HashMap<u32, String> =
        meta.tokens.into_iter().map(|t| (t.index, t.name)).collect();

    let mut mapping = std::collections::HashMap::new();
    for pair in meta.universe {
        if pair.tokens.len() >= 2 {
            let base = token_names
                .get(&pair.tokens[0])
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            let quote = token_names
                .get(&pair.tokens[1])
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            mapping.insert(format!("@{}", pair.index), format!("{base}/{quote}"));
        }
    }
    Ok(mapping)
}

/// Parse dex context from --dex flag and/or coin name containing ":".
/// Returns (dex_name, full_coin_name) where full_coin_name is "dex:COIN" for HIP-3.
pub fn resolve_coin_and_dex(dex: Option<&str>, coin: &str) -> (Option<String>, String) {
    if let Some((d, c)) = coin.split_once(':') {
        (Some(d.to_string()), format!("{d}:{c}"))
    } else if let Some(d) = dex {
        (Some(d.to_string()), format!("{d}:{coin}"))
    } else {
        (None, coin.to_string())
    }
}

/// Resolve a coin name to its perp asset index by fetching metadata.
/// For HIP-3 assets (when dex is Some), computes 100000 + dex_index * 10000 + asset_index.
pub async fn resolve_asset_index(
    network: &Network,
    dex: Option<&str>,
    coin: &str,
) -> Result<u32, CliError> {
    let (dex_name, _full_coin) = resolve_coin_and_dex(dex, coin);

    if let Some(dex_name) = dex_name {
        return resolve_hip3_asset_index(network, &dex_name, coin).await;
    }

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

/// Resolve a HIP-3 asset to its full asset ID.
/// Formula: 100000 + perp_dex_index * 10000 + index_in_dex_meta
async fn resolve_hip3_asset_index(
    network: &Network,
    dex_name: &str,
    coin: &str,
) -> Result<u32, CliError> {
    // Strip dex prefix if present (e.g. "xyz:TSLA" -> "TSLA")
    let bare_coin = coin.split_once(':').map(|(_, c)| c).unwrap_or(coin);

    // Fetch dex index from perpDexs list
    let dexes: Vec<Option<DexInfo>> =
        send_info_request(network, &serde_json::json!({ "type": "perpDexs" })).await?;

    let dex_index = dexes
        .iter()
        .position(|d| {
            d.as_ref()
                .map(|info| info.name.eq_ignore_ascii_case(dex_name))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            CliError::AssetNotFound(format!(
                "Unknown dex: {dex_name}. Use `hl dexes` to list available dexes."
            ))
        })? as u32;

    // Fetch dex meta to find asset index within dex
    #[derive(serde::Deserialize)]
    struct AssetMeta {
        name: String,
    }

    let meta_response: (serde_json::Value, serde_json::Value) = send_info_request(
        network,
        &serde_json::json!({ "type": "metaAndAssetCtxs", "dex": dex_name }),
    )
    .await?;

    let universe: Vec<AssetMeta> = serde_json::from_value(
        meta_response
            .0
            .get("universe")
            .cloned()
            .unwrap_or_default(),
    )
    .map_err(|e| CliError::Api(format!("Failed to parse dex meta: {e}")))?;

    let full_name = format!("{dex_name}:{bare_coin}");
    let asset_index = universe
        .iter()
        .position(|a| a.name.eq_ignore_ascii_case(&full_name))
        .ok_or_else(|| {
            CliError::AssetNotFound(format!(
                "Unknown coin: {bare_coin} on dex {dex_name}. Use `hl --dex {dex_name} meta` to list."
            ))
        })? as u32;

    Ok(100_000 + dex_index * 10_000 + asset_index)
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DexInfo {
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub deployer: String,
}
