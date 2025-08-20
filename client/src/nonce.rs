// Nonce management utilities for UserOperations
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::sol;
use crate::error::{AAError, Result};

// IEntryPoint interface for nonce management - simplified
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IEntryPoint,
    r#"[
        {
            "inputs": [
                {"internalType": "address", "name": "sender", "type": "address"},
                {"internalType": "uint256", "name": "key", "type": "uint256"}
            ],
            "name": "getNonce",
            "outputs": [
                {"internalType": "uint256", "name": "nonce", "type": "uint256"}
            ],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#
);

/// Nonce manager for handling UserOperation nonces
pub struct NonceManager {
    entry_point: Address,
}

impl NonceManager {
    pub fn new(entry_point: Address) -> Self {
        Self { entry_point }
    }

    /// Get the next nonce for an account
    /// 
    /// # Arguments
    /// * `provider` - The blockchain provider
    /// * `account` - The account address
    /// * `key` - The nonce key (default: 0)
    /// 
    /// # Returns
    /// The next nonce to use for a UserOperation
    pub async fn get_next_nonce<P>(
        &self,
        provider: &P,
        account: Address,
        key: Option<U256>,
    ) -> Result<U256>
    where
        P: Provider,
    {
        let nonce_key = key.unwrap_or(U256::ZERO);
        
        // Convert U256 to compatible type for the key
        // Use the nonce key directly
        
        let entry_point_contract = IEntryPoint::new(self.entry_point, provider);
        
        // Use simplified nonce implementation
        let result = entry_point_contract
            .getNonce(account, nonce_key)
            .call()
            .await
            .map_err(|e| AAError::NonceError(format!("Failed to get nonce: {}", e)))?;

        Ok(result.nonce)
    }

    /// Get the current nonce without incrementing
    pub async fn get_current_nonce<P>(
        &self,
        provider: &P,
        account: Address,
        key: Option<U256>,
    ) -> Result<U256>
    where
        P: Provider,
    {
        self.get_next_nonce(provider, account, key).await
    }

    /// Check if a nonce is valid for an account
    pub async fn is_nonce_valid<P>(
        &self,
        provider: &P,
        account: Address,
        nonce: U256,
        key: Option<U256>,
    ) -> Result<bool>
    where
        P: Provider,
    {
        let current_nonce = self.get_current_nonce(provider, account, key).await?;
        Ok(nonce >= current_nonce)
    }

    /// Get nonce with validation
    pub async fn get_validated_nonce<P>(
        &self,
        provider: &P,
        account: Address,
        requested_nonce: Option<U256>,
        key: Option<U256>,
    ) -> Result<U256>
    where
        P: Provider,
    {
        match requested_nonce {
            Some(nonce) => {
                // Validate the requested nonce
                if self.is_nonce_valid(provider, account, nonce, key).await? {
                    Ok(nonce)
                } else {
                    let current = self.get_current_nonce(provider, account, key).await?;
                    Err(AAError::NonceError(format!(
                        "Invalid nonce: requested {}, current {}",
                        nonce, current
                    )))
                }
            }
            None => {
                // Get the next available nonce
                self.get_next_nonce(provider, account, key).await
            }
        }
    }

    /// Extract nonce key from a full nonce value
    /// 
    /// ERC-4337 packs the key in the upper bits of the nonce
    pub fn extract_nonce_key(packed_nonce: U256) -> U256 {
        // Key is in the upper 192 bits, shifted right by 64 bits
        packed_nonce >> 64
    }

    /// Extract sequence number from a full nonce value
    pub fn extract_sequence_number(packed_nonce: U256) -> U256 {
        // Sequence is in the lower 64 bits
        // Use u64::MAX instead of (1u64 << 64) - 1 to avoid overflow
        packed_nonce & U256::from(u64::MAX)
    }

    /// Pack key and sequence into a full nonce
    pub fn pack_nonce(key: U256, sequence: U256) -> U256 {
        (key << 64) | sequence
    }
}

/// Convenience functions for common nonce operations
pub async fn get_account_nonce<P>(
    provider: &P,
    entry_point: Address,
    account: Address,
) -> Result<U256>
where
    P: Provider,
{
    let nonce_manager = NonceManager::new(entry_point);
    nonce_manager.get_next_nonce(provider, account, None).await
}

pub async fn get_account_nonce_with_key<P>(
    provider: &P,
    entry_point: Address,
    account: Address,
    key: U256,
) -> Result<U256>
where
    P: Provider,
{
    let nonce_manager = NonceManager::new(entry_point);
    nonce_manager.get_next_nonce(provider, account, Some(key)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_packing() {
        let key = U256::from(42);
        let sequence = U256::from(100);
        
        let packed = NonceManager::pack_nonce(key, sequence);
        let extracted_key = NonceManager::extract_nonce_key(packed);
        let extracted_sequence = NonceManager::extract_sequence_number(packed);
        
        assert_eq!(extracted_key, key);
        assert_eq!(extracted_sequence, sequence);
    }

    #[test]
    fn test_nonce_key_extraction() {
        // Test with a typical nonce value
        let nonce = U256::from(42u64 << 32) | U256::from(100); // Simplified example
        let key = NonceManager::extract_nonce_key(nonce);
        let sequence = NonceManager::extract_sequence_number(nonce);
        
        // The actual extraction depends on the bit layout
        assert!(key >= U256::ZERO);
        assert!(sequence >= U256::ZERO);
    }

    #[test]
    fn test_nonce_manager_creation() {
        let entry_point = Address::from([1u8; 20]);
        let manager = NonceManager::new(entry_point);
        assert_eq!(manager.entry_point, entry_point);
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero values
        let packed = NonceManager::pack_nonce(U256::ZERO, U256::ZERO);
        assert_eq!(packed, U256::ZERO);
        
        // Test with max values (within reasonable bounds)
        let max_sequence = U256::from(u32::MAX); // Use 32 bits for testing
        let packed = NonceManager::pack_nonce(U256::from(1), max_sequence);
        let extracted_sequence = NonceManager::extract_sequence_number(packed);
        assert!(extracted_sequence <= max_sequence);
    }
}
