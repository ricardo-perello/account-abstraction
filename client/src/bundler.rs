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
