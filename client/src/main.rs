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
    smart_account::SimpleAccount,
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
    
    /// Estimate gas for a UserOperation
    Estimate {
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
    
    /// Run guided demo with Anvil deployed contracts
    Demo {
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
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
        Commands::Estimate { private_key, target, call_data, nonce, rpc_url, entry_point, chain_id, max_fee_per_gas, max_priority_fee_per_gas } => {
            estimate_gas(private_key, target, call_data, *nonce, rpc_url, entry_point, *chain_id, max_fee_per_gas, max_priority_fee_per_gas).await?;
        }
        Commands::Submit { private_key, target, call_data, nonce, rpc_url, entry_point, chain_id, value, max_fee_per_gas, max_priority_fee_per_gas } => {
            submit_user_operation(private_key, target, call_data, *nonce, rpc_url, entry_point, *chain_id, value, max_fee_per_gas, max_priority_fee_per_gas).await?;
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
        Commands::Demo { yes } => {
            run_guided_demo(*yes).await?;
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

async fn estimate_gas(
    private_key: &str,
    target: &str,
    _call_data: &str,
    _nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
    _max_fee_per_gas: &str,
    _max_priority_fee_per_gas: &str,
) -> Result<()> {
    println!("Estimating gas using aa-sdk-rs SmartAccountProvider...");
    
    // Create wallet and parse parameters
    let wallet = Wallet::from_hex(private_key)?;
    let target_addr = Address::from_str(target)?;
    let entry_point_addr = Address::from_str(entry_point)?;
    
    println!("Creating SmartAccountProvider and SimpleAccount...");
    println!("Target: {}", target_addr);
    println!("Entry Point: {}", entry_point_addr);
    
    // Create concrete provider type
    let url = url::Url::parse(rpc_url)?;
    let provider = ProviderBuilder::new().on_http(url);
    
    // Use the deployed factory address for Sepolia or local
    let factory_addr = if chain_id == 11155111 {
        Address::from_str("0x59bcaa1BB72972Df0446FCe98798076e718E3b61")? // Your deployed AAAccountFactory on Sepolia
    } else {
        Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")? // Anvil factory
    };
    let _simple_account = SimpleAccount::new(
        Arc::new(provider.clone()),
        wallet.address(),      // Owner (EOA)
        factory_addr,          // Factory address
        entry_point_addr,      // EntryPoint address  
        chain_id,
    );
    
        // Gas estimation via bundler has compatibility issues with aa-sdk-rs
    // Provide estimated values based on typical ERC-4337 operations
    println!("⚠️  Gas estimation via bundler has compatibility issues");
    println!("📊 Using estimated gas values based on typical ERC-4337 operations:");
    println!("  Pre-verification gas: ~21000 wei (typical)");
    println!("  Verification gas limit: ~100000 wei (account validation)");
    println!("  Call gas limit: ~21000 wei (basic transfer)");
    println!("  Total estimated gas: ~142000 wei");
    println!();
    println!("💡 For accurate gas estimation, use the submit command which works correctly");
    println!("   The submit command successfully interacts with the bundler");
    
    Ok(())
}

/// Submit a UserOperation to a bundler using aa-sdk-rs SmartAccountProvider
async fn submit_user_operation(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
    value: &str,
    max_fee_per_gas: &str,
    max_priority_fee_per_gas: &str,
) -> Result<()> {
    println!("Submitting UserOperation using aa-sdk-rs SmartAccountProvider...");
    
    // Create wallet and parse parameters
    let wallet = Wallet::from_hex(private_key)?;
    let target_addr = Address::from_str(target)?;
    let entry_point_addr = Address::from_str(entry_point)?;
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    
    println!("Creating SmartAccountProvider and SimpleAccount...");
    println!("Target: {}", target_addr);
    println!("Entry Point: {}", entry_point_addr);
    
    // Create concrete provider type  
    let url = url::Url::parse(rpc_url)?;
    let provider = ProviderBuilder::new().on_http(url);
    
    // Use the deployed factory address for Sepolia or local
    let factory_addr = if chain_id == 11155111 {
        Address::from_str("0x59bcaa1BB72972Df0446FCe98798076e718E3b61")? // Your deployed AAAccountFactory on Sepolia
    } else {
        Address::from_str("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")? // Anvil factory
    };
    
    // Create SimpleAccount with proper factory address
    // Note: Using SimpleAccount even with AAAccountFactory since the core interface is compatible
    let simple_account = SimpleAccount::new(
        Arc::new(provider.clone()),
        wallet.address(),      // Owner (EOA)
        factory_addr,          // Factory address
        entry_point_addr,      // EntryPoint address  
        chain_id,
    );
    
    // Create SmartAccountProvider
    let smart_provider = SmartAccountProvider::new(provider, simple_account);
    
    // Parse value and gas fees
    let value_amount = U256::from_str_radix(value, 10)?;
    let max_fee = U256::from_str_radix(max_fee_per_gas, 10)?;
    let priority_fee = U256::from_str_radix(max_priority_fee_per_gas, 10)?;
    
    println!("Transaction value: {} wei", value_amount);
    println!("Gas fees - Max fee: {} wei, Priority fee: {} wei", max_fee, priority_fee);
    
    // Create a UserOperation using our builder
    // Let aa-sdk-rs automatically determine sender and initCode for deployment
    let user_op_request = UserOperationBuilder::new(
        target_addr,
        value_amount,  // Use the parsed value instead of U256::ZERO
        call_data_bytes.clone()
    )
    // Don't set sender manually - let SmartAccountProvider handle deployment
    .with_nonce(U256::from(nonce))
    .with_gas_fees(max_fee, priority_fee)
    .build();
    
    println!("Submitting to bundler via SmartAccountProvider...");
    
    // Submit using SmartAccountProvider
    match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
            Ok(user_op_hash) => {
                println!("✅ UserOperation submitted successfully!");
                println!("UserOperation Hash: {:?}", user_op_hash);
                println!("You can track this transaction on the blockchain");
            }
            Err(e) => {
                println!("❌ Error submitting UserOperation: {}", e);
                println!("Make sure the bundler is running and supports eth_sendUserOperation");
                println!("Also ensure the UserOperation is valid and properly signed");
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
    
    // Convert salt bytes to U256
    let mut salt_array = [0u8; 32];
    let start_idx = 32usize.saturating_sub(salt_bytes.len());
    salt_array[start_idx..].copy_from_slice(&salt_bytes[..32.min(salt_bytes.len())]);
    let salt_u256 = U256::from_be_bytes(salt_array);
    
    println!("Salt: 0x{}", hex::encode(&salt_bytes));
    
    // Create bundler client for contract interactions
    let bundler_client = BundlerClient::new(
        rpc_url.to_string(),
        Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?, // Default entry point
        U256::from(chain_id),
    );
    
    // First, get the predicted address for multi-owner account
    match bundler_client.get_predicted_multi_owner_address(factory_addr, owner_addresses.clone(), salt_u256).await {
        Ok(predicted_address) => {
            println!("📍 Predicted multi-owner account address: {}", predicted_address);
            println!("💡 Make sure this address is funded with ETH for gas fees");
            
            // Generate call data for createAccountWithOwners
            let provider = bundler_client.create_provider().await?;
            let factory_contract = bundler::AAAccountFactory::new(factory_addr, &provider);
            
            // Get the call data for createAccountWithOwners
            let call_data = factory_contract.createAccountWithOwners(owner_addresses.clone(), salt_u256).calldata().clone();
            
            println!("🔧 Creating multi-owner deployment UserOperation...");
            
            // Create concrete provider type for aa-sdk-rs
            let url = url::Url::parse(rpc_url)?;
            let provider = ProviderBuilder::new().on_http(url);
            
            // Create SimpleAccount with proper factory address
            let simple_account = SimpleAccount::new(
                Arc::new(provider.clone()),
                wallet.address(),      // Owner (EOA)
                factory_addr,          // Factory address
                Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?, // EntryPoint address  
                chain_id,
            );
            
            // Create SmartAccountProvider
            let smart_provider = SmartAccountProvider::new(provider, simple_account);
            
            // Create a UserOperation for multi-owner deployment
            let user_op_request = UserOperationBuilder::new(
                factory_addr,
                U256::ZERO,
                call_data.clone()
            )
            .with_nonce(U256::ZERO)
            .build();
            
            println!("✅ Multi-owner deployment UserOperation created!");
            println!("Target (AAAccountFactory): {}", factory_addr);
            println!("Predicted Account: {}", predicted_address);
            println!("Call Data: 0x{}", hex::encode(&call_data));
            println!("Owners: {} addresses", owner_addresses.len());
            
            println!("🚀 Submitting multi-owner deployment UserOperation to bundler...");
            
            // Submit using SmartAccountProvider to actually deploy the account
            match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
                Ok(user_op_hash) => {
                    println!("✅ Multi-owner smart account deployment initiated successfully!");
                    println!("UserOperation Hash: {:?}", user_op_hash);
                    println!("The account will be deployed at: {}", predicted_address);
                    println!("You can track this deployment on the blockchain");
                    
                    println!();
                    println!("💡 Multi-owner features:");
                    println!("- Any owner can execute transactions");
                    println!("- Owners can add new owners (up to 10 total)");
                    println!("- Owners can remove other owners (but not themselves)");
                    println!("- Cannot remove the last owner");
                }
                Err(e) => {
                    println!("❌ Error deploying multi-owner smart account: {}", e);
                    println!("Make sure:");
                    println!("  1. The bundler is running and supports eth_sendUserOperation");
                    println!("  2. The predicted account address is funded with ETH");
                    println!("  3. The AAAccountFactory contract is deployed and accessible");
                }
            }
        }
        Err(e) => {
            println!("❌ Error predicting multi-owner account address: {}", e);
            println!("Make sure the AAAccountFactory contract is deployed and accessible");
            println!("The factory should support createAccountWithOwners function");
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

/// Run a guided demo with the deployed Anvil contracts
async fn run_guided_demo(skip_prompts: bool) -> Result<()> {
    use std::io::BufRead;
    
    println!("🚀 AA Client Demo with Anvil Deployed Contracts");
    println!("================================================");
    println!();
    
    // Anvil constants - clean deployment with deterministic addresses
    let anvil_rpc = "http://localhost:8545";
    let anvil_chain_id = 31337u64;
    let entry_point = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";
    let factory = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";
    
    // Test account from Anvil (Owner1)
    let test_private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    let test_address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
    
    println!("📊 Network Information:");
    println!("  RPC URL: {}", anvil_rpc);
    println!("  Chain ID: {}", anvil_chain_id);
    println!("  EntryPoint: {}", entry_point);
    println!("  Factory: {}", factory);
    println!();
    
    println!("🔑 Test Account:");
    println!("  Address: {}", test_address);
    println!("  Private Key: {}", test_private_key);
    println!();
    
    if !skip_prompts {
        println!("Press Enter to continue with the demo...");
        let stdin = std::io::stdin();
        let _ = stdin.lock().read_line(&mut String::new())?;
    }
    
    // Step 1: Show wallet info
    println!("📋 Step 1: Wallet Information");
    println!("==============================");
    show_wallet_info(test_private_key)?;
    println!();
    
    if !skip_prompts {
        println!("Press Enter to continue...");
        let stdin = std::io::stdin();
        let _ = stdin.lock().read_line(&mut String::new())?;
    }
    
    // Step 2: Predict smart account address
    println!("🔮 Step 2: Predict Smart Account Address");
    println!("=========================================");
    let salt = "0x123456";
    predict_smart_account_address(factory, test_address, salt, anvil_rpc, anvil_chain_id).await?;
    println!();
    
    if !skip_prompts {
        println!("Press Enter to continue...");
        let stdin = std::io::stdin();
        let _ = stdin.lock().read_line(&mut String::new())?;
    }
    
    // Step 3: Deploy single-owner smart account
    println!("🏗️  Step 3: Deploy Single-Owner Smart Account");
    println!("==============================================");
    deploy_smart_account(test_private_key, factory, salt, anvil_rpc, anvil_chain_id, "20000000000", "2000000000").await?;
    println!();
    
    if !skip_prompts {
        println!("Press Enter to continue...");
        let stdin = std::io::stdin();
        let _ = stdin.lock().read_line(&mut String::new())?;
    }
    
    // Step 4: Deploy multi-owner smart account
    println!("👥 Step 4: Deploy Multi-Owner Smart Account (AAAccount)");
    println!("========================================================");
    println!("Note: This requires AAAccountFactory (not SimpleAccountFactory)");
    let owners = format!("{},0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", test_address);
    let multi_salt = "0x654321";
    // For demo purposes, we'll show what would happen with the proper factory
    println!("Demo with AAAccountFactory deployment:");
    deploy_multi_owner_account(test_private_key, factory, &owners, multi_salt, anvil_rpc, anvil_chain_id).await?;
    println!();
    
    if !skip_prompts {
        println!("Press Enter to continue...");
        let stdin = std::io::stdin();
        let _ = stdin.lock().read_line(&mut String::new())?;
    }
    
    // Step 5: Create UserOperation
    println!("⚡ Step 5: Create UserOperation");
    println!("===============================");
    let target = "0x0000000000000000000000000000000000000000"; // null address for demo
    let call_data = "0x";
    let nonce = 0u64;
    create_user_operation(test_private_key, target, call_data, nonce, anvil_rpc, entry_point, anvil_chain_id, "20000000000", "2000000000").await?;
    println!();
    
    println!("✅ Demo Complete!");
    println!("================");
    println!();
    println!("🔧 To interact with the deployed contracts manually:");
    println!("  1. Use the deployed addresses shown above");
    println!("  2. Use test accounts from Anvil for transactions");
    println!("  3. Check DEPLOYMENT_INFO.md for more examples");
    println!();
    println!("📚 Example commands:");
    println!("  aa-client info -p {}", test_private_key);
    println!("  aa-client predict-address -f {} -o {} -s {}", factory, test_address, salt);
    println!("  aa-client deploy-account -p {} -s {}", test_private_key, salt);
    println!();
    
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
