// Network configuration for different chains
use alloy::primitives::Address;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: &'static str,
    pub chain_id: u64,
    pub entry_point: Address,
    pub factory: Address,
    pub rpc_url_template: &'static str,
}

impl NetworkConfig {
    pub fn sepolia() -> Self {
        Self {
            name: "Sepolia Testnet",
            chain_id: 11155111,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::from_str("0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3").unwrap(),
            rpc_url_template: "https://eth-sepolia.g.alchemy.com/v2/{api_key}",
        }
    }

    pub fn anvil() -> Self {
        Self {
            name: "Anvil Local",
            chain_id: 31337,
            entry_point: Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032").unwrap(),
            factory: Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap(),
            rpc_url_template: "http://localhost:8545",
        }
    }
}

pub fn list_supported_networks() -> Vec<NetworkConfig> {
    vec![
        NetworkConfig::sepolia(),
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
    fn test_list_supported_networks() {
        let networks = list_supported_networks();
        assert_eq!(networks.len(), 2);
        assert_eq!(networks[0].chain_id, 11155111); // Sepolia
        assert_eq!(networks[1].chain_id, 31337);    // Anvil
    }
}
