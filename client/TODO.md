# TODO: Implementation Tasks for Account Abstraction Client

## 🚨 CRITICAL - Must Implement for Production

### 1. **ECDSA Signing Implementation** (`src/wallet.rs`)
- **Status**: ❌ Mock implementation only
- **Location**: `create_mock_signature()` function
- **What to do**: Replace with real secp256k1 signing using `k256` crate
- **Why critical**: Mock signatures won't work on real networks

### 2. **Proper Address Derivation** (`src/wallet.rs`)
- **Status**: ❌ Simplified XOR-based implementation
- **Location**: `private_key_to_address()` function
- **What to do**: Implement proper secp256k1 public key derivation + keccak256 hashing
- **Why critical**: Wrong addresses will cause transaction failures

### 3. **ERC-4337 Hashing Algorithm** (`src/userop.rs`)
- **Status**: ❌ Using simple keccak256 instead of exact ERC-4337 spec
- **Location**: `get_user_op_hash()` function
- **What to do**: Implement exact ERC-4337 UserOperation hashing
- **Why critical**: Bundlers will reject incorrectly hashed operations

### 4. **Smart Account Factory Contract** (`../contracts/src/AAAccountFactory.sol`)
- **Status**: ❌ Missing entirely
- **What to do**: Create factory contract to deploy AAAccount instances
- **Why critical**: No way to create smart accounts from CLI

## ⚠️ IMPORTANT - Should Implement Soon

### 5. **Proper ABI Encoding** (`src/userop.rs`)
- **Status**: ❌ Manual byte manipulation
- **Location**: `encode_for_signing()` function
- **What to do**: Use proper ABI encoding library (alloy-abi)
- **Why important**: Ensures correct transaction format

### 6. **Real Random Wallet Generation** (`src/main.rs`)
- **Status**: ❌ Hardcoded wallet (always same address)
- **Location**: `generate_wallet()` function
- **What to do**: Use existing `WalletFactory::random()` instead
- **Why important**: Users need unique wallets

### 7. **BIP39 Mnemonic Support** (`src/wallet.rs`)
- **Status**: ❌ Simplified hash-based derivation
- **Location**: `WalletFactory::from_mnemonic()`
- **What to do**: Implement proper BIP39 + PBKDF2 + BIP32/44
- **Why important**: Industry standard for wallet recovery

## 🔧 INTEGRATION TASKS

### 8. **Connect EOA Wallets to Smart Accounts**
- **Status**: ❌ CLI creates EOAs but can't deploy smart accounts
- **What to do**: 
  1. Implement AAAccountFactory
  2. Add CLI commands to deploy smart accounts
  3. Update UserOperation creation to use smart account addresses

### 9. **Add Smart Account Deployment Commands**
- **Status**: ❌ Missing CLI commands
- **What to do**: Add commands like:
  - `deploy-account` - Deploy new smart account
  - `list-accounts` - Show deployed smart accounts
  - `fund-account` - Send ETH to smart account

## 📋 IMPLEMENTATION ORDER

1. **Start with #1 (ECDSA signing)** - Critical for any real usage
2. **Then #2 (Address derivation)** - Required for correct wallet addresses
3. **Then #4 (Factory contract)** - Enables smart account creation
4. **Then #8 (Integration)** - Connects the pieces together
5. **Then #3 (ERC-4337 hashing)** - Ensures bundler compatibility
6. **Then the rest** - Polish and additional features

## 🧪 TESTING

- **Current**: Basic unit tests exist
- **Need**: Integration tests with real networks
- **Need**: End-to-end tests from CLI → Bundler → Smart Account

## 📚 RESOURCES

- **ECDSA**: Use `k256` crate for secp256k1 operations
- **ABI**: Use `alloy-abi` for proper encoding
- **BIP39**: Use `bip39` crate for mnemonic handling
- **ERC-4337**: Reference the official specification for exact hashing
