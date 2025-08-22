# 🚀 **ERC-4337 Paymaster - Minimal Implementation Guide**

**Goal**: Enable gasless transactions with a simple paymaster contract  
**Timeline**: 1-2 days  
**Components**: Smart contract + Client integration only

---

## 📊 **Implementation Progress**

| Component | Status | Details | Date |
|-----------|--------|---------|------|
| **Paymaster Contract** | ✅ **COMPLETE** | `VerifierSignaturePaymaster.sol` deployed and tested | 2024-12-19 |
| **Account Abstraction** | ✅ **COMPLETE** | `AAAccount.sol` with proper signature validation | 2024-12-19 |
| **Integration Tests** | ✅ **COMPLETE** | Gasless account creation + transaction execution | 2024-12-19 |
| **Signature Verification** | ✅ **COMPLETE** | Fixed classic ERC-4337 paymaster circular dependency | 2024-12-19 |
| **Client Integration** | 🔄 **IN PROGRESS** | Rust client with paymaster support | - |
| **Admin Service** | ⏳ **PENDING** | Paymaster service for verifier management | - |

### 🎯 **Key Achievements**
- ✅ **Gasless Account Creation**: Account deployed with 163 bytes of code
- ✅ **Gasless Transaction**: 0.1 ETH transferred successfully  
- ✅ **Paymaster Signature Fix**: Implemented canonical ERC-4337 pattern
- ✅ **Account Signature Fix**: Removed double EIP-191 formatting
- ✅ **Integration Test**: Complete end-to-end ERC-4337 flow working
- ✅ **Broadcast Error Fix**: Resolved Foundry script broadcast issues

### 🐛 **Issues Resolved**
1. **Paymaster Circular Dependency**: Fixed by implementing paymaster-specific digest
2. **Account Signature Validation**: Fixed double-hashing of EIP-712 userOpHash
3. **Gas Limits**: Increased from 30k to 100k for transaction execution
4. **Test Flow Order**: Corrected sequence: paymaster data → account signature → submit
5. **Broadcast Errors**: Removed `--broadcast` flag from test script  

---

## 🎯 **What We're Building**

A simple paymaster system that allows users to execute transactions without paying gas:

1. **Paymaster Contract**: Verifies signatures and sponsors gas
2. **Client Integration**: Adds paymaster data to UserOperations
3. **Gasless Transactions**: Users can transact without ETH for gas

---

## 🏗️ **Project Structure**

### **Three Separate Components**

```
account-abstraction/
├── contracts/                    # Smart contracts (shared foundation)
│   ├── src/
│   │   ├── AAAccount.sol                    (existing)
│   │   ├── AAAccountFactory.sol             (existing)
│   │   └── VerifierSignaturePaymaster.sol   (NEW)
│   ├── test/
│   │   ├── AAAccount.t.sol                  (existing)
│   │   └── Paymaster.t.sol                  (NEW)
│   ├── script/
│   │   ├── Deploy.s.sol                     (existing)
│   │   └── DeployPaymaster.s.sol            (NEW)
│   ├── foundry.toml                         (existing)
│   └── remappings.txt                        (existing)
│
├── client/                       # User CLI tool (enhanced)
│   ├── src/
│   │   ├── main.rs                          (enhanced with paymaster)
│   │   ├── userop.rs                        (enhanced with paymaster)
│   │   ├── bundler.rs                        (existing)
│   │   ├── wallet.rs                         (existing)
│   │   ├── error.rs                          (existing)
│   │   ├── config.rs                         (existing)
│   │   └── paymaster.rs                      (NEW)
│   ├── Cargo.toml                            (existing)
│   └── Cargo.lock                            (existing)
│
└── paymaster-service/            # Admin API service (NEW)
    ├── src/
    │   ├── main.rs                           (admin API server)
    │   ├── key_manager.rs                    (verifier key management)
    │   ├── signature_service.rs              (signature generation)
    │   └── api.rs                            (HTTP endpoints)
    ├── Cargo.toml                            (NEW)
    └── config/                               (admin configuration)
```

### **Component Responsibilities**

#### **1. Contracts (contracts/)**
- ✅ **Smart contracts** - AAAccount, AAAccountFactory, VerifierSignaturePaymaster
- ✅ **On-chain logic** - signature verification, gas sponsorship
- ✅ **Shared foundation** - used by both client and service

#### **2. User Client (client/)**
- ✅ **User interface** - CLI commands for gasless transactions
- ✅ **UserOperation creation** - builds operations with paymaster data
- ✅ **No sensitive keys** - just creates and submits operations
- ✅ **Public tool** - can be distributed to users

#### **3. Admin Paymaster Service (paymaster-service/)**
- ✅ **Admin interface** - API for managing verifier keys
- ✅ **Signature generation** - signs gas sponsorship requests
- ✅ **Key management** - secure storage of verifier private keys
- ✅ **Private service** - not distributed to users

---

## 🏗️ **System Overview**

```
User Client → Creates UserOp with paymaster data → Bundler → EntryPoint → Paymaster Contract
                                                                    ↓
                                                              Verifies signature
                                                              Sponsors gas
                                                              Transaction executes
```

---

## 📋 **Implementation Steps**

### **Step 1: Create Paymaster Contract**

#### **1.1 Project Setup**
```bash
# Navigate to your contracts directory
cd contracts

# Create paymaster contract file
touch src/VerifierSignaturePaymaster.sol
```

#### **1.2 Paymaster Contract Implementation**
```solidity
// contracts/src/VerifierSignaturePaymaster.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "../lib/account-abstraction/contracts/core/BasePaymaster.sol";

contract VerifierSignaturePaymaster is BasePaymaster {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;
    
    // Verifier address that authorizes gas sponsorship
    address public immutable verifier;
    
    // Events for monitoring
    event GasSponsored(address indexed user, uint256 gasCost, bytes32 userOpHash);
    
    constructor(IEntryPoint _entryPoint, address _verifier) BasePaymaster(_entryPoint) {
        verifier = _verifier;
    }
    
    function _validatePaymasterUserOp(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 maxCost
    ) internal virtual override returns (bytes memory context, uint256 validationData) {
        
        // Decode paymaster data (signature + expiration)
        PaymasterData memory data = _decodePaymasterData(userOp.paymasterAndData);
        
        // Create message hash for signature verification
        bytes32 messageHash = keccak256(abi.encodePacked(
            userOpHash,           // Bind to specific operation
            data.validUntil,      // Time window
            maxCost               // Gas cost limit
        ));
        
        // Use EIP-191 for signature verification
        bytes32 ethSignedHash = messageHash.toEthSignedMessageHash();
        address recoveredSigner = ethSignedHash.recover(data.signature);
        
        // Verify signature is from authorized verifier
        require(recoveredSigner == verifier, "Invalid verifier signature");
        require(block.timestamp <= data.validUntil, "Signature expired");
        
        // Log gas sponsorship
        emit GasSponsored(userOp.sender, maxCost, userOpHash);
        
        // Return success (empty context, 0 validation data)
        return ("", 0);
    }
    
    // Paymaster data structure
    struct PaymasterData {
        bytes signature;         // ECDSA signature from verifier
        uint64 validUntil;       // Expiration timestamp
    }
    
    // Decode paymaster data from UserOperation
    function _decodePaymasterData(bytes calldata paymasterAndData) 
        internal pure returns (PaymasterData memory data) {
        require(paymasterAndData.length >= 97, "Invalid paymaster data length");
        
        // Extract signature (65 bytes) and validUntil (8 bytes)
        data.signature = paymasterAndData[:65];
        data.validUntil = uint64(bytes8(paymasterAndData[65:73]));
    }
}
```

#### **1.3 Deploy Script**
```solidity
// contracts/script/DeployPaymaster.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/VerifierSignaturePaymaster.sol";

contract DeployPaymasterScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address entryPoint = vm.envAddress("ENTRY_POINT");
        address verifier = vm.envAddress("VERIFIER_ADDRESS");
        
        vm.startBroadcast(deployerPrivateKey);
        
        VerifierSignaturePaymaster paymaster = new VerifierSignaturePaymaster(
            IEntryPoint(entryPoint),
            verifier
        );
        
        vm.stopBroadcast();
        
        console.log("Paymaster deployed at:", address(paymaster));
        console.log("EntryPoint:", entryPoint);
        console.log("Verifier:", verifier);
    }
}
```

#### **1.4 Deploy Contract**
```bash
# Set environment variables
export PRIVATE_KEY="your_deployer_private_key"
export ENTRY_POINT="0x0000000071727De22E5E9d8BAf0edAc6f37da032"
export VERIFIER_ADDRESS="your_verifier_address"

# Deploy to testnet
forge script DeployPaymaster --rpc-url $RPC_URL --broadcast --verify
```

### **Step 2: Update Client for Paymaster Support**

#### **2.1 Add Paymaster Types**
```rust
// client/src/paymaster.rs
use alloy::primitives::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PaymasterConfig {
    pub paymaster_address: Address,
    pub signature: [u8; 65],      // ECDSA signature from verifier
    pub valid_until: u64,         // Expiration timestamp
}

#[derive(Debug, Deserialize)]
pub struct SponsorshipResponse {
    pub signature: String,
    pub valid_until: u64,
    pub paymaster_data: String,
}

#[derive(Debug)]
pub enum PaymasterError {
    InvalidSignature,
    ExpiredSignature,
    InvalidVerifier,
    NetworkError(String),
}

impl std::fmt::Display for PaymasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymasterError::InvalidSignature => write!(f, "Invalid signature"),
            PaymasterError::ExpiredSignature => write!(f, "Signature expired"),
            PaymasterError::InvalidVerifier => write!(f, "Invalid verifier"),
            PaymasterError::NetworkError(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for PaymasterError {}
```

#### **2.2 Enhanced UserOperationBuilder**
```rust
// client/src/userop.rs
use alloy::primitives::{Address, Bytes, U256};
use crate::paymaster::PaymasterConfig;

pub struct UserOperationBuilder {
    request: UserOperationRequest,
    paymaster_config: Option<PaymasterConfig>,
}

impl UserOperationBuilder {
    // ... existing methods ...
    
    /// Set paymaster configuration for gasless transactions
    pub fn with_paymaster(mut self, config: PaymasterConfig) -> Self {
        self.paymaster_config = Some(config);
        self
    }
    
    /// Build the final UserOperationRequest with paymaster data
    pub fn build(self) -> UserOperationRequest {
        let mut request = self.request;
        
        if let Some(paymaster_config) = self.paymaster_config {
            // Set paymaster address
            request = request.paymaster(paymaster_config.paymaster_address);
            
            // Encode paymaster data: signature + validUntil
            let mut paymaster_data = Vec::new();
            paymaster_data.extend_from_slice(&paymaster_config.signature);
            paymaster_data.extend_from_slice(&paymaster_config.valid_until.to_be_bytes());
            
            // Set paymaster data
            request = request.paymaster_data(Bytes::from(paymaster_data));
        }
        
        request
    }
}
```

#### **2.3 Add Gasless Transaction Command**
```rust
// client/src/main.rs
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Submit a gasless transaction using paymaster
    SubmitGasless {
        /// Private key in hex format
        #[arg(short, long)]
        private_key: String,
        
        /// Target contract address
        #[arg(short, long)]
        target: String,
        
        /// Call data (hex string)
        #[arg(short = 'd', long)]
        call_data: String,
        
        /// Value to send with the transaction (in wei)
        #[arg(long, default_value = "0")]
        value: String,
        
        /// Paymaster contract address
        #[arg(short, long)]
        paymaster: String,
        
        /// Verifier private key for signing gas sponsorship
        #[arg(long)]
        verifier_key: String,
        
        /// Factory contract address
        #[arg(short, long)]
        factory: String,
        
        /// Salt for deterministic deployment
        #[arg(short, long)]
        salt: String,
        
        /// RPC URL for the network
        #[arg(short, long)]
        rpc_url: String,
        
        /// Chain ID
        #[arg(short, long)]
        chain_id: u64,
        
        /// Maximum fee per gas (in wei)
        #[arg(long, default_value = "20000000000")]
        max_fee_per_gas: String,
        
        /// Maximum priority fee per gas (in wei)
        #[arg(long, default_value = "2000000000")]
        max_priority_fee_per_gas: String,
    },
}
```

#### **2.4 Implement Gasless Transaction Function**
```rust
// client/src/main.rs
async fn submit_gasless_transaction(
    private_key: &str,
    target: &str,
    call_data: &str,
    value: &str,
    paymaster: &str,
    verifier_key: &str,
    factory: &str,
    salt: &str,
    rpc_url: &str,
    chain_id: u64,
    max_fee_per_gas: &str,
    max_priority_fee_per_gas: &str,
) -> Result<()> {
    println!("🚀 Submitting gasless transaction via paymaster...");
    
    // Setup wallets
    let user_wallet = Wallet::from_hex(private_key)?;
    let verifier_wallet = Wallet::from_hex(verifier_key)?;
    
    // Parse addresses
    let paymaster_addr = Address::from_str(paymaster)?;
    let target_addr = Address::from_str(target)?;
    let factory_addr = Address::from_str(factory)?;
    
    println!("🔧 Setting up gasless transaction...");
    println!("Paymaster: {}", paymaster_addr);
    println!("Target: {}", target_addr);
    println!("User: {}", user_wallet.address());
    println!("Verifier: {}", verifier_wallet.address());
    
    // Create UserOperation without paymaster first
    let call_data_bytes = parse_call_data(call_data)?;
    let value_amount = U256::from_str_radix(value, 10)?;
    
    let mut user_op_request = UserOperationBuilder::new(
        target_addr,
        value_amount,
        call_data_bytes
    )
    .with_gas_fees(
        U256::from_str_radix(max_fee_per_gas, 10)?,
        U256::from_str_radix(max_priority_fee_per_gas, 10)?
    )
    .build();
    
    // Get UserOperation hash for signature
    let user_op_hash = get_user_operation_hash(&user_op_request, chain_id)?;
    
    // Create signature data for verifier
    let signature_data = keccak256(abi_encode!(
        user_op_hash,
        get_expiration_timestamp(),
        estimate_max_gas_cost(&user_op_request)?
    ));
    
    // Sign with verifier key
    let signature = verifier_wallet.sign_message(&signature_data)?;
    
    // Create paymaster configuration
    let paymaster_config = PaymasterConfig {
        paymaster_address: paymaster_addr,
        signature: signature.to_bytes(),
        valid_until: get_expiration_timestamp(),
    };
    
    // Build final UserOperation with paymaster
    let final_user_op = UserOperationBuilder::new(
        target_addr,
        value_amount,
        call_data_bytes
    )
    .with_gas_fees(
        U256::from_str_radix(max_fee_per_gas, 10)?,
        U256::from_str_radix(max_priority_fee_per_gas, 10)?
    )
    .with_paymaster(paymaster_config)
    .build();
    
    // Submit via bundler
    submit_user_operation_with_paymaster(
        final_user_op,
        user_wallet,
        factory_addr,
        salt,
        rpc_url,
        chain_id
    ).await?;
    
    println!("✅ Gasless transaction submitted successfully!");
    Ok(())
}

// Helper functions
fn get_expiration_timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64 + 3600 // 1 hour from now
}

fn keccak256(data: Vec<u8>) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let hash = Keccak256::digest(data);
    hash.into()
}

fn abi_encode!(user_op_hash: String, valid_until: u64, max_gas_cost: U256) -> Vec<u8> {
    // Simple ABI encoding for the message
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&user_op_hash.as_bytes());
    encoded.extend_from_slice(&valid_until.to_be_bytes());
    encoded.extend_from_slice(&max_gas_cost.to_be_bytes());
    encoded
}
```

### **Step 3: Test Gasless Transactions**

#### **3.1 Test Command**
```bash
# Submit gasless transaction
./target/debug/aa-client submit-gasless \
  --private-key $USER_PRIVATE_KEY \
  --target 0xRECIPIENT_ADDRESS \
  --call-data 0x \
  --value 100000000000000 \
  --paymaster 0xPAYMASTER_CONTRACT_ADDRESS \
  --verifier-key $VERIFIER_PRIVATE_KEY \
  --factory 0xFACTORY_ADDRESS \
  --salt 0x00 \
  --rpc-url $RPC_URL \
  --chain-id 11155111
```

#### **3.2 Expected Flow**
1. **User creates UserOperation** (without paymaster)
2. **Verifier signs gas sponsorship** request
3. **Client adds paymaster data** to UserOperation
4. **Bundler submits** to EntryPoint
5. **Paymaster validates signature** and sponsors gas
6. **Transaction executes** without user paying gas

---

## 🎯 **Success Criteria**

✅ **Paymaster contract deployed** and verified  
✅ **Client accepts paymaster parameters**  
✅ **Gasless transactions execute** successfully  
✅ **Signature verification works** correctly  
✅ **Gas sponsorship functions** as expected

---

## 🔧 **Technical Implementation Details**

### **Paymaster Signature Fix (Classic ERC-4337 Trap)**
The paymaster was initially failing with "Invalid verifier signature" due to a circular dependency:
- **Problem**: Paymaster tried to verify signature against `userOpHash` which included `paymasterAndData` (containing the signature)
- **Solution**: Implemented canonical ERC-4337 pattern with paymaster-specific digest

```solidity
// Paymaster computes its own digest (excludes paymasterAndData)
function _pmHash(PackedUserOperation calldata u, uint64 validUntil, uint64 validAfter) 
    internal view returns (bytes32) {
    return keccak256(abi.encode(
        _packForPaymaster(u),        // UserOp fields excluding paymasterAndData
        block.chainid,               // Chain ID
        address(this),               // Paymaster address
        validUntil,                  // Expiration
        validAfter                   // Start time
    ));
}
```

### **Account Signature Validation Fix**
The account was failing with "AA24 signature error" due to double-hashing:
- **Problem**: `_validateSignature` applied EIP-191 formatting to already-formatted EIP-712 hash
- **Solution**: Use `userOpHash` directly for signature recovery

```solidity
function _validateSignature(PackedUserOperation calldata userOp, bytes32 userOpHash) 
    internal override returns (uint256 validationData) {
    // userOpHash is already EIP-712 typed data hash, use directly
    (address signer, ECDSA.RecoverError error,) = ECDSA.tryRecover(userOpHash, userOp.signature);
    // ... validation logic
}
```

### **Test Flow Order Fix**
The integration test required correct sequence to avoid signature mismatches:
1. **Create UserOp** without paymaster data
2. **Add paymaster data** (including verifier signature)
3. **Calculate final UserOp hash** (now includes paymaster data)
4. **Sign with account owner** using final hash
5. **Submit** to EntryPoint

### **Gas Limit Optimization**
Increased gas limits for successful transaction execution:
- **Before**: 30,000 gas (insufficient for ETH transfers)
- **After**: 100,000 gas (adequate for basic operations)  

---

## 🚀 **Next Steps (Optional)**

Once the basic paymaster is working, you can enhance it with:

1. **Multiple verifiers** for different use cases
2. **Gas cost limits** and rate limiting
3. **Backend service** for dynamic verification
4. **Frontend dashboard** for monitoring
5. **Advanced security** features

---

**Focus on getting the basic paymaster working first!** 🎯

The key is to have a working gasless transaction before adding complexity.
