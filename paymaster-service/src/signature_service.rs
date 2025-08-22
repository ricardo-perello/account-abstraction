use alloy::primitives::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::key_manager::{KeyManager, KeyManagerError};

#[derive(Debug, Deserialize)]
pub struct SponsorshipRequest {
    pub user_operation_hash: String,
    pub max_gas_cost: U256,
    pub valid_until: u64,
    pub verifier: String,
}

#[derive(Debug, Serialize)]
pub struct SponsorshipResponse {
    pub signature: String,
    pub valid_until: u64,
    pub paymaster_data: String,
}

#[derive(Debug)]
pub enum SignatureError {
    InvalidTimestamp,
    KeyManagerError(KeyManagerError),
    EncodingError,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            SignatureError::KeyManagerError(e) => write!(f, "Key manager error: {}", e),
            SignatureError::EncodingError => write!(f, "Encoding error"),
        }
    }
}

impl std::error::Error for SignatureError {}

impl From<KeyManagerError> for SignatureError {
    fn from(err: KeyManagerError) -> Self {
        SignatureError::KeyManagerError(err)
    }
}

pub struct SignatureService {
    key_manager: Arc<KeyManager>,
}

impl SignatureService {
    pub fn new(key_manager: Arc<KeyManager>) -> Self {
        Self { key_manager }
    }
    
    pub async fn sign_sponsorship(
        &self,
        request: SponsorshipRequest,
    ) -> Result<SponsorshipResponse, SignatureError> {
        // Validate request
        if request.valid_until <= chrono::Utc::now().timestamp() as u64 {
            return Err(SignatureError::InvalidTimestamp);
        }
        
        // Create message hash
        let message = self.create_message_hash(
            &request.user_operation_hash,
            request.valid_until,
            request.max_gas_cost
        );
        
        // Sign with verifier key
        let signature = self.key_manager
            .sign_sponsorship(&request.verifier, &message)
            .await?;
        
        // Encode paymaster data
        let paymaster_data = self.encode_paymaster_data(&signature, request.valid_until);
        
        Ok(SponsorshipResponse {
            signature: hex::encode(&signature),
            valid_until: request.valid_until,
            paymaster_data: hex::encode(&paymaster_data),
        })
    }
    
    fn create_message_hash(&self, user_op_hash: &str, valid_until: u64, max_gas_cost: U256) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        
        let mut encoded = Vec::new();
        encoded.extend_from_slice(user_op_hash.as_bytes());
        encoded.extend_from_slice(&valid_until.to_be_bytes());
        encoded.extend_from_slice(&max_gas_cost.to_be_bytes());
        
        let hash = Keccak256::digest(encoded);
        hash.to_vec()
    }
    
    fn encode_paymaster_data(&self, signature: &[u8], valid_until: u64) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(signature);
        data.extend_from_slice(&valid_until.to_be_bytes());
        data
    }
    
    pub async fn get_metrics(&self) -> Metrics {
        Metrics {
            verifier_count: self.key_manager.get_verifier_count().await,
            service_status: "healthy".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub verifier_count: usize,
    pub service_status: String,
}
