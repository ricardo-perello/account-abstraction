# Account Abstraction (ERC-4337) Implementation

A complete implementation of Ethereum Account Abstraction (ERC-4337) featuring a Rust client and Solidity smart contracts.


## 📁 **Repository Structure**

```
account-abstraction/
├── contracts/                 # Foundry workspace with smart contracts
│   ├── src/
│   │   ├── AAAccount.sol     # Smart Account implementation
│   │   ├── AAAccountFactory.sol  # Factory for deploying smart accounts
│   │   └── interfaces/       # ERC-4337 interfaces
│   ├── test/                 # Contract tests
│   └── foundry.toml         # Foundry configuration
├── client/                   # Rust client implementation
│   ├── src/
│   │   ├── main.rs          # CLI interface
│   │   ├── wallet.rs        # Wallet management & signing
│   │   ├── userop.rs        # UserOperation handling
│   │   └── bundler.rs       # Bundler communication
│   └── Cargo.toml           # Rust dependencies
├── docs/                     # Documentation
└── README.md                 # This file
```

## 🔧 **Features**

### **Smart Contracts**
- ✅ **AAAccount.sol** - ERC-4337 compliant smart account
- ✅ **AAAccountFactory.sol** - CREATE2 factory for deterministic deployment
- ✅ **EntryPoint Integration** - Compatible with standard ERC-4337 entry points

### **Rust Client**
- ✅ **Real ECDSA Signing** - secp256k1 implementation using k256 crate
- ✅ **Proper Address Derivation** - secp256k1 + keccak256 hashing
- ✅ **ERC-4337 Compliance** - Exact specification implementation
- ✅ **Smart Account Deployment** - Full deployment flow
- ✅ **Address Prediction** - Predict addresses before deployment
- ✅ **CLI Interface** - Complete command-line tool

## 🚀 **Quick Start**

### **Prerequisites**
- Rust 1.70+ and Cargo
- Foundry (for smart contracts)
- Node.js 18+ (for deployment scripts)

### **Installation**

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd account-abstraction
   ```

2. **Build the Rust client**
   ```bash
   cd client
   cargo build --release
   ```

3. **Build smart contracts**
   ```bash
   cd ../contracts
   forge build
   ```

### **Environment Setup**

Create a `.env` file in the root directory:
```bash
# Alchemy RPC endpoints
ALCHEMY_HTTP=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
ALCHEMY_HTTP_SEPOLIA=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
ALCHEMY_HTTP_GOERLI=https://eth-goerli.g.alchemy.com/v2/YOUR_API_KEY

# Optional: Private key for testing
PRIVATE_KEY=0x...
```

## 📖 **Usage**

### **Generate a New Wallet**
```bash
./client/target/release/aa-client generate-wallet
```

### **Deploy a Smart Account**
```bash
./client/target/release/aa-client deploy-account \
  --private-key 0x... \
  --factory 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 \
  --salt 0x1234567890abcdef \
  --rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY \
  --chain-id 11155111
```

### **Predict Smart Account Address**
```bash
./client/target/release/aa-client predict-address \
  --factory 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 \
  --owner 0x... \
  --salt 0x1234567890abcdef \
  --rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY \
  --chain-id 11155111
```

### **Create and Sign UserOperation**
```bash
./client/target/release/aa-client create \
  --private-key 0x... \
  --target 0x... \
  --call-data 0x... \
  --nonce 0 \
  --entry-point 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 \
  --chain-id 1
```

## 🧪 **Testing**

### **Test Networks**
- **Sepolia** - Recommended for testing (Chain ID: 11155111)
- **Goerli** - Alternative testnet (Chain ID: 5)
- **Mainnet** - Production use (Chain ID: 1)

### **Run Tests**
```bash
# Rust client tests
cd client
cargo test

# Smart contract tests
cd ../contracts
forge test
```

## 🔍 **CLI Commands Reference**

| Command | Description |
|---------|-------------|
| `generate-wallet` | Create a new random wallet |
| `info` | Display wallet information |
| `deploy-account` | Deploy single-owner smart account |
| `deploy-multi-owner-account` | Deploy multi-owner smart account |
| `predict-address` | Predict smart account address |
| `create` | Create and sign UserOperation |
| `estimate` | Estimate gas for UserOperation |
| `submit` | Submit UserOperation to bundler |

## 🏗️ **Architecture**

### **Smart Account Design**
- **BaseAccount** - Abstract base class for all smart accounts
- **AAAccount** - Concrete implementation with ERC-4337 compliance
- **Factory Pattern** - CREATE2 deployment for deterministic addresses
- **Multi-owner Support** - Configurable ownership structure

### **Client Architecture**
- **Wallet Management** - Private key handling and signing
- **UserOperation** - ERC-4337 operation creation and signing
- **Bundler Integration** - Communication with bundlers and RPC providers
- **CLI Interface** - User-friendly command-line tool

## 🔐 **Security Features**

- **Real ECDSA Signing** - No mock signatures
- **Proper Address Derivation** - Cryptographic best practices
- **ERC-4337 Compliance** - Exact specification implementation
- **CREATE2 Deployment** - Deterministic and verifiable addresses

## 📚 **Documentation**

- **`docs/flow-notes.md`** - Detailed implementation notes
- **`client/TODO.md`** - Development progress and status
- **Smart Contract Comments** - Inline documentation

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 **License**

[Add your license information here]

## 🙏 **Acknowledgments**

- **ERC-4337** - Ethereum Account Abstraction standard
- **Foundry** - Smart contract development framework
- **Rust Ecosystem** - Language and tooling
- **Alloy** - Ethereum primitives for Rust

## 📞 **Support**

For questions or issues:
- Open an issue on GitHub
- Check the documentation in `docs/`
- Review the implementation notes

---

**Status**: 🚀 **Production Ready** - All critical components implemented and tested!
