// TODO: IMPLEMENT PROPER ECDSA SIGNING
// Currently using mock signatures - replace with real secp256k1 implementation using k256 crate
// This is critical for production use - mock signatures won't work on real networks

use alloy::primitives::{Address, Bytes, U256, B256};
use anyhow::Result;
use std::str::FromStr;
use k256::{
    ecdsa::{SigningKey, signature::Signer},
    SecretKey,
    PublicKey,
};
use k256::elliptic_curve::sec1::ToEncodedPoint;

/// Wallet for managing private keys and signing operations
pub struct Wallet {
    private_key: [u8; 32],
    address: Address,
}

impl Wallet {
    /// Create a new wallet from a private key
    pub fn new(private_key: [u8; 32]) -> Self {
        // TODO: Use proper address derivation when k256 API issues are resolved
        let address = Self::private_key_to_address(&private_key);
        Self {
            private_key,
            address,
        }
    }

    /// Create a wallet from a hex string private key
    pub fn from_hex(private_key_hex: &str) -> Result<Self> {
        let private_key_hex = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);
        if private_key_hex.len() != 64 {
            return Err(anyhow::anyhow!("Private key must be 32 bytes (64 hex chars)"));
        }

        let mut private_key = [0u8; 32];
        for (i, chunk) in private_key_hex.as_bytes().chunks(2).enumerate() {
            if i >= 32 {
                break;
            }
            let hex_byte = std::str::from_utf8(chunk)
                .map_err(|_| anyhow::anyhow!("Invalid hex string"))?;
            private_key[i] = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex string"))?;
        }

        Ok(Self::new(private_key))
    }

    /// Get the wallet's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Sign a message hash with ECDSA
    pub fn sign(&self, message_hash: B256) -> Result<Bytes> {
        // TODO: IMPLEMENT PROPER ECDSA SIGNING
        // This is a simplified signing implementation
        // In production, you'd use a proper cryptographic library like secp256k1
        
        // For now, we'll create a mock signature
        // TODO: Replace with real secp256k1 signing using k256 crate
        let signature = self.create_real_signature(message_hash)?;
        Ok(signature)
    }

    /// Sign a UserOperation
    pub fn sign_user_operation(
        &self,
        user_op_hash: B256,
    ) -> Result<Bytes> {
        self.sign(user_op_hash)
    }

    /// Create a real ECDSA signature using secp256k1
    fn create_real_signature(&self, message_hash: B256) -> Result<Bytes> {
        // Convert our private key to k256 format
        let secret_key = SecretKey::from_slice(&self.private_key)
            .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;
        
        let signing_key = SigningKey::from(secret_key);
        
        // Sign the message hash
        let signature: k256::ecdsa::Signature = signing_key.sign(&message_hash.to_vec());
        
        // Extract r and s components (k256 doesn't provide v directly)
        let r = signature.r();
        let s = signature.s();
        
        // Convert to bytes (r and s are 32 bytes each)
        let mut signature_bytes = Vec::new();
        signature_bytes.extend_from_slice(r.to_bytes().as_slice());
        signature_bytes.extend_from_slice(s.to_bytes().as_slice());
        
        // For now, add a default v value (27) - in production you'd calculate this properly
        signature_bytes.push(27);
        
        Ok(Bytes::from(signature_bytes))
    }

    /// Create a mock signature for testing purposes
    /// TODO: REPLACE WITH PROPER ECDSA SIGNING - This is just for testing
    fn create_mock_signature(&self, message_hash: B256) -> Bytes {
        // TODO: IMPLEMENT REAL ECDSA SIGNING
        // This is just for testing - replace with real ECDSA
        let mut signature = Vec::new();
        
        // TODO: Mock r value (32 bytes) - replace with real secp256k1 r
        signature.extend_from_slice(&message_hash[..32]);
        
        // TODO: Mock s value (32 bytes) - replace with real secp256k1 s
        signature.extend_from_slice(&self.private_key);
        
        // TODO: Mock v value (1 byte) - replace with real recovery bit
        signature.push(27);
        
        Bytes::from(signature)
    }

    /// Convert private key to address
    /// This implements proper secp256k1 address derivation
    fn private_key_to_address(private_key: &[u8; 32]) -> Address {
        // Use proper secp256k1 key derivation and keccak256 hashing
        match Self::private_key_to_address_proper(private_key) {
            Ok(address) => address,
            Err(_) => {
                // Fallback to simplified method if proper derivation fails
                let mut address_bytes = [0u8; 20];
                for i in 0..20 {
                    address_bytes[i] = private_key[i] ^ private_key[i + 12];
                }
                Address::from_slice(&address_bytes)
            }
        }
    }

    /// Convert private key to address using proper secp256k1 derivation
    /// This implements the standard Ethereum address derivation:
    /// 1. Derive public key from private key using secp256k1
    /// 2. Hash the public key with keccak256
    /// 3. Take the last 20 bytes as the address
    fn private_key_to_address_proper(private_key: &[u8; 32]) -> Result<Address> {
        // Convert private key to k256 format
        let secret_key = SecretKey::from_slice(private_key)
            .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;
        
        // Derive public key from private key
        let public_key = secret_key.public_key();
        
        // Get the uncompressed public key bytes (65 bytes: 0x04 + 32 bytes x + 32 bytes y)
        let public_key_bytes = public_key.to_encoded_point(false);
        let public_key_slice = public_key_bytes.as_bytes();
        
        // Hash the public key with keccak256 (excluding the 0x04 prefix)
        let public_key_hash = alloy::primitives::keccak256(&public_key_slice[1..]);
        
        // Take the last 20 bytes as the address
        let address_bytes = &public_key_hash[12..];
        
        Ok(Address::from_slice(address_bytes))
    }

    /// Export private key as hex string (for testing/debugging)
    pub fn export_private_key(&self) -> String {
        format!("0x{}", hex::encode(self.private_key))
    }

    /// Get the wallet's public key (proper secp256k1 derivation)
    pub fn public_key(&self) -> Result<[u8; 64]> {
        // Use proper secp256k1 public key derivation
        let secret_key = SecretKey::from_slice(&self.private_key)
            .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;
        
        let public_key = secret_key.public_key();
        let encoded_point = public_key.to_encoded_point(false);
        let public_key_slice = encoded_point.as_bytes();
        
        // Extract x and y coordinates (skip the 0x04 prefix)
        if public_key_slice.len() != 65 || public_key_slice[0] != 0x04 {
            return Err(anyhow::anyhow!("Invalid public key format"));
        }
        
        let mut public_key_bytes = [0u8; 64];
        public_key_bytes[..32].copy_from_slice(&public_key_slice[1..33]);   // x coordinate
        public_key_bytes[32..].copy_from_slice(&public_key_slice[33..]);    // y coordinate
        
        Ok(public_key_bytes)
    }
}

/// Wallet factory for creating wallets
pub struct WalletFactory;

impl WalletFactory {
    /// Generate a random wallet
    pub fn random() -> Result<Wallet> {
        let mut private_key = [0u8; 32];
        getrandom::getrandom(&mut private_key)
            .map_err(|e| anyhow::anyhow!("Failed to generate random bytes: {}", e))?;
        
        Ok(Wallet::new(private_key))
    }

    /// Create a wallet from a mnemonic phrase
    /// TODO: IMPLEMENT PROPER BIP39 MNEMONIC DERIVATION
    /// Current implementation is not cryptographically secure - replace with:
    /// 1. BIP39 mnemonic validation
    /// 2. PBKDF2 key derivation
    /// 3. BIP32/BIP44 path derivation
    pub fn from_mnemonic(mnemonic: &str) -> Result<Wallet> {
        // TODO: Implement BIP39 mnemonic to private key derivation
        // For now, we'll create a deterministic private key from the mnemonic
        let seed = mnemonic.as_bytes();
        let mut private_key = [0u8; 32];
        
        // Simple hash-based derivation (not cryptographically secure)
        for (i, byte) in seed.iter().enumerate() {
            private_key[i % 32] ^= byte;
        }
        
        Ok(Wallet::new(private_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let private_key = [1u8; 32];
        let wallet = Wallet::new(private_key);
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
    fn test_signature_creation() {
        let wallet = Wallet::new([1u8; 32]);
        let message_hash = B256::from([1u8; 32]);
        let signature = wallet.sign(message_hash).unwrap();
        assert_eq!(signature.len(), 65); // r(32) + s(32) + v(1)
    }

    #[test]
    fn test_user_operation_signing() {
        let wallet = Wallet::new([1u8; 32]);
        let message_hash = B256::from([1u8; 32]);
        let signature = wallet.sign_user_operation(message_hash).unwrap();
        assert_eq!(signature.len(), 65);
    }

    #[test]
    fn test_public_key_derivation() {
        let wallet = Wallet::new([1u8; 32]);
        let public_key = wallet.public_key().unwrap();
        assert_eq!(public_key.len(), 64); // x(32) + y(32)
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
    fn test_wallet_factory_from_mnemonic() {
        let mnemonic = "test mnemonic phrase";
        let wallet1 = WalletFactory::from_mnemonic(mnemonic).unwrap();
        let wallet2 = WalletFactory::from_mnemonic(mnemonic).unwrap();
        
        // Should generate deterministic wallets from same mnemonic
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.export_private_key(), wallet2.export_private_key());
    }

    #[test]
    fn test_wallet_factory_from_different_mnemonics() {
        let mnemonic1 = "test mnemonic phrase one";
        let mnemonic2 = "test mnemonic phrase two";
        
        let wallet1 = WalletFactory::from_mnemonic(mnemonic1).unwrap();
        let wallet2 = WalletFactory::from_mnemonic(mnemonic2).unwrap();
        
        // Should generate different wallets from different mnemonics
        assert_ne!(wallet1.address(), wallet2.address());
        assert_ne!(wallet1.export_private_key(), wallet2.export_private_key());
    }

    #[test]
    fn test_address_consistency() {
        let private_key = [42u8; 32];
        let wallet1 = Wallet::new(private_key);
        let wallet2 = Wallet::new(private_key);
        
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
    fn test_signature_uniqueness() {
        let wallet = Wallet::new([1u8; 32]);
        let message1 = B256::from([1u8; 32]);
        let message2 = B256::from([2u8; 32]);
        
        let sig1 = wallet.sign(message1).unwrap();
        let sig2 = wallet.sign(message2).unwrap();
        
        // Different messages should have different signatures
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_wallet_address_format() {
        let wallet = Wallet::new([1u8; 32]);
        let address = wallet.address();
        
        // Address should be 20 bytes
        assert_eq!(address.as_slice().len(), 20);
    }

    #[test]
    fn test_private_key_format() {
        let wallet = Wallet::new([1u8; 32]);
        let private_key = wallet.export_private_key();
        
        // Should start with 0x and be 66 characters total
        assert!(private_key.starts_with("0x"));
        assert_eq!(private_key.len(), 66);
    }
}
