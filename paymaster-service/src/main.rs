use axum::{
    routing::{post, get},
    Router,
};
use std::sync::Arc;

mod key_manager;
mod signature_service;
mod api;

use signature_service::SignatureService;
use key_manager::KeyManager;
use paymaster_service::Config;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::load().expect("Failed to load config");
    
    // Initialize services
    let key_manager = Arc::new(KeyManager::new(&config));
    
    // Parse chain ID and paymaster address from config
    let chain_id = config.chain_id.unwrap_or(1); // Default to Ethereum mainnet
    let paymaster_address = if let Some(addr_str) = &config.paymaster_address {
        let addr_clean = addr_str.strip_prefix("0x").unwrap_or(addr_str);
        hex::decode(addr_clean).unwrap_or(vec![0u8; 20])
    } else {
        vec![0u8; 20] // Default to zero address
    };
    
    let is_simple_paymaster = config.is_simple_paymaster.unwrap_or(false);
    
    let signature_service = Arc::new(SignatureService::new(
        key_manager, 
        config.api_keys, 
        chain_id, 
        paymaster_address,
        is_simple_paymaster
    ));
    
    // Build application
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/sign", post(api::sign_sponsorship))
        .route("/metrics", get(api::get_metrics))
        .with_state(signature_service);
    
    // Start server
    let addr = format!("[::]:{}", config.server_port);
    tracing::info!("Starting paymaster service on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


