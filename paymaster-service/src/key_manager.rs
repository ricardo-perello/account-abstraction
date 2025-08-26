use secp256k1::{SecretKey, Secp256k1};
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::Config;

#[derive(Debug)]
pub enum KeyManagerError {
    VerifierNotFound,
    InvalidKeyFormat,
    SigningError,
}

impl std::fmt::Display for KeyManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyManagerError::VerifierNotFound => write!(f, "Verifier not found"),
            KeyManagerError::InvalidKeyFormat => write!(f, "Invalid key format"),
            KeyManagerError::SigningError => write!(f, "Signing error"),
        }
    }
}

impl std::error::Error for KeyManagerError {}

pub struct KeyManager {
    keys: RwLock<HashMap<String, SecretKey>>,
    secp: Secp256k1<secp256k1::All>,
}

impl KeyManager {
    pub fn new(config: &Config) -> Self {
        let mut keys = HashMap::new();
        
        // Load verifier keys from configuration
        for (name, key_hex) in &config.verifier_keys {
            if let Ok(key_bytes) = hex::decode(key_hex) {
                if let Ok(secret_key) = SecretKey::from_slice(&key_bytes) {
                    keys.insert(name.clone(), secret_key);
                }
            }
        }
        
        Self {
            keys: RwLock::new(keys),
            secp: Secp256k1::new(),
        }
    }
    
    pub async fn sign_sponsorship(
        &self,
        verifier_name: &str,
        message: &[u8],
    ) -> Result<Vec<u8>, KeyManagerError> {
        use sha3::{Digest, Keccak256};
        
        let keys = self.keys.read().await;
        let secret_key = keys
            .get(verifier_name)
            .ok_or(KeyManagerError::VerifierNotFound)?;
        
        // Hash the message first (Ethereum uses Keccak256)
        let hash = Keccak256::digest(message);
        
        let message_obj = secp256k1::Message::from_digest_slice(&hash)
            .map_err(|_| KeyManagerError::SigningError)?;
        
        let signature = self.secp.sign_ecdsa(&message_obj, secret_key);
        
        Ok(signature.serialize_compact().to_vec())
    }
    
    // New method for signing EIP-191 messages with recovery byte
    pub async fn sign_eip191_message(
        &self,
        verifier_name: &str,
        message: &[u8],
    ) -> Result<Vec<u8>, KeyManagerError> {
        use secp256k1::Message;
        
        let keys = self.keys.read().await;
        let secret_key = keys
            .get(verifier_name)
            .ok_or(KeyManagerError::VerifierNotFound)?;
        
        // The message is already the EIP-191 digest - sign it directly (no double-hashing)
        let message_obj = Message::from_digest_slice(message)
            .map_err(|_| KeyManagerError::SigningError)?;
        
        // Sign and get recoverable signature
        let signature = self.secp.sign_ecdsa_recoverable(&message_obj, secret_key);
        let (recovery_id, compact_sig) = signature.serialize_compact();
        
        // Convert to r + s + v format (as expected by contract)
        let mut sig_bytes = Vec::with_capacity(65);
        
        // Split compact signature into r (32 bytes) and s (32 bytes)
        let r = &compact_sig[0..32];
        let s = &compact_sig[32..64];
        
        // Convert recovery ID to Solidity format (27 + recovery_id)
        let v = 27 + recovery_id.to_i32() as u8;
        
        // Build signature as r + s + v (matches abi.encodePacked(r, s, v))
        sig_bytes.extend_from_slice(r);    // r: 32 bytes
        sig_bytes.extend_from_slice(s);    // s: 32 bytes  
        sig_bytes.push(v);                 // v: 1 byte (27 or 28)
        
        Ok(sig_bytes)
    }
    
    pub async fn get_verifier_count(&self) -> usize {
        let keys = self.keys.read().await;
        keys.len()
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> crate::Config {
        let mut verifier_keys = HashMap::new();
        verifier_keys.insert("default".to_string(), "0000000000000000000000000000000000000000000000000000000000000001".to_string());
        
        crate::Config {
            verifier_keys,
            api_keys: HashMap::new(),
            server_port: 3000,
            log_level: "info".to_string(),
            chain_id: Some(1),
            paymaster_address: Some("0x0000000000000000000000000000000000000000".to_string()),
        }
    }

    #[tokio::test]
    async fn test_key_manager_initialization() {
        let config = create_test_config();
        let key_manager = KeyManager::new(&config);
        
        // Check that keys were loaded
        assert_eq!(key_manager.get_verifier_count().await, 1);
    }

    #[tokio::test]
    async fn test_successful_signing() {
        let config = create_test_config();
        let key_manager = KeyManager::new(&config);
        
        let message = b"test message to sign";
        let signature = key_manager.sign_sponsorship("default", message).await;
        
        assert!(signature.is_ok());
        let sig_bytes = signature.unwrap();
        assert_eq!(sig_bytes.len(), 64); // Compact signature should be 64 bytes
    }

    #[tokio::test]
    async fn test_verifier_not_found() {
        let config = create_test_config();
        let key_manager = KeyManager::new(&config);
        
        let message = b"test message";
        let result = key_manager.sign_sponsorship("nonexistent_verifier", message).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyManagerError::VerifierNotFound));
    }
}
