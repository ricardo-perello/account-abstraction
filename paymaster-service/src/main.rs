use axum::{
    routing::{post, get},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod key_manager;
mod signature_service;
mod api;

use signature_service::SignatureService;
use key_manager::KeyManager;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::load().expect("Failed to load config");
    
    // Initialize services
    let key_manager = Arc::new(KeyManager::new(&config));
    let signature_service = Arc::new(SignatureService::new(key_manager));
    
    // Build application
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/sign", post(api::sign_sponsorship))
        .route("/metrics", get(api::get_metrics))
        .with_state(signature_service);
    
    // Start server
    let addr = format!("[::]:{}", config.server_port).parse().unwrap();
    tracing::info!("Starting paymaster service on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub verifier_keys: std::collections::HashMap<String, String>,
    pub server_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::with_prefix("PAYMASTER"))
            .build()?;
        
        settings.try_deserialize()
    }
}
