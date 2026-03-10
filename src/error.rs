use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Client initialization error: {0}")]
    ClientInit(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArg(String),

    #[error("{0}")]
    Serialization(String),
}

impl From<hl_rs::Error> for CliError {
    fn from(e: hl_rs::Error) -> Self {
        CliError::Api(e.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Serialization(e.to_string())
    }
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        CliError::Api(e.to_string())
    }
}
