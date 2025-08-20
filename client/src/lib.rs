pub mod userop;
pub mod bundler;
pub mod wallet;
pub mod error;
pub mod config;
pub mod validation;
pub mod nonce;

// Re-export main types for easier testing
pub use userop::{UserOperationBuilder, UserOperationResponse, GasEstimate};
pub use bundler::BundlerClient;
pub use wallet::{Wallet, WalletFactory};
pub use error::{AAError, Result};
pub use config::{NetworkConfig, get_network_config, list_supported_networks};
pub use validation::{validate_user_operation_basic, validate_gas_fees, validate_address};
pub use nonce::{NonceManager, get_account_nonce, get_account_nonce_with_key};

// Re-export aa-sdk-rs types for convenience
pub use userop::{UserOperationRequest, ExecuteCall, AccountCall};
// Note: UserOperation type is now aa_sdk_rs::types::UserOperation - use directly from aa-sdk-rs
