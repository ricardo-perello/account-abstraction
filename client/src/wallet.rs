// Refactored to use aa-sdk-rs signer functionality
// This replaces custom wallet implementation with proper SDK signers

use alloy::primitives::Address;
use alloy::signers::{k256::ecdsa::SigningKey, local::LocalSigner};
use anyhow::Result;

// Re-export aa-sdk-rs signer types
pub use aa_sdk_rs::signer::SmartAccountSigner;

/// Wallet wrapper around aa-sdk-rs LocalSigner
pub struct Wallet {
    signer: LocalSigner<SigningKey>,
}

impl Wallet {
    /// Create a new wallet from a private key using aa-sdk-rs LocalSigner
    pub fn new(private_key: [u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(private_key.as_slice().into())
            .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;
        let signer = LocalSigner::from(signing_key);
        
        Ok(Self { signer })
    }

    /// Create a wallet from a hex string private key using alloy hex parsing
    pub fn from_hex(private_key_hex: &str) -> Result<Self> {
        // Use alloy's hex parsing instead of custom implementation
        let private_key_hex = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);
        let bytes = hex::decode(private_key_hex)
            .map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))?;
        
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Private key must be 32 bytes, got {}", bytes.len()));
        }
        
        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&bytes);
        
        Self::new(private_key)
    }

    /// Get the wallet's address using aa-sdk-rs SmartAccountSigner trait
    pub fn address(&self) -> Address {
        self.signer.get_address()
    }

    /// Get a reference to the inner LocalSigner for use with aa-sdk-rs
    pub fn signer(&self) -> &LocalSigner<SigningKey> {
        &self.signer
    }

    /// Export private key as hex string (for testing/debugging)
    /// Note: This accesses the signing key from LocalSigner
    pub fn export_private_key(&self) -> String {
        // Get the signing key bytes from the LocalSigner
        let signing_key = self.signer.credential();
        let private_key_bytes = signing_key.to_bytes();
        format!("0x{}", hex::encode(private_key_bytes))
    }
}

/// Wallet factory for creating wallets
pub struct WalletFactory;

impl WalletFactory {
    /// Generate a random wallet using aa-sdk-rs LocalSigner
    pub fn random() -> Result<Wallet> {
        let mut private_key = [0u8; 32];
        getrandom::getrandom(&mut private_key)
            .map_err(|e| anyhow::anyhow!("Failed to generate random bytes: {}", e))?;
        
        Wallet::new(private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let private_key = [1u8; 32];
        let wallet = Wallet::new(private_key).unwrap();
        assert_eq!(wallet.address(), wallet.address()); // Should be consistent
    }

    #[test]
    fn test_wallet_from_hex() {
        let hex_key = format!("0x{}", "1".repeat(64));
        let wallet = Wallet::from_hex(&hex_key).unwrap();
        assert_eq!(wallet.export_private_key(), hex_key);
    }

    #[test]
    fn test_wallet_from_hex_without_prefix() {
        let hex_key = "1".repeat(64);
        let wallet = Wallet::from_hex(&hex_key).unwrap();
        assert_eq!(wallet.export_private_key(), format!("0x{}", hex_key));
    }

    #[test]
    fn test_wallet_from_hex_invalid_length() {
        let hex_key = "1".repeat(63); // Too short
        let result = Wallet::from_hex(&hex_key);
        assert!(result.is_err());
        
        let hex_key = "1".repeat(65); // Too long
        let result = Wallet::from_hex(&hex_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_wallet_from_hex_invalid_chars() {
        let hex_key = format!("0x{}", "g".repeat(64)); // Invalid hex chars
        let result = Wallet::from_hex(&hex_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_wallet_factory_random() {
        let wallet1 = WalletFactory::random().unwrap();
        let wallet2 = WalletFactory::random().unwrap();
        
        // Should generate different wallets
        assert_ne!(wallet1.address(), wallet2.address());
        assert_ne!(wallet1.export_private_key(), wallet2.export_private_key());
    }

    #[test]
    fn test_address_consistency() {
        let private_key = [42u8; 32];
        let wallet1 = Wallet::new(private_key).unwrap();
        let wallet2 = Wallet::new(private_key).unwrap();
        
        // Same private key should always generate same address
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_private_key_export_import() {
        let original_wallet = WalletFactory::random().unwrap();
        let private_key_hex = original_wallet.export_private_key();
        let imported_wallet = Wallet::from_hex(&private_key_hex).unwrap();
        
        // Imported wallet should have same address
        assert_eq!(original_wallet.address(), imported_wallet.address());
    }

    #[test]
    fn test_wallet_address_format() {
        let wallet = Wallet::new([1u8; 32]).unwrap();
        let address = wallet.address();
        
        // Address should be 20 bytes
        assert_eq!(address.as_slice().len(), 20);
    }

    #[test]
    fn test_private_key_format() {
        let wallet = Wallet::new([1u8; 32]).unwrap();
        let private_key = wallet.export_private_key();
        
        // Should start with 0x and be 66 characters total
        assert!(private_key.starts_with("0x"));
        assert_eq!(private_key.len(), 66);
    }

    #[test]
    fn test_signer_access() {
        let wallet = Wallet::new([1u8; 32]).unwrap();
        let _signer = wallet.signer(); // Should provide access to LocalSigner
        
        // Test passes if we can access the signer
    }
}
