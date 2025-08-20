// Error types for the AA client
// Remove specific provider error import to avoid API compatibility issues
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AAError {
    #[error("Invalid UserOperation: {0}")]
    InvalidUserOp(String),
    
    #[error("Bundler error: {0}")]
    BundlerError(String),
    
    #[error("Signature validation failed")]
    InvalidSignature,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Contract error: {0}")]
    ContractError(String),
    
    #[error("Gas estimation failed: {0}")]
    GasEstimationError(String),
    
    #[error("Unsupported network: chain ID {0}")]
    UnsupportedNetwork(u64),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Nonce management error: {0}")]
    NonceError(String),
    
    #[error("Factory error: {0}")]
    FactoryError(String),
    
    #[error("aa-sdk-rs error: {0}")]
    SdkError(#[from] aa_sdk_rs::provider::SmartAccountError),
}

impl From<anyhow::Error> for AAError {
    fn from(err: anyhow::Error) -> Self {
        AAError::ConfigError(err.to_string())
    }
}

// Simplified error conversions
impl From<std::num::ParseIntError> for AAError {
    fn from(err: std::num::ParseIntError) -> Self {
        AAError::ValidationError(format!("Parse error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, AAError>;
