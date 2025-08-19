// Refactored to use aa-sdk-rs SmartAccount functionality
// This now properly integrates with aa-sdk-rs provider architecture

use clap::{Parser, Subcommand};
use anyhow::Result;
use alloy::primitives::{Address, U256, Bytes};
use std::str::FromStr;

mod userop;
mod bundler;
mod wallet;

use userop::UserOperationBuilder;
use bundler::BundlerClient;
use wallet::{Wallet, WalletFactory};

// aa-sdk-rs integration - types available for proper implementation
// use aa_sdk_rs::{
//     smart_account::SimpleAccount,
//     provider::{SmartAccountProvider, SmartAccountProviderTrait},
// };

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
        #[arg(short, long, default_value = "0x5FbDB2315678afecb367f032d93F642f64180aa3")]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
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
        #[arg(short = 'd', long)]
        call_data: String,
        
        /// Nonce value
        #[arg(short, long)]
        nonce: u64,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long, default_value = "0x5FbDB2315678afecb367f032d93F642f64180aa3")]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
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
        #[arg(short = 'd', long)]
        call_data: String,
        
        /// Nonce value
        #[arg(short, long)]
        nonce: u64,
        
        /// RPC URL for the network
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc_url: String,
        
        /// Entry point contract address
        #[arg(short, long, default_value = "0x5FbDB2315678afecb367f032d93F642f64180aa3")]
        entry_point: String,
        
        /// Chain ID
        #[arg(short, long, default_value = "31337")]
        chain_id: u64,
    },
    
    /// Deploy a new smart account using the factory
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
    },
    
    /// Deploy a new smart account with multiple owners
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
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Estimating gas using aa-sdk-rs SmartAccountProvider...");
    
    // Create wallet and parse parameters
    let wallet = Wallet::from_hex(private_key)?;
    let target_addr = Address::from_str(target)?;
    let entry_point_addr = Address::from_str(entry_point)?;
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    
    println!("Estimating gas for UserOperation using real bundler...");
    println!("Target: {}", target_addr);
    println!("Sender: {}", wallet.address());
    println!("Call Data: 0x{}", hex::encode(&call_data_bytes));
    println!("Entry Point: {}", entry_point_addr);
    
    // Create a UserOperation for gas estimation
    let user_op_request = UserOperationBuilder::new(
        target_addr,
        U256::ZERO,
        call_data_bytes.clone()
    )
    .with_sender(wallet.address())
    .with_nonce(U256::from(nonce))
    .build();
    
    // Create bundler client for real gas estimation
    let bundler_client = BundlerClient::new(
        rpc_url.to_string(),
        entry_point_addr,
        U256::from(chain_id),
    );
    
    // Get real gas estimation from bundler
    match bundler_client.estimate_user_operation_gas(&user_op_request).await {
        Ok(gas_estimate) => {
            println!("✅ Real gas estimation from bundler:");
            println!("  Pre-verification gas: {}", gas_estimate.pre_verification_gas);
            println!("  Verification gas limit: {}", gas_estimate.verification_gas_limit);
            println!("  Call gas limit: {}", gas_estimate.call_gas_limit);
        }
        Err(e) => {
            println!("❌ Error getting gas estimation: {}", e);
            println!("Make sure the bundler is running and supports eth_estimateUserOperationGas");
        }
    }
    
    Ok(())
}

/// Submit a UserOperation to a bundler
/// Updated to use aa-sdk-rs architecture
async fn submit_user_operation(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
) -> Result<()> {
    println!("Submitting UserOperation using aa-sdk-rs...");
    
    // Create wallet and parse parameters
    let wallet = Wallet::from_hex(private_key)?;
    let target_addr = Address::from_str(target)?;
    let entry_point_addr = Address::from_str(entry_point)?;
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    
    println!("Submitting UserOperation to real bundler...");
    println!("Target: {}", target_addr);
    println!("Sender: {}", wallet.address());
    println!("Call Data: 0x{}", hex::encode(&call_data_bytes));
    println!("Entry Point: {}", entry_point_addr);
    
    // Create a real UserOperation using our builder
    let user_op_request = UserOperationBuilder::new(
        target_addr,
        U256::ZERO,
        call_data_bytes.clone()
    )
    .with_sender(wallet.address())
    .with_nonce(U256::from(nonce))
    .build();
    
    // Create bundler client for real submission
    let bundler_client = BundlerClient::new(
        rpc_url.to_string(),
        entry_point_addr,
        U256::from(chain_id),
    );
    
    println!("Submitting to bundler: {}", bundler_client.rpc_url());
    
    // Submit to real bundler
    match bundler_client.submit_user_operation(&user_op_request).await {
        Ok(user_op_hash) => {
            println!("✅ UserOperation submitted successfully!");
            println!("UserOperation Hash: 0x{}", hex::encode(user_op_hash));
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
) -> Result<()> {
    println!("Deploying new smart account...");
    
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
        Address::from_str("0x5FbDB2315678afecb367f032d93F642f64180aa3")?, // Default entry point
        U256::from(chain_id),
    );
    
    // First, get the predicted address
    match bundler_client.get_predicted_address(factory_addr, wallet.address(), salt_u256).await {
        Ok(predicted_address) => {
            println!("📍 Predicted smart account address: {}", predicted_address);
            
            // Generate real call data using the factory contract ABI
            let provider = bundler_client.create_provider().await?;
            let factory_contract = bundler::SimpleAccountFactory::new(factory_addr, &provider);
            
            // Get the call data for createAccount
            let call_data = factory_contract.createAccount(wallet.address(), salt_u256).calldata().clone();
            
            // Create a UserOperation for deployment
            let _user_op_request = UserOperationBuilder::new(
                factory_addr,
                U256::ZERO,
                call_data.clone()
            )
            .with_sender(wallet.address())
            .with_nonce(U256::ZERO)
            .build();
            
            println!("✅ Smart account deployment UserOperation created with real ABI!");
            println!("Target (Factory): {}", factory_addr);
            println!("Predicted Account: {}", predicted_address);
            println!("Call Data: 0x{}", hex::encode(&call_data));
            
            // Optionally submit to bundler
            println!("To deploy, submit this UserOperation to a bundler");
        }
        Err(e) => {
            println!("❌ Error predicting smart account address: {}", e);
            println!("Make sure the factory contract is deployed and accessible");
        }
    }
    
    Ok(())
}

/// Deploy a new smart account with multiple owners
async fn deploy_multi_owner_account(
    private_key: &str,
    factory: &str,
    owners: &str,
    salt: &str,
    _rpc_url: &str,
    _chain_id: u64,
) -> Result<()> {
    println!("Deploying new multi-owner smart account...");
    
    // Create wallet from private key
    let wallet = Wallet::from_hex(private_key)?;
    println!("Deployer wallet: {}", wallet.address());
    
    // Parse factory address
    let factory_addr = Address::from_str(factory)?;
    println!("Factory contract: {}", factory_addr);
    
    // Parse owners list
    let owner_addresses: Vec<Address> = owners
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Address::from_str(s))
        .collect::<Result<Vec<_>, _>>()?;
    
    println!("Owners: {:?}", owner_addresses);
    
    // Parse salt
    let salt_bytes = if salt.starts_with("0x") {
        hex::decode(&salt[2..])?
    } else {
        hex::decode(salt)?
    };
    
    println!("Creating multi-owner smart account deployment...");
    println!("Factory: {}", factory_addr);
    println!("Owners: {:?}", owner_addresses);
    println!("Salt: 0x{}", hex::encode(&salt_bytes));
    
    // Create call data for a hypothetical createMultiOwnerAccount function
    // This is simplified - real multi-sig accounts would need more complex logic
    println!("Note: Multi-owner accounts require custom smart contract implementation");
    println!("This would typically use a different factory contract than SimpleAccount");
    
    // For demonstration, show how to encode multiple owners
    println!("Multi-owner account parameters:");
    for (i, owner) in owner_addresses.iter().enumerate() {
        println!("  Owner {}: {}", i + 1, owner);
    }
    
    println!("⚠️  Multi-signature accounts are not yet implemented in this client");
    println!("Use SimpleAccount deployment for single-owner accounts instead");
    
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
    let entry_point = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
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
    deploy_smart_account(test_private_key, factory, salt, anvil_rpc, anvil_chain_id).await?;
    println!();
    
    if !skip_prompts {
        println!("Press Enter to continue...");
        let stdin = std::io::stdin();
        let _ = stdin.lock().read_line(&mut String::new())?;
    }
    
    // Step 4: Deploy multi-owner smart account
    println!("👥 Step 4: Deploy Multi-Owner Smart Account");
    println!("============================================");
    let owners = format!("{},0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", test_address);
    let multi_salt = "0x654321";
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
    create_user_operation(test_private_key, target, call_data, nonce, anvil_rpc, entry_point, anvil_chain_id).await?;
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
    println!("🌐 Network Presets");
    println!("==================");
    println!();
    
    println!("📍 Anvil (Local):");
    println!("  RPC URL: http://localhost:8545");
    println!("  Chain ID: 31337");
    println!("  EntryPoint: 0x5FbDB2315678afecb367f032d93F642f64180aa3");
    println!("  Factory: 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
    println!();
    
    println!("🌍 Sepolia (Testnet):");
    println!("  RPC URL: https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY");
    println!("  Chain ID: 11155111");
    println!("  EntryPoint: [Deploy with forge script]");
    println!("  Factory: [Deploy with forge script]");
    println!();
    
    println!("📋 Usage Examples:");
    println!("  # Anvil (default)");
    println!("  aa-client demo --yes");
    println!();
    println!("  # Sepolia");
    println!("  aa-client create -r https://eth-sepolia.g.alchemy.com/v2/KEY --chain-id 11155111 -p KEY -t TARGET -c DATA -n NONCE");
    
    Ok(())
}
