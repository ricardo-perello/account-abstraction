// TODO: IMPLEMENT MISSING SMART ACCOUNT FACTORY CONTRACT
// Currently this client creates EOA wallets but has no way to deploy AAAccount smart accounts
// You need to create AAAccountFactory.sol and integrate it with this CLI

use clap::{Parser, Subcommand};
use anyhow::Result;
use alloy::primitives::{Address, U256, Bytes};
use std::str::FromStr;

mod userop;
mod bundler;
mod wallet;

use userop::UserOperation;
use bundler::BundlerClient;
use wallet::{Wallet, WalletFactory};

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
        #[arg(short, long)]
        call_data: String,
        
        /// Nonce value
        #[arg(short, long)]
        nonce: u64,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long)]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
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
        #[arg(short, long)]
        call_data: String,
        
        /// Nonce value
        #[arg(short, long)]
        nonce: u64,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long)]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
    },
    
    /// Submit a UserOperation to a bundler
    Submit {
        /// Private key in hex format
        #[arg(short, long)]
        private_key: String,
        
        /// Target contract address
        #[arg(short, long)]
        target: String,
        
        /// Call data (hex string)
        #[arg(short, long)]
        call_data: String,
        
        /// Nonce value
        #[arg(short, long)]
        nonce: u64,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long)]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
    },
    
    /// Deploy a new smart account using the factory
    DeployAccount {
        /// Private key in hex format (for signing deployment transaction)
        #[arg(short, long)]
        private_key: String,
        
        /// Factory contract address
        #[arg(short, long)]
        factory: String,
        
        /// Salt for deterministic deployment (hex string)
        #[arg(short, long)]
        salt: String,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
    },
    
    /// Deploy a new smart account with multiple owners
    DeployMultiOwnerAccount {
        /// Private key in hex format (for signing deployment transaction)
        #[arg(short, long)]
        private_key: String,
        
        /// Factory contract address
        #[arg(short, long)]
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
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
    },
    
    /// Get predicted smart account address before deployment
    PredictAddress {
        /// Factory contract address
        #[arg(short, long)]
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
        #[arg(short, long, default_value = "1")]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create { private_key, target, call_data, nonce, rpc_url, entry_point, chain_id } => {
            create_user_operation(private_key, target, call_data, *nonce, rpc_url, entry_point, *chain_id).await?;
        }
        Commands::Estimate { private_key, target, call_data, nonce, rpc_url, entry_point, chain_id } => {
            estimate_gas(private_key, target, call_data, *nonce, rpc_url, entry_point, *chain_id).await?;
        }
        Commands::Submit { private_key, target, call_data, nonce, rpc_url, entry_point, chain_id } => {
            submit_user_operation(private_key, target, call_data, *nonce, rpc_url, entry_point, *chain_id).await?;
        }
        Commands::DeployAccount { private_key, factory, salt, rpc_url, chain_id } => {
            deploy_smart_account(private_key, factory, salt, rpc_url, *chain_id).await?;
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
    }

    Ok(())
}

async fn create_user_operation(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
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
    
    // Create UserOperation
    let mut user_op = UserOperation::new(wallet.address(), U256::from(nonce));
    user_op = user_op.with_call_data(call_data_bytes);
    
    // Get user operation hash for signing
    let entry_point_addr = Address::from_str(entry_point)?;
    let user_op_hash = user_op.get_user_op_hash(entry_point_addr, U256::from(chain_id))?;
    
    // Sign the user operation
    let signature = wallet.sign_user_operation(user_op_hash)?;
    user_op = user_op.with_signature(signature);
    
    println!("UserOperation created successfully!");
    println!("Sender: {}", user_op.sender);
    println!("Nonce: {}", user_op.nonce);
    println!("Target: {}", target_addr);
    println!("Call Data: 0x{}", hex::encode(&user_op.call_data));
    println!("Signature: 0x{}", hex::encode(&user_op.signature));
    
    Ok(())
}

async fn estimate_gas(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Estimating gas for UserOperation...");
    
    // Create wallet and UserOperation (similar to create function)
    let wallet = Wallet::from_hex(private_key)?;
    let target_addr = Address::from_str(target)?;
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    
    let mut user_op = UserOperation::new(wallet.address(), U256::from(nonce));
    user_op = user_op.with_call_data(call_data_bytes);
    
    // Create bundler client
    let entry_point_addr = Address::from_str(entry_point)?;
    let bundler = BundlerClient::new(
        rpc_url.to_string(),
        entry_point_addr,
        U256::from(chain_id),
    );
    
    // Estimate gas
    let gas_estimate = bundler.estimate_user_operation_gas(&user_op).await?;
    
    println!("Gas estimates:");
    println!("Pre-verification gas: {}", gas_estimate.pre_verification_gas);
    println!("Verification gas limit: {}", gas_estimate.verification_gas_limit);
    println!("Call gas limit: {}", gas_estimate.call_gas_limit);
    
    Ok(())
}

/// Submit a UserOperation to a bundler
/// TODO: UPDATE TO WORK WITH SMART ACCOUNTS
/// Currently this creates UserOperations from EOA wallets, but you need to:
/// 1. Deploy AAAccount smart accounts via factory
/// 2. Use the smart account address as sender
/// 3. Sign with the EOA wallet that owns the smart account
async fn submit_user_operation(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Submitting UserOperation...");
    
    // Create wallet and UserOperation (similar to create function)
    let wallet = Wallet::from_hex(private_key)?;
    let target_addr = Address::from_str(target)?;
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    
    let mut user_op = UserOperation::new(wallet.address(), U256::from(nonce));
    user_op = user_op.with_call_data(call_data_bytes);
    
    // Get user operation hash and sign
    let entry_point_addr = Address::from_str(entry_point)?;
    let user_op_hash = user_op.get_user_op_hash(entry_point_addr, U256::from(chain_id))?;
    let signature = wallet.sign_user_operation(user_op_hash)?;
    user_op = user_op.with_signature(signature);
    
    // Create bundler client and submit
    let bundler = BundlerClient::new(
        rpc_url.to_string(),
        entry_point_addr,
        U256::from(chain_id),
    );
    
    let response = bundler.submit_user_operation(&user_op).await?;
    
    println!("UserOperation submitted successfully!");
    println!("UserOperation hash: {}", response.user_op_hash);
    
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
    
    // Try to get public key
    match wallet.public_key() {
        Ok(pub_key) => println!("Public Key: 0x{}", hex::encode(pub_key)),
        Err(e) => println!("Could not derive public key: {}", e),
    }
    
    Ok(())
}

/// Deploy a new smart account using the factory
async fn deploy_smart_account(
    private_key: &str,
    factory: &str,
    salt: &str,
    rpc_url: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Deploying new smart account...");
    
    // TODO: IMPLEMENT SMART ACCOUNT DEPLOYMENT
    // This function needs to:
    // 1. Create wallet from private key
    // 2. Call factory.createAccount() via RPC
    // 3. Sign the deployment transaction
    // 4. Submit to network
    
    println!("Smart account deployment not yet implemented");
    println!("Factory: {}", factory);
    println!("Salt: {}", salt);
    println!("RPC: {}", rpc_url);
    println!("Chain ID: {}", chain_id);
    
    Ok(())
}

/// Deploy a new smart account with multiple owners
async fn deploy_multi_owner_account(
    private_key: &str,
    factory: &str,
    owners: &str,
    salt: &str,
    rpc_url: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Deploying new multi-owner smart account...");
    
    // TODO: IMPLEMENT MULTI-OWNER SMART ACCOUNT DEPLOYMENT
    // This function needs to:
    // 1. Parse comma-separated owners list
    // 2. Create wallet from private key
    // 3. Call factory.createAccountWithOwners() via RPC
    // 4. Sign the deployment transaction
    // 5. Submit to network
    
    println!("Multi-owner smart account deployment not yet implemented");
    println!("Factory: {}", factory);
    println!("Owners: {}", owners);
    println!("Salt: {}", salt);
    println!("RPC: {}", rpc_url);
    println!("Chain ID: {}", chain_id);
    
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
    
    // TODO: IMPLEMENT ADDRESS PREDICTION
    // This function needs to:
    // 1. Call factory.getAddress() via RPC
    // 2. Display the predicted address
    
    println!("Address prediction not yet implemented");
    println!("Factory: {}", factory);
    println!("Owner: {}", owner);
    println!("Salt: {}", salt);
    println!("RPC: {}", rpc_url);
    println!("Chain ID: {}", chain_id);
    
    Ok(())
}
