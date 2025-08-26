use alloy::primitives::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};
use aa_sdk_rs::types::UserOperationRequest;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PaymasterConfig {
    pub paymaster_address: Address,
    pub signature: [u8; 65],      // ECDSA signature from verifier
    pub valid_until: u64,         // Expiration timestamp
    pub valid_after: u64,         // Start timestamp (usually 0)
}

/// Request format for paymaster-service
#[derive(Debug, Serialize)]
pub struct PaymasterServiceRequest {
    pub api_key: String,
    pub user_operation: PackedUserOperationData,
    pub valid_until: u64,
    pub valid_after: Option<u64>,
}

/// PackedUserOperation format expected by paymaster-service
#[derive(Debug, Serialize)]
pub struct PackedUserOperationData {
    pub sender: String,
    pub nonce: String,
    pub init_code: String,
    pub call_data: String,
    pub account_gas_limits: String,
    pub pre_verification_gas: String,
    pub gas_fees: String,
    pub paymaster_and_data: String,
}

/// Response from paymaster-service
#[derive(Debug, Deserialize)]
pub struct PaymasterServiceResponse {
    pub signature: String,
    pub valid_until: u64,
    pub valid_after: u64,
    pub paymaster_data: String,
}

/// Paymaster service client for ERC-4337 gas sponsorship
pub struct PaymasterService {
    pub service_url: String,
    pub api_key: String,
    pub paymaster_address: Address,
    client: reqwest::Client,
}

impl PaymasterService {
    pub fn new(service_url: String, api_key: String, paymaster_address: Address) -> Self {
        Self {
            service_url,
            api_key,
            paymaster_address,
            client: reqwest::Client::new(),
        }
    }

    /// Request sponsorship for a UserOperation
    pub async fn request_sponsorship(
        &self,
        user_op: &UserOperationRequest,
        valid_until: u64,
        valid_after: Option<u64>,
    ) -> Result<PaymasterConfig> {
        // Convert UserOperationRequest to format expected by paymaster-service
        let packed_user_op = self.convert_user_operation(user_op)?;
        
        let request = PaymasterServiceRequest {
            api_key: self.api_key.clone(),
            user_operation: packed_user_op,
            valid_until,
            valid_after,
        };

        println!("🔧 Requesting paymaster sponsorship...");
        println!("Service URL: {}", self.service_url);
        println!("Valid until: {}", valid_until);
        
        let response = self
            .client
            .post(&format!("{}/sign", self.service_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Paymaster service error: {}", error_text));
        }

        let service_response: PaymasterServiceResponse = response.json().await?;
        
        // Check if this is a SimplePaymaster response (empty signature/data)
        if service_response.signature == "0x" && service_response.paymaster_data == "0x" {
            println!("SimplePaymaster detected - no signature needed");
            
            // For SimplePaymaster, create empty signature and data
            let mut signature = [0u8; 65];
            // Set a dummy signature that won't be used for validation
            signature[0] = 0x00; // r starts with 0
            
            println!("SimplePaymaster sponsorship approved!");
            println!("Signature: 0x{}", service_response.signature);
            println!("Paymaster data: 0x{}", service_response.paymaster_data);

            return Ok(PaymasterConfig {
                paymaster_address: self.paymaster_address,
                signature,
                valid_until: service_response.valid_until,
                valid_after: service_response.valid_after,
            });
        }
        
        // Parse signature from hex for VerifierSignaturePaymaster
        let signature_bytes = hex::decode(&service_response.signature)?;
        if signature_bytes.len() != 65 {
            return Err(anyhow::anyhow!("Invalid signature length: expected 65 bytes, got {}", signature_bytes.len()));
        }
        
        let mut signature = [0u8; 65];
        signature.copy_from_slice(&signature_bytes);

        println!("VerifierSignaturePaymaster sponsorship approved!");
        println!("Signature: 0x{}", service_response.signature);
        println!("Paymaster data: 0x{}", service_response.paymaster_data);

        Ok(PaymasterConfig {
            paymaster_address: self.paymaster_address,
            signature,
            valid_until: service_response.valid_until,
            valid_after: service_response.valid_after,
        })
    }

    /// Convert aa-sdk-rs UserOperationRequest to paymaster-service format
    fn convert_user_operation(&self, user_op: &UserOperationRequest) -> Result<PackedUserOperationData> {
        // Extract values from UserOperationRequest
        let sender = user_op.sender.unwrap_or_default();
        let nonce = user_op.nonce.unwrap_or_default();
        
        // Use factory and factory_data if available, otherwise empty
        let init_code = if let (Some(factory), Some(factory_data)) = (&user_op.factory, &user_op.factory_data) {
            let mut init_code_bytes = Vec::new();
            init_code_bytes.extend_from_slice(factory.as_slice());
            init_code_bytes.extend_from_slice(factory_data);
            Bytes::from(init_code_bytes)
        } else {
            Bytes::default()
        };
        
        // Use the actual call_data from the UserOperationRequest
        let call_data = user_op.call_data.clone().unwrap_or_default();
        
        // Use actual gas values from UserOperationRequest
        let pre_verification_gas = user_op.pre_verification_gas.unwrap_or_default();
        let verification_gas = user_op.verification_gas_limit.unwrap_or_default();
        let call_gas = user_op.call_gas_limit.unwrap_or_default();
        let account_gas_limits = format!("0x{:032x}{:032x}", verification_gas, call_gas);
        
        // Use actual gas fee values from UserOperationRequest  
        let max_priority_fee = user_op.max_priority_fee_per_gas.unwrap_or_default();
        let max_fee = user_op.max_fee_per_gas.unwrap_or_default();
        let gas_fees = format!("0x{:032x}{:032x}", max_priority_fee, max_fee);
        
        // For now, use empty paymaster_and_data since we'll set it later
        let paymaster_and_data = "0x".to_string();

        Ok(PackedUserOperationData {
            sender: format!("0x{:x}", sender),
            nonce: nonce.to_string(),
            init_code: format!("0x{}", hex::encode(&init_code)),
            call_data: format!("0x{}", hex::encode(&call_data)),
            account_gas_limits,
            pre_verification_gas: pre_verification_gas.to_string(),
            gas_fees,
            paymaster_and_data,
        })
    }

    /// Build paymasterAndData EXTRA DATA ONLY (v0.7): signature + validUntil + validAfter
    /// The bundler/EntryPoint prefixes address (20) + verificationGas (16) + postOpGas (16).
    /// Here we must only return the paymaster-specific data: 65 + 8 + 8 = 81 bytes.
    /// For SimplePaymaster, returns empty data since no signature is needed.
    pub fn build_paymaster_and_data(&self, config: &PaymasterConfig) -> Bytes {
        // Check if this is a SimplePaymaster (signature starts with 0x00)
        if config.signature[0] == 0x00 {
            println!("SimplePaymaster detected - returning empty paymaster data");
            return Bytes::new(); // Empty data for SimplePaymaster
        }
        
        // For VerifierSignaturePaymaster, build signature + validUntil + validAfter
        let mut data = Vec::new();
        
        // Signature (65 bytes: r || s || v)
        data.extend_from_slice(&config.signature);
        
        // validUntil (8 bytes, big-endian uint64)
        data.extend_from_slice(&config.valid_until.to_be_bytes());
        
        // validAfter (8 bytes, big-endian uint64)
        data.extend_from_slice(&config.valid_after.to_be_bytes());
        
        // Total: 65 + 8 + 8 = 81 bytes
        Bytes::from(data)
    }
}

#[derive(Debug)]
pub enum PaymasterError {
    InvalidSignature,
    ExpiredSignature,
    InvalidVerifier,
    NetworkError(String),
    ServiceError(String),
}

impl std::fmt::Display for PaymasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymasterError::InvalidSignature => write!(f, "Invalid signature"),
            PaymasterError::ExpiredSignature => write!(f, "Signature expired"),
            PaymasterError::InvalidVerifier => write!(f, "Invalid verifier"),
            PaymasterError::NetworkError(e) => write!(f, "Network error: {}", e),
            PaymasterError::ServiceError(e) => write!(f, "Service error: {}", e),
        }
    }
}

impl std::error::Error for PaymasterError {}
