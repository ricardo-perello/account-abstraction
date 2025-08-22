use alloy::primitives::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PaymasterConfig {
    pub paymaster_address: Address,
    pub signature: [u8; 65],      // ECDSA signature from verifier
    pub valid_until: u64,         // Expiration timestamp
}

#[derive(Debug, Deserialize)]
pub struct SponsorshipResponse {
    pub signature: String,
    pub valid_until: u64,
    pub paymaster_data: String,
}

#[derive(Debug)]
pub enum PaymasterError {
    InvalidSignature,
    ExpiredSignature,
    InvalidVerifier,
    NetworkError(String),
}

impl std::fmt::Display for PaymasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymasterError::InvalidSignature => write!(f, "Invalid signature"),
            PaymasterError::ExpiredSignature => write!(f, "Signature expired"),
            PaymasterError::InvalidVerifier => write!(f, "Invalid verifier"),
            PaymasterError::NetworkError(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for PaymasterError {}
