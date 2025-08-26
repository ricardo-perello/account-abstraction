use serde::Deserialize;

pub mod key_manager;
pub mod signature_service;
pub mod api;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub verifier_keys: std::collections::HashMap<String, String>,
    pub api_keys: std::collections::HashMap<String, String>,
    pub server_port: u16,
    pub log_level: String,
    pub chain_id: Option<u64>,
    pub paymaster_address: Option<String>,
    pub is_simple_paymaster: Option<bool>,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        // Allow config file to be specified via environment variable or default
        let config_file = std::env::var("PAYMASTER_CONFIG")
            .unwrap_or_else(|_| "config/default".to_string());
        
        // Remove .toml extension if present for config::File::with_name
        let config_name = config_file.strip_suffix(".toml").unwrap_or(&config_file);
        
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_name))
            .add_source(config::Environment::with_prefix("PAYMASTER"))
            .build()?;
        
        settings.try_deserialize()
    }
    
    pub fn load_from_file(file_path: &str) -> Result<Self, config::ConfigError> {
        let config_name = file_path.strip_suffix(".toml").unwrap_or(file_path);
        
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_name))
            .add_source(config::Environment::with_prefix("PAYMASTER"))
            .build()?;
        
        settings.try_deserialize()
    }
}
