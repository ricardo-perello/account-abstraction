// Error types for the AA client
// Simplified to only include errors that are actually used
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AAError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
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
