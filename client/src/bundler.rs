// Full implementation with real network calls and ABIs
// This implements actual bundler RPC calls and contract interactions

use alloy::primitives::{Address, U256, B256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use anyhow::Result;
use url::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Standard ERC-4337 SimpleAccountFactory ABI
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SimpleAccountFactory,
    r#"[
        {
            "inputs": [
                {"internalType": "address", "name": "owner", "type": "address"},
                {"internalType": "uint256", "name": "salt", "type": "uint256"}
            ],
            "name": "createAccount",
            "outputs": [
                {"internalType": "contract SimpleAccount", "name": "ret", "type": "address"}
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "owner", "type": "address"},
                {"internalType": "uint256", "name": "salt", "type": "uint256"}
            ],
            "name": "getAddress",
            "outputs": [
                {"internalType": "address", "name": "", "type": "address"}
            ],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#
);

// AAAccountFactory ABI for multi-owner support
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AAAccountFactory,
    r#"[
        {
            "inputs": [
                {"internalType": "address", "name": "owner", "type": "address"},
                {"internalType": "uint256", "name": "salt", "type": "uint256"}
            ],
            "name": "createAccount",
            "outputs": [
                {"internalType": "contract AAAccount", "name": "account", "type": "address"}
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address[]", "name": "owners", "type": "address[]"},
                {"internalType": "uint256", "name": "salt", "type": "uint256"}
            ],
            "name": "createAccountWithOwners",
            "outputs": [
                {"internalType": "contract AAAccount", "name": "account", "type": "address"}
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "owner", "type": "address"},
                {"internalType": "uint256", "name": "salt", "type": "uint256"}
            ],
            "name": "getAddress",
            "outputs": [
                {"internalType": "address", "name": "", "type": "address"}
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address[]", "name": "owners", "type": "address[]"},
                {"internalType": "uint256", "name": "salt", "type": "uint256"}
            ],
            "name": "getAddressWithOwners",
            "outputs": [
                {"internalType": "address", "name": "", "type": "address"}
            ],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#
);

// Real bundler JSON-RPC request/response types
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    pub jsonrpc: String,
    #[allow(dead_code)]
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[allow(dead_code)]
    pub data: Option<serde_json::Value>,
}

// Note: Compatibility types moved to aa-sdk-rs
// UserOperationResponse and GasEstimate are re-exported from userop module when needed

/// Modern bundler client wrapper that can create aa-sdk-rs providers
/// This provides compatibility while enabling use of aa-sdk-rs functionality
pub struct BundlerClient {
    rpc_url: String,
    entry_point: Address,
    #[allow(dead_code)]
    chain_id: U256,
}

// JSON-RPC structures removed - aa-sdk-rs handles this internally

impl BundlerClient {
    /// Create a new bundler client
    pub fn new(rpc_url: String, entry_point: Address, chain_id: U256) -> Self {
        Self {
            rpc_url,
            entry_point,
            chain_id,
        }
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Get the entry point address
    #[allow(dead_code)]
    pub fn entry_point(&self) -> Address {
        self.entry_point
    }

    /// Get the chain ID
    #[allow(dead_code)]
    pub fn chain_id(&self) -> U256 {
        self.chain_id
    }

    /// Create an Alloy provider from this bundler client configuration
    pub async fn create_provider(&self) -> Result<impl Provider<alloy::network::Ethereum>> {
        let url = Url::parse(&self.rpc_url)?;
        let provider = ProviderBuilder::new().on_http(url);
        Ok(provider)
    }

    /// Get real predicted address from standard ERC-4337 SimpleAccountFactory contract
    pub async fn get_predicted_address(&self, factory_address: Address, owner: Address, salt: U256) -> Result<Address> {
        let provider = self.create_provider().await?;
        let factory_contract = SimpleAccountFactory::new(factory_address, &provider);
        
        let result = factory_contract.getAddress(owner, salt).call().await?;
        Ok(result._0)
    }

    /// Get predicted address for multi-owner account from AAAccountFactory contract
    pub async fn get_predicted_multi_owner_address(&self, factory_address: Address, owners: Vec<Address>, salt: U256) -> Result<Address> {
        let provider = self.create_provider().await?;
        let factory_contract = AAAccountFactory::new(factory_address, &provider);
        
        let result = factory_contract.getAddressWithOwners(owners, salt).call().await?;
        Ok(result._0)
    }

    /// Submit UserOperation to bundler via eth_sendUserOperation
    pub async fn submit_user_operation(&self, user_op: &aa_sdk_rs::types::UserOperationRequest) -> Result<B256> {
        let client = reqwest::Client::new();
        
        // Convert UserOperationRequest to JSON format expected by bundlers
        let user_op_json = self.user_op_to_json(user_op)?;
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_sendUserOperation".to_string(),
            params: json!([user_op_json, self.entry_point]),
            id: 1,
        };

        let response = client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse<String> = response.json().await?;
        
        if let Some(error) = rpc_response.error {
            return Err(anyhow::anyhow!("RPC Error {}: {}", error.code, error.message));
        }

        let hash_str = rpc_response.result.ok_or_else(|| anyhow::anyhow!("No result in response"))?;
        let hash = B256::from_slice(&hex::decode(hash_str.strip_prefix("0x").unwrap_or(&hash_str))?);
        Ok(hash)
    }

    /// Get real gas estimation from bundler via eth_estimateUserOperationGas
    pub async fn estimate_user_operation_gas(&self, user_op: &aa_sdk_rs::types::UserOperationRequest) -> Result<aa_sdk_rs::types::UserOperationGasEstimation> {
        let client = reqwest::Client::new();
        
        let user_op_json = self.user_op_to_json(user_op)?;
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_estimateUserOperationGas".to_string(),
            params: json!([user_op_json, self.entry_point]),
            id: 1,
        };

        let response = client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse<serde_json::Value> = response.json().await?;
        
        if let Some(error) = rpc_response.error {
            return Err(anyhow::anyhow!("RPC Error {}: {}", error.code, error.message));
        }

        let result = rpc_response.result.ok_or_else(|| anyhow::anyhow!("No result in response"))?;
        
        // Parse the gas estimation response
        let pre_verification_gas = U256::from_str_radix(
            result["preVerificationGas"].as_str().unwrap_or("0x0"), 
            16
        )?;
        let verification_gas_limit = U256::from_str_radix(
            result["verificationGasLimit"].as_str().unwrap_or("0x0"), 
            16
        )?;
        let call_gas_limit = U256::from_str_radix(
            result["callGasLimit"].as_str().unwrap_or("0x0"), 
            16
        )?;

        Ok(aa_sdk_rs::types::UserOperationGasEstimation {
            pre_verification_gas,
            verification_gas_limit,
            call_gas_limit,
            paymaster_verification_gas_limit: None,
        })
    }

    /// Convert UserOperationRequest to JSON format for RPC calls
    fn user_op_to_json(&self, user_op: &aa_sdk_rs::types::UserOperationRequest) -> Result<serde_json::Value> {
        let sender_str = user_op.sender
            .map(|addr| format!("0x{}", hex::encode(addr.as_slice())))
            .unwrap_or_else(|| "0x".to_string());
            
        let nonce_str = user_op.nonce
            .map(|n| format!("0x{:x}", n))
            .unwrap_or_else(|| "0x0".to_string());
            
        let call_data_str = user_op.call_data.as_ref()
            .map(|data| format!("0x{}", hex::encode(data)))
            .unwrap_or_else(|| "0x".to_string());
            
        let call_gas_limit_str = user_op.call_gas_limit
            .map(|gas| format!("0x{:x}", gas))
            .unwrap_or_else(|| "0x0".to_string());
            
        let verification_gas_limit_str = user_op.verification_gas_limit
            .map(|gas| format!("0x{:x}", gas))
            .unwrap_or_else(|| "0x0".to_string());
            
        let pre_verification_gas_str = user_op.pre_verification_gas
            .map(|gas| format!("0x{:x}", gas))
            .unwrap_or_else(|| "0x0".to_string());
            
        let max_fee_per_gas_str = user_op.max_fee_per_gas
            .map(|gas| format!("0x{:x}", gas))
            .unwrap_or_else(|| "0x0".to_string());
            
        let max_priority_fee_per_gas_str = user_op.max_priority_fee_per_gas
            .map(|gas| format!("0x{:x}", gas))
            .unwrap_or_else(|| "0x0".to_string());
            
        // Combine factory and factory_data into initCode
        let init_code_str = if let (Some(factory), Some(factory_data)) = (&user_op.factory, &user_op.factory_data) {
            format!("0x{}{}", hex::encode(factory.as_slice()), hex::encode(factory_data))
        } else {
            "0x".to_string()
        };
        
        let paymaster_data_str = user_op.paymaster_data.as_ref()
            .map(|data| format!("0x{}", hex::encode(data)))
            .unwrap_or_else(|| "0x".to_string());
            
        let signature_str = user_op.signature.as_ref()
            .map(|sig| format!("0x{}", hex::encode(sig)))
            .unwrap_or_else(|| "0x".to_string());

        Ok(json!({
            "sender": sender_str,
            "nonce": nonce_str,
            "initCode": init_code_str,
            "callData": call_data_str,
            "callGasLimit": call_gas_limit_str,
            "verificationGasLimit": verification_gas_limit_str,
            "preVerificationGas": pre_verification_gas_str,
            "maxFeePerGas": max_fee_per_gas_str,
            "maxPriorityFeePerGas": max_priority_fee_per_gas_str,
            "paymasterAndData": paymaster_data_str,
            "signature": signature_str
        }))
    }

    // Note: The following methods are deprecated in favor of aa-sdk-rs SmartAccountProvider
    // For new implementations, use SmartAccountProvider which provides:
    // - send_user_operation() for submitting operations
    // - estimate_user_operation_gas() for gas estimation
    // - fill_user_operation() for filling missing fields
    // - And many more aa-sdk-rs provider methods

    // DEPRECATED METHODS REMOVED
    // These methods were replaced with proper aa-sdk-rs SmartAccountProvider usage
    // Use SmartAccountProvider::send_user_operation for submitting operations
    // Use SmartAccountProvider::estimate_user_operation_gas for gas estimation
}

// Helper function removed - now using Alloy provider methods

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn test_bundler_client_creation() {
        let rpc_url = "http://localhost:8545".to_string();
        let entry_point = Address::from([1u8; 20]);
        let chain_id = U256::from(1u64);
        
        let client = BundlerClient::new(rpc_url.clone(), entry_point, chain_id);
        
        assert_eq!(client.rpc_url(), rpc_url);
        assert_eq!(client.entry_point(), entry_point);
        assert_eq!(client.chain_id(), chain_id);
    }

    // Removed tests for deprecated helper functions and JSON-RPC structures
    // aa-sdk-rs handles JSON-RPC communication internally

    #[test]
    fn test_chain_id_handling() {
        let rpc_url = "http://localhost:8545".to_string();
        let entry_point = Address::from([1u8; 20]);
        
        // Test different chain IDs
        let client_mainnet = BundlerClient::new(rpc_url.clone(), entry_point, U256::from(1u64));
        let client_polygon = BundlerClient::new(rpc_url.clone(), entry_point, U256::from(137u64));
        let client_arbitrum = BundlerClient::new(rpc_url, entry_point, U256::from(42161u64));
        
        assert_eq!(client_mainnet.chain_id(), U256::from(1u64));
        assert_eq!(client_polygon.chain_id(), U256::from(137u64));
        assert_eq!(client_arbitrum.chain_id(), U256::from(42161u64));
    }

    #[test]
    fn test_entry_point_handling() {
        let rpc_url = "http://localhost:8545".to_string();
        let chain_id = U256::from(1u64);
        
        let entry_point1 = Address::from([1u8; 20]);
        let entry_point2 = Address::from([2u8; 20]);
        
        let client1 = BundlerClient::new(rpc_url.clone(), entry_point1, chain_id);
        let client2 = BundlerClient::new(rpc_url, entry_point2, chain_id);
        
        assert_ne!(client1.entry_point(), client2.entry_point());
    }

    #[test]
    fn test_rpc_url_handling() {
        let entry_point = Address::from([1u8; 20]);
        let chain_id = U256::from(1u64);
        
        let rpc_url1 = "http://localhost:8545".to_string();
        let rpc_url2 = "https://mainnet.infura.io/v3/your-project-id".to_string();
        let rpc_url3 = "wss://mainnet.infura.io/ws/v3/your-project-id".to_string();
        
        let client1 = BundlerClient::new(rpc_url1.clone(), entry_point, chain_id);
        let client2 = BundlerClient::new(rpc_url2.clone(), entry_point, chain_id);
        let client3 = BundlerClient::new(rpc_url3.clone(), entry_point, chain_id);
        
        assert_eq!(client1.rpc_url(), &rpc_url1);
        assert_eq!(client2.rpc_url(), &rpc_url2);
        assert_eq!(client3.rpc_url(), &rpc_url3);
    }

    #[tokio::test]
    async fn test_provider_creation() {
        let rpc_url = "http://localhost:8545".to_string();
        let entry_point = Address::from([1u8; 20]);
        let chain_id = U256::from(1u64);
        
        let client = BundlerClient::new(rpc_url, entry_point, chain_id);
        
        // Test that provider creation works (though it may fail to connect)
        let _provider_result = client.create_provider().await;
        // We just test that the method can be called, not that it connects
    }
}
