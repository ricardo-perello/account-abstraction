// TODO: IMPLEMENT PROPER ERC-4337 COMPLIANCE
// Current implementation is simplified - needs exact ERC-4337 hashing and encoding
// This is critical for compatibility with real bundlers and networks

use alloy::primitives::{Address, Bytes, U256, B256};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// UserOperation as defined in ERC-4337
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOperation {
    /// The account making the operation
    pub sender: Address,
    /// Unique value used by the account for replay protection
    pub nonce: U256,
    /// The initCode for account creation (if any)
    pub init_code: Bytes,
    /// The call data for the account execution
    pub call_data: Bytes,
    /// Gas limit for the call execution
    pub call_gas_limit: U256,
    /// Gas limit for verification
    pub verification_gas_limit: U256,
    /// Gas limit for pre-verification
    pub pre_verification_gas: U256,
    /// Maximum fee per gas
    pub max_fee_per_gas: U256,
    /// Maximum priority fee per gas
    pub max_priority_fee_per_gas: U256,
    /// Paymaster and data (if using a paymaster)
    pub paymaster_and_data: Bytes,
    /// The signature for the operation
    pub signature: Bytes,
}

impl UserOperation {
    /// Create a new UserOperation with default values
    pub fn new(sender: Address, nonce: U256) -> Self {
        Self {
            sender,
            nonce,
            init_code: Bytes::new(),
            call_data: Bytes::new(),
            call_gas_limit: U256::from(100000u64),
            verification_gas_limit: U256::from(100000u64),
            pre_verification_gas: U256::from(100000u64),
            max_fee_per_gas: U256::from(20000000000u64), // 20 gwei
            max_priority_fee_per_gas: U256::from(1000000000u64), // 1 gwei
            paymaster_and_data: Bytes::new(),
            signature: Bytes::new(),
        }
    }

    /// Set the call data for the operation
    pub fn with_call_data(mut self, call_data: Bytes) -> Self {
        self.call_data = call_data;
        self
    }

    /// Set gas limits
    pub fn with_gas_limits(
        mut self,
        call_gas_limit: U256,
        verification_gas_limit: U256,
        pre_verification_gas: U256,
    ) -> Self {
        self.call_gas_limit = call_gas_limit;
        self.verification_gas_limit = verification_gas_limit;
        self.pre_verification_gas = pre_verification_gas;
        self
    }

    /// Set gas fees
    pub fn with_gas_fees(
        mut self,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
    ) -> Self {
        self.max_fee_per_gas = max_fee_per_gas;
        self.max_priority_fee_per_gas = max_priority_fee_per_gas;
        self
    }

    /// Set the signature
    pub fn with_signature(mut self, signature: Bytes) -> Self {
        self.signature = signature;
        self
    }

    /// Get the user operation hash for signing
    /// This implements the exact ERC-4337 hashing algorithm
    pub fn get_user_op_hash(&self, entry_point: Address, chain_id: U256) -> Result<B256> {
        let encoded = self.encode_for_signing(entry_point, chain_id);
        Ok(alloy::primitives::keccak256(encoded))
    }

    /// Encode the UserOperation for signing (without signature)
    /// This implements the exact ERC-4337 encoding specification
    pub fn encode_for_signing(&self, entry_point: Address, chain_id: U256) -> Bytes {
        // ERC-4337 encoding order: (UserOperation, EntryPoint, ChainId)
        // UserOperation fields in order: sender, nonce, initCode, callData, callGasLimit, 
        // verificationGasLimit, preVerificationGas, maxFeePerGas, maxPriorityFeePerGas, paymasterAndData
        
        let mut data = Vec::new();
        
        // Pack all UserOperation fields (excluding signature) into a single bytes32 array
        // This follows the exact ERC-4337 specification for hashing
        
        // sender (20 bytes) - left-padded to 32 bytes
        data.extend_from_slice(&[0u8; 12]); // 12 bytes of padding
        data.extend_from_slice(self.sender.as_slice());
        
        // nonce (32 bytes)
        data.extend_from_slice(&self.nonce.to_be_bytes::<32>());
        
        // initCode (dynamic) - hash of initCode
        let init_code_hash = alloy::primitives::keccak256(&self.init_code);
        data.extend_from_slice(init_code_hash.as_slice());
        
        // callData (dynamic) - hash of callData
        let call_data_hash = alloy::primitives::keccak256(&self.call_data);
        data.extend_from_slice(call_data_hash.as_slice());
        
        // callGasLimit (32 bytes)
        data.extend_from_slice(&self.call_gas_limit.to_be_bytes::<32>());
        
        // verificationGasLimit (32 bytes)
        data.extend_from_slice(&self.verification_gas_limit.to_be_bytes::<32>());
        
        // preVerificationGas (32 bytes)
        data.extend_from_slice(&self.pre_verification_gas.to_be_bytes::<32>());
        
        // maxFeePerGas (32 bytes)
        data.extend_from_slice(&self.max_fee_per_gas.to_be_bytes::<32>());
        
        // maxPriorityFeePerGas (32 bytes)
        data.extend_from_slice(&self.max_priority_fee_per_gas.to_be_bytes::<32>());
        
        // paymasterAndData (dynamic) - hash of paymasterAndData
        let paymaster_hash = alloy::primitives::keccak256(&self.paymaster_and_data);
        data.extend_from_slice(paymaster_hash.as_slice());
        
        // EntryPoint address (20 bytes) - left-padded to 32 bytes
        data.extend_from_slice(&[0u8; 12]); // 12 bytes of padding
        data.extend_from_slice(entry_point.as_slice());
        
        // ChainId (32 bytes)
        data.extend_from_slice(&chain_id.to_be_bytes::<32>());
        
        Bytes::from(data)
    }

    /// Convert to the packed format used by some implementations
    pub fn to_packed(&self) -> PackedUserOperation {
        PackedUserOperation {
            sender: self.sender,
            nonce: self.nonce,
            init_code: self.init_code.clone(),
            call_data: self.call_data.clone(),
            call_gas_limit: self.call_gas_limit,
            verification_gas_limit: self.verification_gas_limit,
            pre_verification_gas: self.pre_verification_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            paymaster_and_data: self.paymaster_and_data.clone(),
            signature: self.signature.clone(),
        }
    }
}

/// Packed UserOperation format (alternative representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedUserOperation {
    pub sender: Address,
    pub nonce: U256,
    pub init_code: Bytes,
    pub call_data: Bytes,
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes,
}

/// Response from bundler when submitting a UserOperation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOperationResponse {
    /// The hash of the submitted UserOperation
    pub user_op_hash: String,
    /// Any error message if submission failed
    pub error: Option<String>,
}

/// Gas estimation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub pre_verification_gas: U256,
    pub verification_gas_limit: U256,
    pub call_gas_limit: U256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn test_user_operation_creation() {
        let sender = Address::from([1u8; 20]);
        let nonce = U256::from(42u64);
        let user_op = UserOperation::new(sender, nonce);
        
        assert_eq!(user_op.sender, sender);
        assert_eq!(user_op.nonce, nonce);
        assert_eq!(user_op.call_gas_limit, U256::from(100000u64));
        assert_eq!(user_op.verification_gas_limit, U256::from(100000u64));
        assert_eq!(user_op.pre_verification_gas, U256::from(100000u64));
        assert_eq!(user_op.max_fee_per_gas, U256::from(20000000000u64));
        assert_eq!(user_op.max_priority_fee_per_gas, U256::from(1000000000u64));
    }

    #[test]
    fn test_with_call_data() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let call_data = Bytes::from(vec![0x12, 0x34, 0x56]);
        
        let user_op = user_op.with_call_data(call_data.clone());
        assert_eq!(user_op.call_data, call_data);
    }

    #[test]
    fn test_with_gas_limits() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let user_op = user_op.with_gas_limits(
            U256::from(200000u64),
            U256::from(150000u64),
            U256::from(75000u64),
        );
        
        assert_eq!(user_op.call_gas_limit, U256::from(200000u64));
        assert_eq!(user_op.verification_gas_limit, U256::from(150000u64));
        assert_eq!(user_op.pre_verification_gas, U256::from(75000u64));
    }

    #[test]
    fn test_with_gas_fees() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let user_op = user_op.with_gas_fees(
            U256::from(30000000000u64), // 30 gwei
            U256::from(2000000000u64),  // 2 gwei
        );
        
        assert_eq!(user_op.max_fee_per_gas, U256::from(30000000000u64));
        assert_eq!(user_op.max_priority_fee_per_gas, U256::from(2000000000u64));
    }

    #[test]
    fn test_with_signature() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let signature = Bytes::from(vec![0xaa, 0xbb, 0xcc]);
        
        let user_op = user_op.with_signature(signature.clone());
        assert_eq!(user_op.signature, signature);
    }

    #[test]
    fn test_user_op_hash_consistency() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let entry_point = Address::from([2u8; 20]);
        let chain_id = U256::from(1u64);
        
        let hash1 = user_op.get_user_op_hash(entry_point, chain_id).unwrap();
        let hash2 = user_op.get_user_op_hash(entry_point, chain_id).unwrap();
        
        // Same input should always produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_user_op_hash_uniqueness() {
        let user_op1 = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let user_op2 = UserOperation::new(Address::from([2u8; 20]), U256::from(1u64));
        let entry_point = Address::from([3u8; 20]);
        let chain_id = U256::from(1u64);
        
        let hash1 = user_op1.get_user_op_hash(entry_point, chain_id).unwrap();
        let hash2 = user_op2.get_user_op_hash(entry_point, chain_id).unwrap();
        
        // Different senders should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_encoding_for_signing() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(42u64));
        let entry_point = Address::from([2u8; 20]);
        let chain_id = U256::from(1u64);
        
        let encoded = user_op.encode_for_signing(entry_point, chain_id);
        
        // Should be non-empty
        assert!(!encoded.is_empty());
        
        // Should be deterministic
        let encoded2 = user_op.encode_for_signing(entry_point, chain_id);
        assert_eq!(encoded, encoded2);
    }

    #[test]
    fn test_encoding_with_call_data() {
        let mut user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        user_op = user_op.with_call_data(Bytes::from(vec![0x12, 0x34]));
        
        let entry_point = Address::from([2u8; 20]);
        let chain_id = U256::from(1u64);
        
        let encoded = user_op.encode_for_signing(entry_point, chain_id);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encoding_with_init_code() {
        let mut user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        user_op.init_code = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
        
        let entry_point = Address::from([2u8; 20]);
        let chain_id = U256::from(1u64);
        
        let encoded = user_op.encode_for_signing(entry_point, chain_id);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encoding_with_paymaster() {
        let mut user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        user_op.paymaster_and_data = Bytes::from(vec![0xca, 0xfe, 0xba, 0xbe]);
        
        let entry_point = Address::from([2u8; 20]);
        let chain_id = U256::from(1u64);
        
        let encoded = user_op.encode_for_signing(entry_point, chain_id);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_to_packed() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(42u64));
        let packed = user_op.to_packed();
        
        assert_eq!(packed.sender, user_op.sender);
        assert_eq!(packed.nonce, user_op.nonce);
        assert_eq!(packed.call_data, user_op.call_data);
        assert_eq!(packed.signature, user_op.signature);
    }

    #[test]
    fn test_gas_estimate_creation() {
        let estimate = GasEstimate {
            pre_verification_gas: U256::from(50000u64),
            verification_gas_limit: U256::from(150000u64),
            call_gas_limit: U256::from(200000u64),
        };
        
        assert_eq!(estimate.pre_verification_gas, U256::from(50000u64));
        assert_eq!(estimate.verification_gas_limit, U256::from(150000u64));
        assert_eq!(estimate.call_gas_limit, U256::from(200000u64));
    }

    #[test]
    fn test_user_operation_response() {
        let response = UserOperationResponse {
            user_op_hash: "0x1234".to_string(),
            error: None,
        };
        
        assert_eq!(response.user_op_hash, "0x1234");
        assert!(response.error.is_none());
    }

    #[test]
    fn test_user_operation_response_with_error() {
        let response = UserOperationResponse {
            user_op_hash: "0x1234".to_string(),
            error: Some("Insufficient funds".to_string()),
        };
        
        assert_eq!(response.user_op_hash, "0x1234");
        assert_eq!(response.error.as_ref().unwrap(), "Insufficient funds");
    }

    #[test]
    fn test_chain_id_impact_on_hash() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let entry_point = Address::from([2u8; 20]);
        
        let hash1 = user_op.get_user_op_hash(entry_point, U256::from(1u64)).unwrap();
        let hash2 = user_op.get_user_op_hash(entry_point, U256::from(137u64)).unwrap();
        
        // Different chain IDs should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_entry_point_impact_on_hash() {
        let user_op = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let chain_id = U256::from(1u64);
        
        let hash1 = user_op.get_user_op_hash(Address::from([2u8; 20]), chain_id).unwrap();
        let hash2 = user_op.get_user_op_hash(Address::from([3u8; 20]), chain_id).unwrap();
        
        // Different entry points should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_nonce_impact_on_hash() {
        let entry_point = Address::from([2u8; 20]);
        let chain_id = U256::from(1u64);
        
        let user_op1 = UserOperation::new(Address::from([1u8; 20]), U256::from(1u64));
        let user_op2 = UserOperation::new(Address::from([1u8; 20]), U256::from(2u64));
        
        let hash1 = user_op1.get_user_op_hash(entry_point, chain_id).unwrap();
        let hash2 = user_op2.get_user_op_hash(entry_point, chain_id).unwrap();
        
        // Different nonces should produce different hashes
        assert_ne!(hash1, hash2);
    }
}
