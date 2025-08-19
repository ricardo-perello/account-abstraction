// Refactored to use aa-sdk-rs types and functionality
// This replaces the custom implementation with the proper SDK

use alloy::primitives::{Address, Bytes, U256, B256};
use serde::{Deserialize, Serialize};
use anyhow::Result;

// Re-export aa-sdk-rs types for compatibility
pub use aa_sdk_rs::types::{
    UserOperation as SdkUserOperation,
    UserOperationRequest, 
    AccountCall, 
    ExecuteCall
};

// Use aa-sdk-rs UserOperation directly instead of custom implementation
pub type UserOperation = SdkUserOperation;

/// Helper struct for creating user operations with a builder pattern
/// This provides backward compatibility with existing code
pub struct UserOperationBuilder {
    request: UserOperationRequest,
}

impl UserOperationBuilder {
    /// Create a new UserOperationBuilder with a simple execute call
    pub fn new(target: Address, value: U256, call_data: Bytes) -> Self {
        let execute_call = ExecuteCall::new(target, value, call_data);
        let account_call = AccountCall::Execute(execute_call);
        let request = UserOperationRequest::new_with_call(account_call);
        
        Self { request }
    }

    /// Set the sender address
    pub fn with_sender(mut self, sender: Address) -> Self {
        self.request = self.request.sender(sender);
        self
    }

    /// Set gas fees
    pub fn with_gas_fees(mut self, max_fee_per_gas: U256, max_priority_fee_per_gas: U256) -> Self {
        self.request = self.request
            .max_fee_per_gas(max_fee_per_gas)
            .max_priority_fee_per_gas(max_priority_fee_per_gas);
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: U256) -> Self {
        self.request = self.request.nonce(nonce);
        self
    }

    /// Build the final UserOperationRequest
    pub fn build(self) -> UserOperationRequest {
        self.request
    }
}

/// Helper functions for backward compatibility
pub fn create_execute_call(target: Address, value: U256, data: Bytes) -> ExecuteCall {
    ExecuteCall::new(target, value, data)
}

pub fn create_account_call(execute_call: ExecuteCall) -> AccountCall {
    AccountCall::Execute(execute_call)
}

pub fn create_user_operation_request(account_call: AccountCall) -> UserOperationRequest {
    UserOperationRequest::new_with_call(account_call)
}

// Remove custom PackedUserOperation - use aa-sdk-rs types instead

// Custom response types removed - aa-sdk-rs provides these through SmartAccountProvider
// Use UserOpHash from aa-sdk-rs::types::request::UserOpHash 
// Use UserOperationGasEstimation from aa-sdk-rs::types::UserOperationGasEstimation

// Re-export aa-sdk-rs types for responses
pub use aa_sdk_rs::types::{
    request::UserOpHash,
    UserOperationGasEstimation as GasEstimate,
};

/// Temporary compatibility wrapper for existing code
/// TODO: Replace all usage with aa-sdk-rs types directly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOperationResponse {
    /// The hash of the submitted UserOperation
    pub user_op_hash: String,
    /// Any error message if submission failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn test_user_operation_builder() {
        let target = Address::from([1u8; 20]);
        let value = U256::from(100);
        let call_data = Bytes::from(vec![0x12, 0x34, 0x56]);
        
        let builder = UserOperationBuilder::new(target, value, call_data)
            .with_sender(Address::from([2u8; 20]))
            .with_nonce(U256::from(42))
            .with_gas_fees(U256::from(20000000000u64), U256::from(1000000000u64));
        
        let request = builder.build();
        // Test passes if builder works without errors
    }

    #[test]
    fn test_helper_functions() {
        let execute_call = create_execute_call(
            Address::ZERO,
            U256::from(100),
            Bytes::default(),
        );
        
        let account_call = create_account_call(execute_call);
        let _user_op_request = create_user_operation_request(account_call);
        
        // Test passes if helper functions work without errors
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
}
