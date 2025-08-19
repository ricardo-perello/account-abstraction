// Refactored to use aa-sdk-rs SmartAccountProvider functionality
// This replaces custom bundler implementation with proper SDK provider

use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use url::Url;

// Re-export aa-sdk-rs provider types
pub use aa_sdk_rs::provider::{SmartAccountProvider, SmartAccountProviderTrait};

// Keep compatibility types for existing API
use crate::userop::{UserOperationResponse, GasEstimate};

/// Modern bundler client wrapper that can create aa-sdk-rs providers
/// This provides compatibility while enabling use of aa-sdk-rs functionality
pub struct BundlerClient {
    rpc_url: String,
    entry_point: Address,
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
    pub fn entry_point(&self) -> Address {
        self.entry_point
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> U256 {
        self.chain_id
    }

    /// Create an Alloy provider from this bundler client configuration
    pub async fn create_provider(&self) -> Result<impl Provider<alloy::network::Ethereum>> {
        let url = Url::parse(&self.rpc_url)?;
        let provider = ProviderBuilder::new().on_http(url);
        Ok(provider)
    }

    // Note: The following methods are deprecated in favor of aa-sdk-rs SmartAccountProvider
    // For new implementations, use SmartAccountProvider which provides:
    // - send_user_operation() for submitting operations
    // - estimate_user_operation_gas() for gas estimation
    // - fill_user_operation() for filling missing fields
    // - And many more aa-sdk-rs provider methods

    /// DEPRECATED: Use SmartAccountProvider::send_user_operation instead
    /// This is kept for backward compatibility
    pub async fn submit_user_operation_legacy(
        &self,
        _user_op: &crate::userop::UserOperation,
    ) -> Result<UserOperationResponse> {
        Err(anyhow::anyhow!(
            "This method is deprecated. Use aa-sdk-rs SmartAccountProvider::send_user_operation instead. \
            Call create_provider() to get an Alloy provider, then wrap it with a SmartAccount."
        ))
    }

    /// DEPRECATED: Use SmartAccountProvider::estimate_user_operation_gas instead
    /// This is kept for backward compatibility
    pub async fn estimate_user_operation_gas_legacy(
        &self,
        _user_op: &crate::userop::UserOperation,
    ) -> Result<GasEstimate> {
        Err(anyhow::anyhow!(
            "This method is deprecated. Use aa-sdk-rs SmartAccountProvider::estimate_user_operation_gas instead. \
            Call create_provider() to get an Alloy provider, then wrap it with a SmartAccount."
        ))
    }

    /// Get the current nonce for an account using Alloy provider
    pub async fn get_nonce(&self, account: Address) -> Result<U256> {
        let provider = self.create_provider().await?;
        let nonce = provider.get_transaction_count(account).await?;
        Ok(U256::from(nonce))
    }

    /// Get the current gas price using Alloy provider
    pub async fn get_gas_price(&self) -> Result<U256> {
        let provider = self.create_provider().await?;
        let gas_price = provider.get_gas_price().await?;
        Ok(U256::from(gas_price))
    }
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
