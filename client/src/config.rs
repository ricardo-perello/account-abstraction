// Network configuration for different chains
use alloy::primitives::Address;
use crate::error::{AAError, Result};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: &'static str,
    pub chain_id: u64,
    pub entry_point: Address,
    pub factory: Address,
    pub rpc_url_template: &'static str,
    pub bundler_url_template: Option<&'static str>,
}

impl NetworkConfig {
    pub fn mainnet() -> Self {
        Self {
            name: "Ethereum Mainnet",
            chain_id: 1,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::ZERO, // TODO: Deploy to mainnet
            rpc_url_template: "https://eth-mainnet.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://eth-mainnet.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn sepolia() -> Self {
        Self {
            name: "Sepolia Testnet",
            chain_id: 11155111,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::from_str("0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3").unwrap(),
            rpc_url_template: "https://eth-sepolia.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://eth-sepolia.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn goerli() -> Self {
        Self {
            name: "Goerli Testnet",
            chain_id: 5,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::ZERO, // TODO: Deploy to goerli
            rpc_url_template: "https://eth-goerli.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://eth-goerli.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn polygon() -> Self {
        Self {
            name: "Polygon Mainnet",
            chain_id: 137,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::ZERO, // TODO: Deploy to polygon
            rpc_url_template: "https://polygon-mainnet.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://polygon-mainnet.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn polygon_mumbai() -> Self {
        Self {
            name: "Polygon Mumbai",
            chain_id: 80001,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::ZERO, // TODO: Deploy to mumbai
            rpc_url_template: "https://polygon-mumbai.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://polygon-mumbai.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn arbitrum() -> Self {
        Self {
            name: "Arbitrum One",
            chain_id: 42161,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::ZERO, // TODO: Deploy to arbitrum
            rpc_url_template: "https://arb-mainnet.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://arb-mainnet.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn optimism() -> Self {
        Self {
            name: "Optimism",
            chain_id: 10,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::ZERO, // TODO: Deploy to optimism
            rpc_url_template: "https://opt-mainnet.g.alchemy.com/v2/{api_key}",
            bundler_url_template: Some("https://opt-mainnet.g.alchemy.com/v2/{api_key}"),
        }
    }

    pub fn anvil() -> Self {
        Self {
            name: "Anvil Local",
            chain_id: 31337,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap(),
            rpc_url_template: "http://localhost:8545",
            bundler_url_template: None, // Use same as RPC for local testing
        }
    }

    pub fn hardhat() -> Self {
        Self {
            name: "Hardhat Local",
            chain_id: 31337,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap(),
            rpc_url_template: "http://localhost:8545",
            bundler_url_template: None,
        }
    }

    pub fn get_rpc_url(&self, api_key: Option<&str>) -> Result<String> {
        if self.rpc_url_template.contains("{api_key}") {
            match api_key {
                Some(key) => Ok(self.rpc_url_template.replace("{api_key}", key)),
                None => Err(AAError::ConfigError(
                    format!("API key required for {} network", self.name)
                )),
            }
        } else {
            Ok(self.rpc_url_template.to_string())
        }
    }

    pub fn get_bundler_url(&self, api_key: Option<&str>) -> Result<String> {
        match &self.bundler_url_template {
            Some(template) => {
                if template.contains("{api_key}") {
                    match api_key {
                        Some(key) => Ok(template.replace("{api_key}", key)),
                        None => Err(AAError::ConfigError(
                            format!("API key required for {} bundler", self.name)
                        )),
                    }
                } else {
                    Ok(template.to_string())
                }
            }
            None => self.get_rpc_url(api_key), // Fallback to RPC URL
        }
    }
}

pub fn get_network_config(chain_id: u64) -> Result<NetworkConfig> {
    match chain_id {
        1 => Ok(NetworkConfig::mainnet()),
        5 => Ok(NetworkConfig::goerli()),
        10 => Ok(NetworkConfig::optimism()),
        137 => Ok(NetworkConfig::polygon()),
        11155111 => Ok(NetworkConfig::sepolia()),
        31337 => Ok(NetworkConfig::anvil()),
        42161 => Ok(NetworkConfig::arbitrum()),
        80001 => Ok(NetworkConfig::polygon_mumbai()),
        _ => Err(AAError::UnsupportedNetwork(chain_id)),
    }
}

pub fn list_supported_networks() -> Vec<NetworkConfig> {
    vec![
        NetworkConfig::mainnet(),
        NetworkConfig::sepolia(),
        NetworkConfig::goerli(),
        NetworkConfig::polygon(),
        NetworkConfig::polygon_mumbai(),
        NetworkConfig::arbitrum(),
        NetworkConfig::optimism(),
        NetworkConfig::anvil(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_configs() {
        let sepolia = NetworkConfig::sepolia();
        assert_eq!(sepolia.chain_id, 11155111);
        assert_eq!(sepolia.name, "Sepolia Testnet");
        
        let anvil = NetworkConfig::anvil();
        assert_eq!(anvil.chain_id, 31337);
        assert_eq!(anvil.name, "Anvil Local");
    }

    #[test]
    fn test_get_network_config() {
        let config = get_network_config(11155111).unwrap();
        assert_eq!(config.chain_id, 11155111);
        
        let result = get_network_config(999999);
        assert!(result.is_err());
    }

    #[test]
    fn test_rpc_url_generation() {
        let sepolia = NetworkConfig::sepolia();
        
        // With API key
        let url = sepolia.get_rpc_url(Some("test_key")).unwrap();
        assert_eq!(url, "https://eth-sepolia.g.alchemy.com/v2/test_key");
        
        // Without API key (should fail)
        let result = sepolia.get_rpc_url(None);
        assert!(result.is_err());
        
        // Local network (no API key needed)
        let anvil = NetworkConfig::anvil();
        let url = anvil.get_rpc_url(None).unwrap();
        assert_eq!(url, "http://localhost:8545");
    }
}
