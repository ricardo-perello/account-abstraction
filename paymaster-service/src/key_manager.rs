use secp256k1::{SecretKey, PublicKey, Secp256k1};
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
        let keys = self.keys.read().await;
        let secret_key = keys
            .get(verifier_name)
            .ok_or(KeyManagerError::VerifierNotFound)?;
        
        let signature = self.secp.sign_ecdsa(
            &secp256k1::Message::from_slice(message).unwrap(),
            secret_key,
        );
        
        Ok(signature.serialize_compact().to_vec())
    }
    
    pub async fn get_verifier_count(&self) -> usize {
        let keys = self.keys.read().await;
        keys.len()
    }
}
