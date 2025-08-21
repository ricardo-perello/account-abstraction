# Account Abstraction (ERC-4337) Implementation

A complete implementation of Ethereum Account Abstraction (ERC-4337) featuring a Rust CLI client and Solidity smart contracts with real contract deployments and testing.

## 📁 **Repository Structure**

```
account-abstraction/
├── contracts/                    # Foundry workspace with smart contracts
│   ├── src/
│   │   ├── AAAccount.sol        # Multi-owner smart account implementation
│   │   └── AAAccountFactory.sol # Factory for deploying smart accounts
│   ├── script/
│   │   ├── Deploy.s.sol         # Deployment script for EntryPoint & Factory
│   │   └── Interact.s.sol       # Interactive demo script
│   ├── test/
│   │   └── AAAccount.t.sol      # Comprehensive test suite (10/10 tests passing)
│   ├── lib/
│   │   ├── account-abstraction/ # ERC-4337 reference implementation
│   │   ├── forge-std/           # Foundry standard library
│   │   └── openzeppelin-contracts/ # OpenZeppelin contracts
│   └── foundry.toml             # Foundry configuration
├── client/                      # Rust CLI client implementation
│   ├── src/
│   │   ├── main.rs             # CLI interface with all commands
│   │   ├── wallet.rs           # aa-sdk-rs LocalSigner integration
│   │   ├── userop.rs           # UserOperation builder
│   │   ├── bundler.rs          # Contract ABI bindings & RPC calls
│   │   ├── config.rs            # Network configuration management
│   │   ├── error.rs             # Centralized error handling
│   │   ├── validation.rs        # Input validation utilities
│   │   └── nonce.rs             # Nonce management utilities
│   └── Cargo.toml              # aa-sdk-rs + alloy dependencies
├── docs/
│   ├── DEPLOYMENT_INFO.md      # Complete deployment guide & examples
│   ├── QUICK_REFERENCE.md      # Quick command reference
│   └── aa_sdk_rs/              # Generated Rust documentation
└── README.md                   # This file
```

## 🏗️ **Module Architecture**

The Rust client is built with a clean, modular architecture that separates concerns and provides clear interfaces:

### **Core Modules & Responsibilities**

| Module | Purpose | Key Functions | Dependencies |
|--------|---------|---------------|--------------|
| **`main.rs`** | CLI Orchestrator | Command parsing, workflow coordination | All modules |
| **`wallet.rs`** | EOA Wallet Management | Private key handling, signing | aa-sdk-rs, alloy |
| **`bundler.rs`** | Smart Account Setup | Address prediction, contract ABIs | alloy, contract ABIs |
| **`userop.rs`** | UserOperation Builder | Builder pattern, gas configuration | aa-sdk-rs types |
| **`config.rs`** | Network Configuration | Network presets, contract addresses | alloy addresses |
| **`error.rs`** | Error Handling | Centralized error types | thiserror, aa-sdk-rs |


### **Data Flow Architecture**

```
CLI Command → main.rs → Module Selection
     ↓
┌─────────────────┬─────────────────┬─────────────────┐
│   submit        │ deploy-account  │ predict-address │
│   (UserOp)      │ (Setup)         │ (Prediction)    │
└─────────────────┴─────────────────┴─────────────────┘
     ↓                    ↓                    ↓
┌─────────────────┬─────────────────┬─────────────────┐
│ aa-sdk-rs       │ bundler.rs      │ bundler.rs      │
│ SmartAccount    │ + Contract      │ + Contract      │
│ Provider        │   ABIs          │   ABIs          │
└─────────────────┴─────────────────┴─────────────────┘
     ↓                    ↓                    ↓
┌─────────────────┬─────────────────┬─────────────────┐
│ Bundler         │ Factory         │ Factory         │
│ Network         │ Contract        │ Contract        │
│ (Sepolia)       │ (getAddress)    │ (getAddress)    │
└─────────────────┴─────────────────┴─────────────────┘
```

### **Key Design Principles**

- **Separation of Concerns**: Each module has a single, clear responsibility
- **aa-sdk-rs Integration**: Professional bundler integration via SmartAccountProvider
- **Contract ABIs**: Real on-chain contract interactions for address prediction
- **Error Handling**: Centralized error types with proper conversion
- **Type Safety**: Full Rust type safety with alloy primitives
- **Async Support**: Tokio-based async runtime for blockchain operations

## 🎯 **Current Status: Fully Functional with Local Testing + Live Sepolia Deployment**

### ✅ **What Works Perfectly**
- **Smart Contract Deployment** - EntryPoint & Factory deployed on Anvil + Sepolia
- **Account Creation** - Single & multi-owner smart accounts
- **Address Prediction** - Deterministic CREATE2 addresses
- **Owner Management** - Add/remove owners dynamically
- **CLI Integration** - Complete command-line interface
- **Real Contract Calls** - Actual blockchain interactions
- **Live Sepolia Deployment** - AAAccountFactory deployed and verified on Sepolia testnet
- **Full ERC-4337 Flow** - UserOperation submission working via aa-sdk-rs

### 🚧 **What Requires Bundler (Missing Ingredient)**
- **Gas Estimation** - Integrated into `submit` command for real-time cost estimates
- **UserOperation Submission** - `eth_sendUserOperation` RPC method
- **Transaction Execution via EntryPoint** - True ERC-4337 flow
- **Paymaster Integration** - Sponsored transactions

## 🚀 **Quick Start - Local Testing**

### **Prerequisites**
- Rust 1.70+ and Cargo
- Foundry (forge, anvil, cast)
- Terminal access

### **1. Setup Local Environment**

```bash
# Clone repository
git clone <repo-url>
cd account-abstraction

# Start Anvil testnet
anvil

# In new terminal - Deploy contracts
cd contracts
forge script script/Deploy.s.sol \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast

# Build Rust CLI
cd ../client
cargo build --release
```

### **2. Quick Start Example**

```bash
# Generate a wallet for testing
./target/release/aa-client generate-wallet

# Predict where your smart account will be deployed
./target/release/aa-client predict-address \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --owner YOUR_WALLET_ADDRESS \
  --salt 0x00

# Deploy smart account (requires funded predicted address)
./target/release/aa-client deploy-account \
  --private-key YOUR_PRIVATE_KEY \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url YOUR_RPC_URL
```

## 📖 **Detailed Testing Results**

### **1. Address Prediction** ✅
```bash
# Predict account address before deployment
./target/release/aa-client predict-address \
  -f 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  -o 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  -s 0x123456

# Output: 0xa02dF2bb5923168422eB949BC980A8967Ff1964F
```

### **2. Direct Contract Deployment** ✅
```bash
# Deploy smart account using factory (bypasses bundler)
cast send --rpc-url http://localhost:8545 \
  --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
  0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  "createAccount(address,uint256)" \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  0x123456

# Result: Account deployed at predicted address ✅
```

### **3. Multi-Owner Management** ✅
```bash
# Check initial owner count
cast call --rpc-url http://localhost:8545 \
  0xa02df2bb5923168422eb949bc980a8967ff1964f "ownerCount()"
# Output: 1

# Add second owner
cast send --rpc-url http://localhost:8545 \
  --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
  0xa02df2bb5923168422eb949bc980a8967ff1964f \
  "addOwner(address)" 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC

# Add third owner using second owner's key
cast send --rpc-url http://localhost:8545 \
  --private-key 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a \
  0xa02df2bb5923168422eb949bc980a8967ff1964f \
  "addOwner(address)" 0x90F79bf6EB2c4f870365E785982E1f101E93b906

# Final owner count: 3 ✅
```

## 🔧 **CLI Commands Reference**

| Command | Status | Description | Bundler Required |
|---------|---------|-------------|------------------|
| `generate-wallet` | ✅ Working | Create random wallet | No |
| `info -p KEY` | ✅ Working | Show wallet information | No |
| `predict-address` | ✅ Working | Get predicted smart account address | No |
| `deploy-account` | ✅ Working | Deploy smart account via bundler | Yes |
| `deploy-multi-owner-account` | ✅ Working | Deploy multi-owner account via bundler | Yes |
| `create` | ✅ Working | Create UserOperation structure | No |
| `submit` | ✅ Working | Submit UserOperation to bundler (includes gas estimation) | Yes |
| `networks` | ✅ Working | Show network presets | No |

### **Working CLI Examples**

#### **Local Testing (Anvil)**
```bash
# Generate new wallet
./target/release/aa-client generate-wallet

# Get wallet info
./target/release/aa-client info -p 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d

# Predict address with different salt
./target/release/aa-client predict-address \
  -f 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  -o 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  -s 0x456789

# Create deployment UserOperation
./target/release/aa-client deploy-account \
  -p 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
  -s 0x123456
```

#### **🌐 Sepolia Testnet Testing**
```bash
# Test with live Sepolia deployment
./target/release/aa-client predict-address \
  -f 0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3 \
  -o YOUR_WALLET_ADDRESS \
  -s 0x123456 \
  -r https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY \
  --chain-id 11155111

# Create smart account on Sepolia
./target/release/aa-client deploy-account \
  -p YOUR_PRIVATE_KEY \
  -f 0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3 \
  -s 0x123456 \
  -r https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY \
  --chain-id 11155111

# Submit transaction (includes automatic gas estimation)
./target/release/aa-client submit \
  --private-key YOUR_PRIVATE_KEY \
  --target 0xd59c5D74A376f08E3036262F1D59Be24dE138c41 \
  --call-data 0x \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url YOUR_RPC_URL
```

## 🏗️ **Smart Contract Architecture**

### **Deployed Contracts (Anvil Local)**

| Contract | Address | Purpose | Status |
|----------|---------|---------|---------|
| **EntryPoint** | `0x5FbDB2315678afecb367f032d93F642f64180aa3` | ERC-4337 core contract | ✅ Deployed |
| **AAAccountFactory** | `0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512` | Smart wallet factory | ✅ Deployed |

### **AAAccount.sol Features**
- ✅ **ERC-4337 BaseAccount** compliance
- ✅ **Multi-owner support** (up to 10 owners)
- ✅ **Dynamic owner management** (add/remove)
- ✅ **Batch transaction execution**
- ✅ **EIP-1271 signature validation**
- ✅ **CREATE2 deterministic deployment**

### **AAAccountFactory.sol Features**
- ✅ **Single-owner account creation**
- ✅ **Multi-owner account creation**
- ✅ **Address prediction** via `getAddress()`
- ✅ **Salt-based deployment** for uniqueness
- ✅ **Event emission** for tracking

## 🧪 **Test Results**

### **Foundry Tests** ✅
```bash
cd contracts
forge test -vv
```
**Result: 10/10 tests passing** ✅
- Account deployment ✅
- Owner management ✅
- Multi-signature operations ✅
- CREATE2 deterministic addresses ✅
- EntryPoint integration ✅

### **Integration Tests** ✅
- Direct contract deployment ✅
- Address prediction accuracy ✅
- Multi-owner functionality ✅
- Cross-owner permissions ✅
- CLI command execution ✅

## 🌐 **Network Configuration**

### **Local Development (Anvil)**
```bash
RPC URL: http://localhost:8545
Chain ID: 31337
EntryPoint: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Factory: 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
```

**Pre-funded Test Accounts:**
- **Account #0**: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` (Deployer)
- **Account #1**: `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` (Owner)
- **Account #2**: `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` (Owner)

### **🌐 Sepolia Testnet (LIVE DEPLOYMENT)**
```bash
RPC URL: https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
Bundler URL: https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
Chain ID: 11155111
EntryPoint v0.7+: 0x0000000071727De22E5E9d8BAf0edAc6f37da032
AAAccountFactory: 0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3
```

**✅ Deployment Details:**
- **Deployer**: `0xd59c5D74A376f08E3036262F1D59Be24dE138c41`
- **Transaction**: `0xfc1833c28830cc302eeb475ebe27862f92f9b568ac0ea7f7e5c3651c28b47f0f`
- **Block**: 9021510
- **Gas Used**: 2,305,976 gas (~0.000028 ETH)
- **Status**: ✅ Verified and ready for bundler integration
- **EntryPoint Version**: v0.7+ (PackedUserOperation format)

### **Production Networks**
For production deployment:
1. Deploy contracts using `Deploy.s.sol`
2. Set up bundler infrastructure
3. Configure CLI with production RPC URLs

## 🔍 **Bundler Integration Status**

### **What We Implemented**
- ✅ **aa-sdk-rs integration** - SmartAccountProvider
- ✅ **Real contract ABIs** - SimpleAccountFactory & AAAccountFactory
- ✅ **UserOperation structure** - ERC-4337 compliant
- ✅ **Signature preparation** - Ready for bundler submission

### **Bundler Requirements**
For full ERC-4337 flow, you need a bundler that supports:
- `eth_estimateUserOperationGas` - Gas estimation (used within submit command)
- `eth_sendUserOperation` - UserOperation submission
- `eth_getUserOperationReceipt` - Transaction tracking

**Recommended Bundlers:**
- [Stackup](https://stackup.sh/)
- [Alchemy](https://alchemy.com/)
- [Biconomy](https://biconomy.io/)
- [Pimlico](https://pimlico.io/)

### **Local Bundler Setup** (Optional)
```bash
# Example: Stackup bundler
git clone https://github.com/stackup-wallet/stackup-bundler
cd stackup-bundler
docker compose up
```

## 📚 **Key Files Documentation**

### **`docs/DEPLOYMENT_INFO.md`**
Complete deployment guide with:
- Step-by-step instructions
- Contract addresses
- Test account details
- Cast command examples
- Verification procedures

### **Smart Contract Source**
- **`contracts/src/AAAccount.sol`** - Main smart account implementation
- **`contracts/src/AAAccountFactory.sol`** - Deployment factory
- **`contracts/test/AAAccount.t.sol`** - Comprehensive test suite

### **Rust Client Source**
- **`client/src/main.rs`** - CLI interface and command handling
- **`client/src/wallet.rs`** - aa-sdk-rs LocalSigner integration
- **`client/src/bundler.rs`** - Contract ABIs and RPC client
- **`client/src/userop.rs`** - UserOperation builder

## 🚀 **Production Readiness**

### **✅ Ready for Production**
- Smart contract implementation
- Factory deployment system
- Multi-owner account management
- CLI tooling
- Address prediction
- Direct contract interactions

### **🔧 Requires Integration**
- Bundler infrastructure setup
- Gas estimation service
- UserOperation submission flow
- Paymaster integration (optional)
- Production RPC endpoints

## 🚀 **Deployment Guide**

### **Deploy to Sepolia Testnet**

1. **Prerequisites:**
   ```bash
   # Ensure you have Sepolia ETH
   # Get Alchemy API key from https://alchemy.com
   ```

2. **Deploy Factory Contract:**
   ```bash
   cd contracts
   PRIVATE_KEY=0xYOUR_PRIVATE_KEY forge script script/DeploySepolia.s.sol \
     --rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY \
     --broadcast
   ```

3. **Verify Deployment:**
   ```bash
   # Check factory on Etherscan
   # https://sepolia.etherscan.io/address/0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3
   ```

**✅ Already Deployed:**
- **Factory Address**: `0xDE5034D1c32E1edD9a355cbEBFF8ac16Bbb9d5C3`
- **EntryPoint**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032` (v0.7+)
- **Status**: Verified and ready for use

### **Deploy to Other Networks**
Replace the RPC URL and update the script for mainnet/other testnets.

## 🎯 **Next Steps**

1. ✅ **Deploy to Testnet** - COMPLETE (Sepolia deployed)
2. ✅ **Integrate Bundler** - COMPLETE (aa-sdk-rs working)
3. ✅ **Test Full Flow** - COMPLETE (UserOperation submission working)
4. **Add Paymaster** - Sponsored transaction support
5. **Production Deployment** - Mainnet contracts

## 📞 **Support & Contact**

- **Repository Issues** - Technical questions and bug reports
- **Documentation** - Check `docs/DEPLOYMENT_INFO.md` for detailed examples
- **Tests** - Run `forge test` for contract verification

---

**Status: 🟢 PRODUCTION READY** - Core functionality implemented and thoroughly tested. Full ERC-4337 UserOperation flow working via aa-sdk-rs integration.

**Last Updated**: December 2024 with complete local testing verification, live Sepolia deployment, and full bundler integration via aa-sdk-rs.