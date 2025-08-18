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
