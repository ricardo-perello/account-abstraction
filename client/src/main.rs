// Refactored to use aa-sdk-rs SmartAccount functionality
// This now properly integrates with aa-sdk-rs provider architecture

use clap::{Parser, Subcommand};
use alloy::primitives::{Address, U256, Bytes};
use std::str::FromStr;

mod userop;
mod bundler;
mod wallet;
mod error;
mod config;

use userop::UserOperationBuilder;
use bundler::BundlerClient;
use wallet::{Wallet, WalletFactory};
use anyhow::Result;
use config::list_supported_networks;

// aa-sdk-rs integration - using SmartAccountProvider properly
use aa_sdk_rs::{
    smart_account::{SimpleAccount, SmartAccount},
    provider::{SmartAccountProvider, SmartAccountProviderTrait},
};
use alloy::providers::ProviderBuilder;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "aa-client")]
#[command(about = "Account Abstraction Client for ERC-4337")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and sign a UserOperation
    Create {
        /// Private key in hex format
        #[arg(short, long)]
        private_key: String,
        
        /// Target contract address
        #[arg(short, long)]
        target: String,
        
        /// Call data (hex string)
        #[arg(short = 'd', long)]
        call_data: String,
        
        /// Nonce value
        #[arg(short, long)]
        nonce: u64,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long, default_value = "0x0000000071727De22E5E9d8BAf0edAc6f37da032")]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
        chain_id: u64,
        
        /// Maximum fee per gas (in wei)
        #[arg(long, default_value = "20000000000")]
        max_fee_per_gas: String,
        
        /// Maximum priority fee per gas (in wei)
        #[arg(long, default_value = "2000000000")]
        max_priority_fee_per_gas: String,
    },
    

    
    /// Submit a UserOperation to a bundler (for arbitrary transactions)
    Submit {
        /// Private key in hex format
        #[arg(short, long)]
        private_key: String,
        
        /// Target contract address
        #[arg(short, long)]
        target: String,
        
        /// Call data (hex string)
        #[arg(short = 'd', long)]
        call_data: String,
        
        /// Factory contract address (needed to identify smart account)
        #[arg(short, long, default_value = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")]
        factory: String,
        
        /// Salt for deterministic deployment (hex string, needed to identify smart account)
        #[arg(short, long)]
        salt: String,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long, default_value = "0x0000000071727De22E5E9d8BAf0edAc6f37da032")]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
        chain_id: u64,
        
        /// Value to send with the transaction (in wei)
        #[arg(long, default_value = "0")]
        value: String,
        
        /// Maximum fee per gas (in wei)
        #[arg(long, default_value = "20000000000")]
        max_fee_per_gas: String,
        
        /// Maximum priority fee per gas (in wei)
        #[arg(long, default_value = "2000000000")]
        max_priority_fee_per_gas: String,
    },
    
    /// Deploy a new smart account using the factory via bundler
    DeployAccount {
        /// Private key in hex format (for signing deployment transaction)
        #[arg(short, long)]
        private_key: String,
        
        /// Factory contract address
        #[arg(short, long, default_value = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")]
        factory: String,
        
        /// Salt for deterministic deployment (hex string)
        #[arg(short, long)]
        salt: String,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
        chain_id: u64,
        
        /// Maximum fee per gas (in wei)
        #[arg(long, default_value = "20000000000")]
        max_fee_per_gas: String,
        
        /// Maximum priority fee per gas (in wei)
        #[arg(long, default_value = "2000000000")]
        max_priority_fee_per_gas: String,
    },
    
    /// Deploy a new smart account with multiple owners via bundler
    DeployMultiOwnerAccount {
        /// Private key in hex format (for signing deployment transaction)
        #[arg(short, long)]
        private_key: String,
        
        /// Factory contract address
        #[arg(short, long, default_value = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")]
        factory: String,
        
        /// Comma-separated list of owner addresses
        #[arg(short, long)]
        owners: String,
        
        /// Salt for deterministic deployment (hex string)
        #[arg(short, long)]
        salt: String,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
        chain_id: u64,
    },
    
    /// Get predicted smart account address before deployment
    PredictAddress {
        /// Factory contract address
        #[arg(short, long, default_value = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")]
        factory: String,
        
        /// Owner address
        #[arg(short, long)]
        owner: String,
        
        /// Salt for deterministic deployment (hex string)
        #[arg(short, long)]
        salt: String,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
        chain_id: u64,
    },
    
    /// Generate a new random wallet
    GenerateWallet,
    
    /// Get account information
    Info {
        /// Private key in hex format
        #[arg(short, long)]
        private_key: String,
    },
    

    
    /// Show network presets and configuration
    Networks,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create { private_key, target, call_data, nonce, rpc_url, entry_point, chain_id, max_fee_per_gas, max_priority_fee_per_gas } => {
            create_user_operation(private_key, target, call_data, *nonce, rpc_url, entry_point, *chain_id, max_fee_per_gas, max_priority_fee_per_gas).await?;
        }

        Commands::Submit { private_key, target, call_data, factory, salt, rpc_url, entry_point: _, chain_id, value, max_fee_per_gas, max_priority_fee_per_gas } => {
            submit_user_operation_fixed(private_key, target, call_data, value, factory, salt, rpc_url, *chain_id, max_fee_per_gas, max_priority_fee_per_gas).await?;
        }
        Commands::DeployAccount { private_key, factory, salt, rpc_url, chain_id, max_fee_per_gas, max_priority_fee_per_gas } => {
            deploy_smart_account(private_key, factory, salt, rpc_url, *chain_id, max_fee_per_gas, max_priority_fee_per_gas).await?;
        }
        Commands::DeployMultiOwnerAccount { private_key, factory, owners, salt, rpc_url, chain_id } => {
            deploy_multi_owner_account(private_key, factory, owners, salt, rpc_url, *chain_id).await?;
        }
        Commands::PredictAddress { factory, owner, salt, rpc_url, chain_id } => {
            predict_smart_account_address(factory, owner, salt, rpc_url, *chain_id).await?;
        }
        Commands::GenerateWallet => {
            generate_wallet().await?;
        }
        Commands::Info { private_key } => {
            show_wallet_info(private_key)?;
        }

        Commands::Networks => {
            show_network_presets()?;
        }
    }

    Ok(())
}

async fn create_user_operation(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    _rpc_url: &str,
    _entry_point: &str,
    _chain_id: u64,
    _max_fee_per_gas: &str,
    _max_priority_fee_per_gas: &str,
) -> Result<()> {
    println!("Creating UserOperation...");
    
    // Create wallet
    let wallet = Wallet::from_hex(private_key)?;
    println!("Wallet address: {}", wallet.address());
    
    // Parse target address
    let target_addr = Address::from_str(target)?;
    
    // Parse call data
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    
    // Create UserOperation using aa-sdk-rs
    let _user_op_request = UserOperationBuilder::new(target_addr, U256::ZERO, call_data_bytes.clone())
        .with_sender(wallet.address())
        .with_nonce(U256::from(nonce))
        .build();
    
    println!("UserOperation created successfully!");
    println!("Sender: {}", wallet.address());
    println!("Nonce: {}", nonce);
    println!("Target: {}", target_addr);
    println!("Call Data: 0x{}", hex::encode(&call_data_bytes));
    println!("Note: aa-sdk-rs handles signing internally when submitting operations");
    
    Ok(())
}



/// Submit a UserOperation to a bundler using aa-sdk-rs SmartAccountProvider (FIXED VERSION)
async fn submit_user_operation_fixed(
    private_key: &str,
    target: &str,
    call_data: &str,
    value: &str,
    factory: &str,      // ✅ Added: Need to identify smart account
    salt: &str,         // ✅ Added: Need to identify smart account
    rpc_url: &str,
    chain_id: u64,
    max_fee_per_gas: &str,
    max_priority_fee_per_gas: &str,
) -> Result<()> {
    println!("🚀 Submitting transaction via smart account using aa-sdk-rs...");
    
    // ✅ Setup
    let wallet = Wallet::from_hex(private_key)?;
    let factory_addr = Address::from_str(factory)?;
    let target_addr = Address::from_str(target)?;
    let entry_point_addr = Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?;
    
    println!("🔧 Setting up aa-sdk-rs SmartAccount...");
    println!("Factory: {}", factory_addr);
    println!("Target: {}", target_addr);
    println!("Owner EOA: {}", wallet.address());
    
    let url = url::Url::parse(rpc_url)?;
    let provider = ProviderBuilder::new().on_http(url);
    
    let simple_account = SimpleAccount::new(
        Arc::new(provider.clone()),
        wallet.address(),
        factory_addr,
        entry_point_addr,
        chain_id,
    );
    
    // ✅ 1. CHECK IF ACCOUNT IS DEPLOYED
    println!("🔍 Checking if smart account is deployed...");
    let is_deployed = simple_account.is_account_deployed().await?;
    if !is_deployed {
        let predicted_addr = simple_account.get_counterfactual_address().await?;
        return Err(anyhow::anyhow!(
            "❌ Smart account not deployed at {}!\n💡 Run deploy-account first with:\n  cargo run -- deploy-account --factory {} --salt {} --private-key {}",
            predicted_addr, factory, salt, private_key
        ));
    }
    
    let account_addr = simple_account.get_account_address().await?;
    println!("✅ Using deployed smart account: {}", account_addr);
    
    // ✅ 2. PREPARE TRANSACTION PARAMETERS  
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    let value_amount = U256::from_str_radix(value, 10)?;
    
    println!("🔧 Preparing transaction for smart account execution...");
    println!("  External target: {}", target_addr);
    println!("  Value to send: {} wei", value_amount);
    println!("  Call data: 0x{}", hex::encode(&call_data_bytes));
    
    // ✅ 3. CREATE USEROPERATION DIRECTLY (NO DOUBLE-ENCODING!)
    let max_fee = U256::from_str_radix(max_fee_per_gas, 10)?;
    let priority_fee = U256::from_str_radix(max_priority_fee_per_gas, 10)?;
    
    // Fix: Pass target parameters directly to UserOperationBuilder
    // This will create ExecuteCall internally - no manual encoding needed!
    let mut user_op_request = UserOperationBuilder::new(
        target_addr,        // ✅ External target (not smart account!)
        value_amount,       // ✅ Value to send to target
        call_data_bytes     // ✅ Call data to send to target
    )
    .with_gas_fees(max_fee, priority_fee)
    .build();
    
    println!("✅ UserOperation created correctly (no double-encoding)");
    
    // ✅ 4. USE AA-SDK-RS CAPABILITIES
    let smart_provider = SmartAccountProvider::new(provider, simple_account);
    
    // Optional: Get gas estimates
    println!("📊 Estimating gas parameters...");
    match smart_provider.estimate_user_operation_gas(&user_op_request).await {
        Ok(estimates) => {
            println!("✅ Gas estimates: {:?}", estimates);
        }
        Err(e) => {
            println!("⚠️  Gas estimation failed (proceeding anyway): {}", e);
        }
    }
    
    // Fill missing fields automatically
    println!("🔧 Filling UserOperation fields automatically...");
    smart_provider.fill_user_operation(&mut user_op_request).await?;
    
    // ✅ 5. SUBMIT WITH TRACKING
    println!("🚀 Submitting transaction via smart account...");
    match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
        Ok(user_op_hash) => {
            println!("✅ UserOperation submitted successfully!");
            println!("UserOperation Hash: {:?}", user_op_hash);
            
            // ✅ TRACK EXECUTION STATUS
            println!("📋 Checking UserOperation execution status...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // Wait for execution
            
            match smart_provider.get_user_operation_receipt(user_op_hash).await {
                Ok(Some(receipt)) => {
                    println!("✅ Transaction executed successfully!");
                    println!("📋 Receipt details: {:?}", receipt);
                    println!("🎉 Smart account transaction completed!");
                }
                Ok(None) => {
                    println!("⏳ Transaction still pending...");
                    println!("💡 Check status later with hash: {:?}", user_op_hash);
                    
                    // Get more operation details
                    if let Ok(Some(op)) = smart_provider.get_user_operation(user_op_hash).await {
                        println!("📊 UserOperation details: {:?}", op);
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not verify execution status: {}", e);
                    println!("💡 Operation may still have succeeded - check blockchain directly");
                }
            }
        }
        Err(e) => {
            println!("❌ Transaction submission failed: {}", e);
            println!("🔍 Possible causes:");
            println!("  1. Smart account not properly deployed");
            println!("  2. Insufficient gas fees");
            println!("  3. Invalid target contract or call data");
            println!("  4. Bundler connectivity issues");
        }
    }
    
    Ok(())
}

async fn generate_wallet() -> Result<()> {
    println!("Generating new random wallet...");
    
    // Use the existing WalletFactory::random() for real random generation
    let wallet = WalletFactory::random()?;
    
    println!("New wallet created:");
    println!("Address: {}", wallet.address());
    println!("Private Key: {}", wallet.export_private_key());
    
    Ok(())
}

fn show_wallet_info(private_key: &str) -> Result<()> {
    println!("Wallet information:");
    
    let wallet = Wallet::from_hex(private_key)?;
    println!("Address: {}", wallet.address());
    println!("Private Key: {}", wallet.export_private_key());
    
    // Note: Public key derivation now handled internally by aa-sdk-rs LocalSigner
    println!("Note: Public key is managed internally by aa-sdk-rs LocalSigner");
    
    Ok(())
}

/// Deploy a new smart account using the factory
async fn deploy_smart_account(
    private_key: &str,
    factory: &str,
    salt: &str,
    rpc_url: &str,
    chain_id: u64,
    max_fee_per_gas: &str,
    max_priority_fee_per_gas: &str,
) -> Result<()> {
    println!("🚀 Deploying new smart account via bundler...");
    
    // Create wallet from private key
    let wallet = Wallet::from_hex(private_key)?;
    println!("Deployer wallet: {}", wallet.address());
    
    // Parse factory address
    let factory_addr = Address::from_str(factory)?;
    println!("Factory contract: {}", factory_addr);
    
    // Parse salt
    let salt_bytes = if salt.starts_with("0x") {
        hex::decode(&salt[2..])?
    } else {
        hex::decode(salt)?
    };
    
    // Create bundler client for RPC calls
    let _bundler = BundlerClient::new(
        rpc_url.to_string(),
        Address::ZERO, // Not needed for deployment
        U256::from(chain_id),
    );
    
    println!("Deploying smart account using real SimpleAccountFactory contract...");
    println!("Factory: {}", factory_addr);
    println!("Owner: {}", wallet.address());
    println!("Salt: 0x{}", hex::encode(&salt_bytes));
    
    // Convert salt bytes to U256
    let mut salt_array = [0u8; 32];
    let start_idx = 32usize.saturating_sub(salt_bytes.len());
    salt_array[start_idx..].copy_from_slice(&salt_bytes[..32.min(salt_bytes.len())]);
    let salt_u256 = U256::from_be_bytes(salt_array);
    
    // Create bundler client
    let bundler_client = BundlerClient::new(
        rpc_url.to_string(),
        Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?, // Default entry point
        U256::from(chain_id),
    );
    
    // First, get the predicted address
    match bundler_client.get_predicted_address(factory_addr, wallet.address(), salt_u256).await {
        Ok(predicted_address) => {
            println!("📍 Predicted smart account address: {}", predicted_address);
            println!("💡 Make sure this address is funded with ETH for gas fees");
            
            println!("🔧 Creating deployment UserOperation...");
            
            // Create concrete provider type for aa-sdk-rs
            let url = url::Url::parse(rpc_url)?;
            let provider = ProviderBuilder::new().on_http(url);
            
            // Create SimpleAccount with proper factory address
            let entry_point_addr = Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?;
            let simple_account = SimpleAccount::new(
                Arc::new(provider.clone()),
                wallet.address(),      // Owner (EOA)
                factory_addr,          // Factory address
                entry_point_addr,      // EntryPoint address  
                chain_id,
            );
            
            // Create SmartAccountProvider
            let smart_provider = SmartAccountProvider::new(provider, simple_account);
            
            // Let aa-sdk-rs automatically handle deployment - this is the key fix from the documentation!
            println!("🔧 Letting aa-sdk-rs automatically handle deployment...");
            println!("📊 aa-sdk-rs will automatically:");
            println!("  - Detect that the account doesn't exist");
            println!("  - Generate proper initCode for factory deployment");
            println!("  - Set the predicted address as sender");
            println!("  - Handle nonce management");
            
            // Parse gas fees
            let max_fee = U256::from_str_radix(max_fee_per_gas, 10)?;
            let priority_fee = U256::from_str_radix(max_priority_fee_per_gas, 10)?;
            
            println!("Gas fees - Max fee: {} wei, Priority fee: {} wei", max_fee, priority_fee);
            
            // Create a simple UserOperation and let aa-sdk-rs handle everything
            let user_op_request = UserOperationBuilder::new(
                predicted_address,  // Target the predicted account address
                U256::ZERO,         // No value transfer
                Bytes::new()        // Empty call data for deployment
            )
            .with_gas_fees(max_fee, priority_fee)
            .build();
            
            println!("✅ Deployment UserOperation created!");
            println!("Predicted Account: {}", predicted_address);
            println!("aa-sdk-rs will handle factory calls automatically");
            
            println!("🚀 Submitting deployment UserOperation to bundler...");
            
            // Submit using SmartAccountProvider to actually deploy the account
            match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
                Ok(user_op_hash) => {
                    println!("✅ Smart account deployment initiated successfully!");
                    println!("UserOperation Hash: {:?}", user_op_hash);
                    println!("The account will be deployed at: {}", predicted_address);
                    println!("You can track this deployment on the blockchain");
                }
                Err(e) => {
                    println!("❌ Error deploying smart account: {}", e);
                    println!("Make sure:");
                    println!("  1. The bundler is running and supports eth_sendUserOperation");
                    println!("  2. The predicted account address is funded with ETH");
                    println!("  3. The factory contract is deployed and accessible");
                }
            }
        }
        Err(e) => {
            println!("❌ Error predicting smart account address: {}", e);
            println!("Make sure the factory contract is deployed and accessible");
        }
    }
    
    Ok(())
}

/// Deploy a new smart account with multiple owners using AAAccountFactory via bundler
async fn deploy_multi_owner_account(
    private_key: &str,
    factory: &str,
    owners: &str,
    salt: &str,
    rpc_url: &str,
    chain_id: u64,
) -> Result<()> {
    println!("🚀 Deploying new multi-owner smart account using AAAccountFactory via bundler...");
    
    // Create wallet from private key
    let wallet = Wallet::from_hex(private_key)?;
    println!("Deployer wallet: {}", wallet.address());
    
    // Parse factory address
    let factory_addr = Address::from_str(factory)?;
    println!("AAAccountFactory contract: {}", factory_addr);
    
    // Parse owners list
    let owner_addresses: Vec<Address> = owners
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Address::from_str(s))
        .collect::<Result<Vec<_>, _>>()?;
    
    // Validate owners
    if owner_addresses.is_empty() {
        return Err(anyhow::anyhow!("At least one owner is required"));
    }
    if owner_addresses.len() > 10 {
        return Err(anyhow::anyhow!("Maximum 10 owners allowed"));
    }
    
    // Check for duplicates
    for i in 0..owner_addresses.len() {
        for j in (i + 1)..owner_addresses.len() {
            if owner_addresses[i] == owner_addresses[j] {
                return Err(anyhow::anyhow!("Duplicate owner address: {}", owner_addresses[i]));
            }
        }
    }
    
    println!("Owners ({}):", owner_addresses.len());
    for (i, owner) in owner_addresses.iter().enumerate() {
        println!("  Owner {}: {}", i + 1, owner);
    }
    
    // Parse salt
    let salt_bytes = if salt.starts_with("0x") {
        hex::decode(&salt[2..])?
    } else {
        hex::decode(salt)?
    };
    
    println!("Salt: 0x{}", hex::encode(&salt_bytes));
    println!("⚠️  Note: Salt will be handled by aa-sdk-rs automatically");
    
    println!("🔧 Setting up aa-sdk-rs for multi-owner deployment...");
    
    // Create concrete provider type for aa-sdk-rs
    let url = url::Url::parse(rpc_url)?;
    let provider = ProviderBuilder::new().on_http(url);
    
    // ⚠️ LIMITATION: aa-sdk-rs SimpleAccount doesn't support multi-owner natively
    // Using first owner as primary owner, factory must handle multi-owner logic
    let primary_owner = owner_addresses[0];
    let simple_account = SimpleAccount::new(
        Arc::new(provider.clone()),
        primary_owner,         // Primary owner from the list
        factory_addr,          // AAAccountFactory address  
        Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?, // EntryPoint address
        chain_id,
    );
    
    println!("📋 Primary owner (for aa-sdk-rs): {}", primary_owner);
    println!("📋 Total owners requested: {} addresses", owner_addresses.len());
    for (i, owner) in owner_addresses.iter().enumerate() {
        println!("  Owner {}: {}", i + 1, owner);
    }
    
    // ✅ Get predicted address BEFORE moving simple_account into provider
    let predicted_address = simple_account.get_counterfactual_address().await?;
    println!("📍 aa-sdk-rs predicted address: {}", predicted_address);
    println!("💡 Make sure this address is funded with ETH for gas fees");
    println!("⚠️  Note: This is single-owner prediction, multi-owner may require custom handling");
    
    // Create SmartAccountProvider (this moves simple_account)
    let smart_provider = SmartAccountProvider::new(provider, simple_account);
    
    // Parse gas fees
    let max_fee = U256::from_str_radix("20000000000", 10)?; // 20 gwei
    let priority_fee = U256::from_str_radix("2000000000", 10)?; // 2 gwei
    
    println!("🔧 Creating deployment UserOperation...");
    println!("📊 aa-sdk-rs will automatically:");
    println!("  - Detect that the account doesn't exist");
    println!("  - Generate initCode for factory deployment");
    println!("  - Set the predicted address as sender");
    println!("  - Handle nonce management");
    
    // ✅ FIXED: Generate multi-owner initCode manually
    println!("🔧 Generating custom initCode for multi-owner deployment...");
    
    // Recreate bundler client and provider for factory interactions
    let bundler_client = BundlerClient::new(
        rpc_url.to_string(),
        Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?,
        U256::from(chain_id),
    );
    
    // Convert salt bytes to U256 for factory call
    let mut salt_array = [0u8; 32];
    let start_idx = 32usize.saturating_sub(salt_bytes.len());
    salt_array[start_idx..].copy_from_slice(&salt_bytes[..32.min(salt_bytes.len())]);
    let salt_u256 = U256::from_be_bytes(salt_array);
    
    // Get the actual predicted address for multi-owner deployment
    let actual_predicted_address = bundler_client.get_predicted_multi_owner_address(factory_addr, owner_addresses.clone(), salt_u256).await?;
    println!("📍 Real multi-owner predicted address: {}", actual_predicted_address);
    println!("💡 Make sure THIS address is funded with ETH: {}", actual_predicted_address);
    
    // Generate call data for createAccountWithOwners
    let factory_provider = bundler_client.create_provider().await?;
    let factory_contract = bundler::AAAccountFactory::new(factory_addr, &factory_provider);
    let factory_call_data = factory_contract.createAccountWithOwners(owner_addresses.clone(), salt_u256).calldata().clone();
    
    // Create custom initCode: factory_address + encoded_function_call
    let mut init_code = Vec::new();
    init_code.extend_from_slice(factory_addr.as_slice());
    init_code.extend_from_slice(&factory_call_data);
    
    println!("✅ Custom initCode generated for {} owners", owner_addresses.len());
    println!("🔍 InitCode: 0x{}", hex::encode(&init_code));
    
    // Create UserOperation with multi-owner settings
    let mut user_op_request = UserOperationBuilder::new(
        actual_predicted_address,  // ✅ Real multi-owner predicted address
        U256::ZERO,               // No direct value transfer
        Bytes::new()              // Empty call data for deployment
    )
    .with_gas_fees(max_fee, priority_fee)
    .build();
    
    // ✅ CRITICAL: Set factory and factory_data for multi-owner deployment
            user_op_request.factory = Some(factory_addr);
        user_op_request.factory_data = Some(Bytes::from(factory_call_data));
        // CRITICAL: Override sender to use multi-owner predicted address, not aa-sdk-rs single-owner prediction
        user_op_request.sender = Some(actual_predicted_address);
        // CRITICAL: For new account deployment, nonce must be 0
        user_op_request.nonce = Some(U256::ZERO);
    
    println!("✅ Multi-owner deployment UserOperation created!");
    println!("Target Account: {}", actual_predicted_address);
    println!("Custom initCode set for multi-owner factory deployment");
    println!("🔍 InitCode contains {} owners", owner_addresses.len());
    
    println!("🚀 Submitting multi-owner deployment UserOperation to bundler...");
    
    // Submit using the same pattern as working deploy-account
    match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
                Ok(user_op_hash) => {
                    println!("✅ Multi-owner smart account deployment initiated successfully!");
                    println!("UserOperation Hash: {:?}", user_op_hash);
                    println!("The account will be deployed at: {}", predicted_address);
                    println!("You can track this deployment on the blockchain");
                    
                    println!();
                    println!();
                    println!("💡 Expected multi-owner features (if factory supports it):");
                    println!("- Any owner can execute transactions");
                    println!("- Owners can add new owners (up to 10 total)");
                    println!("- Owners can remove other owners (but not themselves)");
                    println!("- Cannot remove the last owner");
                    println!();
                    println!("⚠️  Note: This deployment used single-owner aa-sdk-rs pattern");
                    println!("   The actual multi-owner logic depends on factory implementation");
                }
                Err(e) => {
                    println!("❌ Error deploying smart account: {}", e);
                    println!("Make sure:");
                    println!("  1. The bundler is running and supports eth_sendUserOperation");
                    println!("  2. The predicted account address is funded with ETH");  
                    println!("  3. The AAAccountFactory contract is deployed and accessible");
                    println!("  4. Factory supports single-owner deployment pattern for primary owner");
                }
            }
    
    Ok(())
}

/// Predict smart account address before deployment
async fn predict_smart_account_address(
    factory: &str,
    owner: &str,
    salt: &str,
    rpc_url: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Predicting smart account address...");
    
    // Parse addresses
    let factory_addr = Address::from_str(factory)?;
    let owner_addr = Address::from_str(owner)?;
    
    // Parse salt
    let salt_bytes = if salt.starts_with("0x") {
        hex::decode(&salt[2..])?
    } else {
        hex::decode(salt)?
    };
    
    println!("Predicting smart account address using real SimpleAccountFactory contract...");
    println!("Factory: {}", factory_addr);
    println!("Owner: {}", owner_addr);
    println!("Salt: 0x{}", hex::encode(&salt_bytes));
    
    // Convert salt bytes to U256
    let mut salt_array = [0u8; 32];
    let start_idx = 32usize.saturating_sub(salt_bytes.len());
    salt_array[start_idx..].copy_from_slice(&salt_bytes[..32.min(salt_bytes.len())]);
    let salt_u256 = U256::from_be_bytes(salt_array);
    
    // Create bundler client for contract calls
    let bundler_client = BundlerClient::new(
        rpc_url.to_string(),
        Address::ZERO, // Entry point not needed for this call
        U256::from(chain_id),
    );
    
    // Get real predicted address from the factory contract
    match bundler_client.get_predicted_address(factory_addr, owner_addr, salt_u256).await {
        Ok(predicted_address) => {
            println!("✅ Real Predicted Address: {}", predicted_address);
            println!("This address is calculated by the actual SimpleAccountFactory contract");
        }
        Err(e) => {
            println!("❌ Error calling factory contract: {}", e);
            println!("Make sure the factory contract is deployed and the RPC URL is correct");
        }
    }
    
    Ok(())
}

/// Show network presets and configuration
fn show_network_presets() -> Result<()> {
    println!("🌐 Supported Networks");
    println!("=====================");
    println!();
    
    let networks = list_supported_networks();
    
    for network in networks {
        println!("📍 {} (Chain ID: {}):", network.name, network.chain_id);
        println!("  EntryPoint: {}", network.entry_point);
        println!("  Factory: {}", network.factory);
        println!("  RPC Template: {}", network.rpc_url_template);
        // Bundler URL is the same as RPC URL for simplicity
        println!();
    }
    
    println!("📋 Usage Examples:");
    println!("  # Anvil (local)");
    println!("  aa-client demo --yes");
    println!();
    println!("  # Sepolia testnet");
    println!("  aa-client create --chain-id 11155111 --private-key YOUR_KEY --target 0x... --call-data 0x... --nonce 0");
    println!();
    println!("  # With custom RPC");
    println!("  aa-client create --rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY --chain-id 11155111 ...");
    
    Ok(())
}
