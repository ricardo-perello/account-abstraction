use alloy_primitives::U256;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use crate::key_manager::{KeyManager, KeyManagerError};

#[derive(Debug, Deserialize)]
pub struct SponsorshipRequest {
    pub api_key: String,
    pub user_operation: PackedUserOperation,
    pub valid_until: u64,
    pub valid_after: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PackedUserOperation {
    pub sender: String,
    pub nonce: U256,
    pub init_code: String,
    pub call_data: String,
    pub account_gas_limits: String,    // bytes32 packed
    pub pre_verification_gas: U256,
    pub gas_fees: String,              // bytes32 packed
    pub paymaster_and_data: String,
}

#[derive(Debug, Serialize)]
pub struct SponsorshipResponse {
    pub signature: String,
    pub valid_until: u64,
    pub valid_after: u64,
    pub paymaster_data: String,
}

#[derive(Debug)]
pub enum SignatureError {
    InvalidApiKey,
    InvalidTimestamp,
    KeyManagerError(KeyManagerError),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::InvalidApiKey => write!(f, "Invalid API key"),
            SignatureError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            SignatureError::KeyManagerError(e) => write!(f, "Key manager error: {}", e),
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
    api_keys: HashMap<String, String>, // api_key -> client_name
    chain_id: u64,
    paymaster_address: Vec<u8>,
}

impl SignatureService {
    pub fn new(
        key_manager: Arc<KeyManager>, 
        api_keys: HashMap<String, String>, 
        chain_id: u64,
        paymaster_address: Vec<u8>
    ) -> Self {
        Self {
            key_manager,
            api_keys,
            chain_id,
            paymaster_address,
        }
    }
    
    pub async fn sign_sponsorship(
        &self,
        request: SponsorshipRequest,
    ) -> Result<SponsorshipResponse, SignatureError> {
        // 1. Check API key
        if !self.api_keys.contains_key(&request.api_key) {
            return Err(SignatureError::InvalidApiKey);
        }

        // 2. Validate timestamp
        if request.valid_until <= chrono::Utc::now().timestamp() as u64 {
            return Err(SignatureError::InvalidTimestamp);
        }
        
        let valid_after = request.valid_after.unwrap_or(0);
        
        // 3. Create paymaster message hash (matches VerifierSignaturePaymaster._pmHash)
        let paymaster_hash = self.create_paymaster_hash(
            &request.user_operation,
            request.valid_until,
            valid_after
        );
        
        // 4. Apply EIP-191 formatting (matches VerifierSignaturePaymaster digest)
        let eip191_message = self.create_eip191_message(&paymaster_hash);
        
        // 5. Sign with default verifier key
        let signature = self.key_manager
            .sign_eip191_message("default", &eip191_message)
            .await?;
        
        // 6. Encode paymaster data (signature + validUntil + validAfter)
        let paymaster_data = self.encode_paymaster_data(&signature, request.valid_until, valid_after);
        
        Ok(SponsorshipResponse {
            signature: hex::encode(&signature),
            valid_until: request.valid_until,
            valid_after,
            paymaster_data: hex::encode(&paymaster_data),
        })
    }
    
    // Pack UserOperation for paymaster (matches VerifierSignaturePaymaster._packForPaymaster)
    fn pack_for_paymaster(&self, user_op: &PackedUserOperation) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        
        // Parse hex strings (remove 0x prefix if present)
        let init_code = self.decode_hex(&user_op.init_code);
        let call_data = self.decode_hex(&user_op.call_data);
        
        // Hash init_code and call_data (as per _packForPaymaster)
        let init_code_hash = Keccak256::digest(&init_code);
        let call_data_hash = Keccak256::digest(&call_data);
        
        // Solidity abi.encode format - each field is 32-byte aligned
        let mut encoded = Vec::new();
        
        // sender (address - left-padded to 32 bytes)
        let sender_bytes = self.decode_hex(&user_op.sender);
        encoded.extend_from_slice(&[0u8; 12]); // pad to 32 bytes
        encoded.extend_from_slice(&sender_bytes);
        
        // nonce (uint256 - 32 bytes)
        encoded.extend_from_slice(&user_op.nonce.to_be_bytes::<32>());
        
        // keccak256(initCode) (bytes32)
        encoded.extend_from_slice(&init_code_hash);
        
        // keccak256(callData) (bytes32) 
        encoded.extend_from_slice(&call_data_hash);
        
        // accountGasLimits (bytes32) - already packed
        let gas_limits = self.decode_hex(&user_op.account_gas_limits);
        let mut gas_limits_32 = [0u8; 32];
        gas_limits_32[32 - gas_limits.len()..].copy_from_slice(&gas_limits);
        encoded.extend_from_slice(&gas_limits_32);
        
        // preVerificationGas (uint256 - 32 bytes)
        encoded.extend_from_slice(&user_op.pre_verification_gas.to_be_bytes::<32>());
        
        // gasFees (bytes32) - already packed
        let gas_fees = self.decode_hex(&user_op.gas_fees);
        let mut gas_fees_32 = [0u8; 32];
        gas_fees_32[32 - gas_fees.len()..].copy_from_slice(&gas_fees);
        encoded.extend_from_slice(&gas_fees_32);
        
        encoded
    }
    
    // Create paymaster hash (matches VerifierSignaturePaymaster._pmHash exactly)
    fn create_paymaster_hash(&self, user_op: &PackedUserOperation, valid_until: u64, valid_after: u64) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        
        let packed_user_op = self.pack_for_paymaster(user_op);
        
        // Solidity abi.encode format for the _pmHash function:
        // abi.encode(_packForPaymaster(u), block.chainid, address(this), validUntil, validAfter)
        let mut encoded = Vec::new();
        
        // _packForPaymaster(u) - dynamic bytes, so we need offset + length + data
        let offset = 5 * 32; // 5 fields after this one (chain_id, address, validUntil, validAfter, length)
        let mut offset_bytes = [0u8; 32];
        offset_bytes[24..].copy_from_slice(&(offset as u64).to_be_bytes());
        encoded.extend_from_slice(&offset_bytes); // offset to packed data
        
        // block.chainid (uint256 - 32 bytes)
        let mut chain_id_bytes = [0u8; 32];
        chain_id_bytes[24..].copy_from_slice(&self.chain_id.to_be_bytes());
        encoded.extend_from_slice(&chain_id_bytes);
        
        // address(this) - paymaster address (address - left-padded to 32 bytes)
        encoded.extend_from_slice(&[0u8; 12]); // pad to 32 bytes
        encoded.extend_from_slice(&self.paymaster_address);
        
        // validUntil (uint64 - right-padded to 32 bytes)
        let mut valid_until_bytes = [0u8; 32];
        valid_until_bytes[24..].copy_from_slice(&valid_until.to_be_bytes());
        encoded.extend_from_slice(&valid_until_bytes);
        
        // validAfter (uint64 - right-padded to 32 bytes) 
        let mut valid_after_bytes = [0u8; 32];
        valid_after_bytes[24..].copy_from_slice(&valid_after.to_be_bytes());
        encoded.extend_from_slice(&valid_after_bytes);
        
        // Length of packed_user_op
        let mut length_bytes = [0u8; 32];
        length_bytes[24..].copy_from_slice(&(packed_user_op.len() as u64).to_be_bytes());
        encoded.extend_from_slice(&length_bytes);
        
        // Actual packed_user_op data (padded to 32-byte boundary)
        encoded.extend_from_slice(&packed_user_op);
        let padding = 32 - (packed_user_op.len() % 32);
        if padding != 32 {
            encoded.extend_from_slice(&vec![0u8; padding]);
        }
        
        let hash = Keccak256::digest(encoded);
        hash.to_vec()
    }
    
    // Apply EIP-191 formatting (matches MessageHashUtils.toEthSignedMessageHash)
    fn create_eip191_message(&self, hash: &[u8]) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
        message.extend_from_slice(hash);
        message
    }
    
    // Encode paymaster data: signature (65) + validUntil (8) + validAfter (8)
    fn encode_paymaster_data(&self, signature: &[u8], valid_until: u64, valid_after: u64) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(signature);                    // 65 bytes signature
        data.extend_from_slice(&valid_until.to_be_bytes());   // 8 bytes
        data.extend_from_slice(&valid_after.to_be_bytes());   // 8 bytes
        data                                                  // Total: 81 bytes
    }
    
    // Helper to decode hex strings
    fn decode_hex(&self, hex_str: &str) -> Vec<u8> {
        let hex_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        hex::decode(hex_clean).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;

    fn create_test_config() -> crate::Config {
        let mut verifier_keys = HashMap::new();
        verifier_keys.insert("default".to_string(), "0000000000000000000000000000000000000000000000000000000000000001".to_string());
        
        crate::Config {
            verifier_keys,
            api_keys: HashMap::new(),
            server_port: 3000,
            log_level: "info".to_string(),
            chain_id: Some(1),
            paymaster_address: Some("0x0000000000000000000000000000000000000000".to_string()),
        }
    }

    fn create_test_api_keys() -> HashMap<String, String> {
        let mut api_keys = HashMap::new();
        api_keys.insert("test_key_123".to_string(), "Test Client".to_string());
        api_keys
    }

    fn create_test_request() -> SponsorshipRequest {
        SponsorshipRequest {
            api_key: "test_key_123".to_string(),
            user_operation: PackedUserOperation {
                sender: "0x1234567890123456789012345678901234567890".to_string(),
                nonce: U256::from(1),
                init_code: "0x".to_string(),
                call_data: "0x1234".to_string(),
                account_gas_limits: "0x00000000000f424000000000000f4240".to_string(), // 1M each
                pre_verification_gas: U256::from(21000),
                gas_fees: "0x000000000077359400000000003b9aca00".to_string(), // 2 gwei, 1 gwei
                paymaster_and_data: "0x".to_string(),
            },
            valid_until: (Utc::now().timestamp() + 3600) as u64,
            valid_after: Some(0),
        }
    }

    #[tokio::test]
    async fn test_valid_sponsorship_request() {
        let config = create_test_config();
        let key_manager = Arc::new(KeyManager::new(&config));
        let api_keys = create_test_api_keys();
        let signature_service = SignatureService::new(
            key_manager, 
            api_keys, 
            1, // chain_id
            vec![0u8; 20] // paymaster_address
        );
        
        let request = create_test_request();
        let result = signature_service.sign_sponsorship(request).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // Check response structure
        assert!(!response.signature.is_empty());
        assert!(!response.paymaster_data.is_empty());
        assert!(response.valid_until > 0);
        
        // Signature should be hex encoded (130 chars for 65 bytes)
        assert_eq!(response.signature.len(), 130);
        
        // Paymaster data from service response should contain signature + validUntil + validAfter (65 + 8 + 8 bytes = 162 hex chars)
        assert_eq!(response.paymaster_data.len(), 162);
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let config = create_test_config();
        let key_manager = Arc::new(KeyManager::new(&config));
        let api_keys = create_test_api_keys();
        let signature_service = SignatureService::new(
            key_manager, 
            api_keys, 
            1, // chain_id
            vec![0u8; 20] // paymaster_address
        );
        
        let mut request = create_test_request();
        request.api_key = "invalid_key".to_string();
        
        let result = signature_service.sign_sponsorship(request).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SignatureError::InvalidApiKey));
    }

    #[tokio::test]
    async fn test_expired_timestamp() {
        let config = create_test_config();
        let key_manager = Arc::new(KeyManager::new(&config));
        let api_keys = create_test_api_keys();
        let signature_service = SignatureService::new(
            key_manager, 
            api_keys, 
            1, // chain_id
            vec![0u8; 20] // paymaster_address
        );
        
        let mut request = create_test_request();
        request.valid_until = (Utc::now().timestamp() - 3600) as u64; // Expired
        
        let result = signature_service.sign_sponsorship(request).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SignatureError::InvalidTimestamp));
    }
}

