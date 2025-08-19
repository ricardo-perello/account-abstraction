use crate::userop::{UserOperation, UserOperationResponse, GasEstimate};
use alloy::primitives::{Address, U256};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client for interacting with bundlers and RPC providers
pub struct BundlerClient {
    client: Client,
    rpc_url: String,
    entry_point: Address,
    chain_id: U256,
}

/// JSON-RPC request structure
#[derive(Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: String,
    id: u64,
    method: String,
    params: T,
}

/// JSON-RPC response structure
#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    id: u64,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl BundlerClient {
    /// Create a new bundler client
    pub fn new(rpc_url: String, entry_point: Address, chain_id: U256) -> Self {
        Self {
            client: Client::new(),
            rpc_url,
            entry_point,
            chain_id,
        }
    }

    /// Submit a UserOperation to the bundler
    pub async fn submit_user_operation(
        &self,
        user_op: &UserOperation,
    ) -> Result<UserOperationResponse> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_sendUserOperation".to_string(),
            params: vec![
                serde_json::to_value(user_op)?,
                serde_json::to_value(self.entry_point)?,
            ],
        };

        let response: JsonRpcResponse<String> = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("RPC Error: {}", error.message));
        }

        let user_op_hash = response.result.ok_or_else(|| {
            anyhow::anyhow!("No result in RPC response")
        })?;

        Ok(UserOperationResponse {
            user_op_hash,
            error: None,
        })
    }

    /// Get gas estimates for a UserOperation
    pub async fn estimate_user_operation_gas(
        &self,
        user_op: &UserOperation,
    ) -> Result<GasEstimate> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_estimateUserOperationGas".to_string(),
            params: vec![
                serde_json::to_value(user_op)?,
                serde_json::to_value(self.entry_point)?,
            ],
        };

        let response: JsonRpcResponse<HashMap<String, String>> = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("RPC Error: {}", error.message));
        }

        let result = response.result.ok_or_else(|| {
            anyhow::anyhow!("No result in RPC response")
        })?;

        let pre_verification_gas = from_str_radix(
            result.get("preVerificationGas").unwrap_or(&"0".to_string()),
            16u32,
        )?;
        let verification_gas_limit = from_str_radix(
            result.get("verificationGasLimit").unwrap_or(&"0".to_string()),
            16u32,
        )?;
        let call_gas_limit = from_str_radix(
            result.get("callGasLimit").unwrap_or(&"0".to_string()),
            16u32,
        )?;

        Ok(GasEstimate {
            pre_verification_gas,
            verification_gas_limit,
            call_gas_limit,
        })
    }

    /// Get the current nonce for an account
    pub async fn get_nonce(&self, account: Address) -> Result<U256> {
        let request: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_getTransactionCount".to_string(),
            params: vec![
                serde_json::to_value(account)?,
                serde_json::to_value("latest")?,
            ],
        };

        let response: JsonRpcResponse<String> = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("RPC Error: {}", error.message));
        }

        let nonce_hex = response.result.ok_or_else(|| {
            anyhow::anyhow!("No result in RPC response")
        })?;

        let nonce = from_str_radix(&nonce_hex[2..], 16)?;
        Ok(nonce)
    }

    /// Get the current gas price
    pub async fn get_gas_price(&self) -> Result<U256> {
        let request: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_gasPrice".to_string(),
            params: vec![],
        };

        let response: JsonRpcResponse<String> = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("RPC Error: {}", error.message));
        }

        let gas_price_hex = response.result.ok_or_else(|| {
            anyhow::anyhow!("No result in RPC response")
        })?;

        let gas_price = from_str_radix(&gas_price_hex[2..], 16)?;
        Ok(gas_price)
    }
}

// Helper function to parse hex strings to U256
fn from_str_radix(s: &str, radix: u32) -> Result<U256> {
    if s.starts_with("0x") {
        U256::from_str_radix(&s[2..], radix.into())
            .map_err(|e| anyhow::anyhow!("Failed to parse hex: {}", e))
    } else {
        U256::from_str_radix(s, radix.into())
            .map_err(|e| anyhow::anyhow!("Failed to parse hex: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::userop::UserOperation;
    use alloy::primitives::Address;

    #[test]
    fn test_bundler_client_creation() {
        let rpc_url = "http://localhost:8545".to_string();
        let entry_point = Address::from([1u8; 20]);
        let chain_id = U256::from(1u64);
        
        let client = BundlerClient::new(rpc_url.clone(), entry_point, chain_id);
        
        assert_eq!(client.rpc_url, rpc_url);
        assert_eq!(client.entry_point, entry_point);
        assert_eq!(client.chain_id, chain_id);
    }

    #[test]
    fn test_from_str_radix_with_0x_prefix() {
        let result = from_str_radix("0x1a", 16).unwrap();
        assert_eq!(result, U256::from(26u64));
    }

    #[test]
    fn test_from_str_radix_without_0x_prefix() {
        let result = from_str_radix("1a", 16).unwrap();
        assert_eq!(result, U256::from(26u64));
    }

    #[test]
    fn test_from_str_radix_decimal() {
        let result = from_str_radix("42", 10).unwrap();
        assert_eq!(result, U256::from(42u64));
    }

    #[test]
    fn test_from_str_radix_zero() {
        let result = from_str_radix("0", 16).unwrap();
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn test_from_str_radix_large_number() {
        let result = from_str_radix("0xffffffffffffffff", 16).unwrap();
        assert_eq!(result, U256::from(u64::MAX));
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_test".to_string(),
            params: vec!["param1".to_string(), "param2".to_string()],
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("2.0"));
        assert!(serialized.contains("eth_test"));
        assert!(serialized.contains("param1"));
        assert!(serialized.contains("param2"));
    }

    #[test]
    fn test_json_rpc_response_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1234"
        }"#;
        
        let response: JsonRpcResponse<String> = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert_eq!(response.result, Some("0x1234".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_error_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }"#;
        
        let response: JsonRpcResponse<String> = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[test]
    fn test_gas_estimate_parsing() {
        let mut gas_data = HashMap::new();
        gas_data.insert("preVerificationGas".to_string(), "0x186a0".to_string()); // 100000
        gas_data.insert("verificationGasLimit".to_string(), "0x30d40".to_string()); // 200000
        gas_data.insert("callGasLimit".to_string(), "0x4e20".to_string()); // 20000
        
        // Test the parsing logic that would be used in estimate_user_operation_gas
        let pre_verification_gas = from_str_radix(
            gas_data.get("preVerificationGas").unwrap(),
            16u32,
        ).unwrap();
        let verification_gas_limit = from_str_radix(
            gas_data.get("verificationGasLimit").unwrap(),
            16u32,
        ).unwrap();
        let call_gas_limit = from_str_radix(
            gas_data.get("callGasLimit").unwrap(),
            16u32,
        ).unwrap();
        
        assert_eq!(pre_verification_gas, U256::from(100000u64));
        assert_eq!(verification_gas_limit, U256::from(200000u64));
        assert_eq!(call_gas_limit, U256::from(20000u64));
    }

    #[test]
    fn test_gas_estimate_parsing_missing_fields() {
        let mut gas_data = HashMap::new();
        gas_data.insert("preVerificationGas".to_string(), "0x186a0".to_string());
        // Missing verificationGasLimit and callGasLimit
        
        let pre_verification_gas = from_str_radix(
            gas_data.get("preVerificationGas").unwrap(),
            16u32,
        ).unwrap();
        let verification_gas_limit = from_str_radix(
            gas_data.get("verificationGasLimit").unwrap_or(&"0".to_string()),
            16u32,
        ).unwrap();
        let call_gas_limit = from_str_radix(
            gas_data.get("callGasLimit").unwrap_or(&"0".to_string()),
            16u32,
        ).unwrap();
        
        assert_eq!(pre_verification_gas, U256::from(100000u64));
        assert_eq!(verification_gas_limit, U256::ZERO);
        assert_eq!(call_gas_limit, U256::ZERO);
    }

    #[test]
    fn test_hex_string_parsing_edge_cases() {
        // Test various hex string formats
        assert_eq!(from_str_radix("0x0", 16).unwrap(), U256::ZERO);
        assert_eq!(from_str_radix("0", 16).unwrap(), U256::ZERO);
        assert_eq!(from_str_radix("0x1", 16).unwrap(), U256::from(1u64));
        assert_eq!(from_str_radix("1", 16).unwrap(), U256::from(1u64));
        assert_eq!(from_str_radix("0xff", 16).unwrap(), U256::from(255u64));
        assert_eq!(from_str_radix("ff", 16).unwrap(), U256::from(255u64));
    }

    #[test]
    fn test_chain_id_handling() {
        let rpc_url = "http://localhost:8545".to_string();
        let entry_point = Address::from([1u8; 20]);
        
        // Test different chain IDs
        let client_mainnet = BundlerClient::new(rpc_url.clone(), entry_point, U256::from(1u64));
        let client_polygon = BundlerClient::new(rpc_url.clone(), entry_point, U256::from(137u64));
        let client_arbitrum = BundlerClient::new(rpc_url, entry_point, U256::from(42161u64));
        
        assert_eq!(client_mainnet.chain_id, U256::from(1u64));
        assert_eq!(client_polygon.chain_id, U256::from(137u64));
        assert_eq!(client_arbitrum.chain_id, U256::from(42161u64));
    }

    #[test]
    fn test_entry_point_handling() {
        let rpc_url = "http://localhost:8545".to_string();
        let chain_id = U256::from(1u64);
        
        let entry_point1 = Address::from([1u8; 20]);
        let entry_point2 = Address::from([2u8; 20]);
        
        let client1 = BundlerClient::new(rpc_url.clone(), entry_point1, chain_id);
        let client2 = BundlerClient::new(rpc_url, entry_point2, chain_id);
        
        assert_ne!(client1.entry_point, client2.entry_point);
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
        
        assert_eq!(client1.rpc_url, rpc_url1);
        assert_eq!(client2.rpc_url, rpc_url2);
        assert_eq!(client3.rpc_url, rpc_url3);
    }
}
