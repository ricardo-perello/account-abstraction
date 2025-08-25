use axum::http::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use tower::ServiceExt;
use paymaster_service::{Config};

mod test_app {
    use super::*;
    use axum::{
        routing::{post, get},
        Router,
    };
    use std::sync::Arc;

    pub fn create_test_config() -> Config {
        let mut verifier_keys = HashMap::new();
        verifier_keys.insert("default".to_string(), "0000000000000000000000000000000000000000000000000000000000000001".to_string());
        
        let mut api_keys = HashMap::new();
        api_keys.insert("test_key_123".to_string(), "Test Client".to_string());
        
        Config {
            verifier_keys,
            api_keys,
            server_port: 3000,
            log_level: "info".to_string(),
            chain_id: Some(1),
            paymaster_address: Some("0x0000000000000000000000000000000000000000".to_string()),
        }
    }

    pub async fn create_test_app() -> Router {
        use paymaster_service::signature_service::SignatureService;
        use paymaster_service::key_manager::KeyManager;
        use paymaster_service::api;
        
        let config = create_test_config();
        let key_manager = Arc::new(KeyManager::new(&config));
        let signature_service = Arc::new(SignatureService::new(
            key_manager, 
            config.api_keys,
            1, // chain_id
            vec![0u8; 20] // paymaster_address
        ));
        
        Router::new()
            .route("/health", get(api::health_check))
            .route("/sign", post(api::sign_sponsorship))
            .route("/metrics", get(api::get_metrics))
            .with_state(signature_service)
    }
}

#[tokio::test]
async fn test_health_check() {
    let app = test_app::create_test_app().await;
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_valid_sign_request() {
    let app = test_app::create_test_app().await;
    
    let request_body = json!({
        "api_key": "test_key_123",
        "user_operation": {
            "sender": "0x1234567890123456789012345678901234567890",
            "nonce": "1",
            "init_code": "0x",
            "call_data": "0x1234",
            "account_gas_limits": "0x00000000000f424000000000000f4240",
            "pre_verification_gas": "21000",
            "gas_fees": "0x000000000077359400000000003b9aca00",
            "paymaster_and_data": "0x"
        },
        "valid_until": (chrono::Utc::now().timestamp() + 3600) as u64,
        "valid_after": 0
    });
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/sign")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_api_key() {
    let app = test_app::create_test_app().await;
    
    let request_body = json!({
        "api_key": "invalid_key",
        "user_operation": {
            "sender": "0x1234567890123456789012345678901234567890",
            "nonce": "1",
            "init_code": "0x",
            "call_data": "0x1234",
            "account_gas_limits": "0x00000000000f424000000000000f4240",
            "pre_verification_gas": "21000",
            "gas_fees": "0x000000000077359400000000003b9aca00",
            "paymaster_and_data": "0x"
        },
        "valid_until": (chrono::Utc::now().timestamp() + 3600) as u64,
        "valid_after": 0
    });
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/sign")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_metrics() {
    let app = test_app::create_test_app().await;
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
