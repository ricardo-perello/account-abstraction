# 🚀 ERC-4337 CLI - Quick Reference Card

**Status**: ✅ **PRODUCTION READY** - Full bundler integration achieved & tested on Sepolia!

## 🔑 **Key Architecture (IMPORTANT)**

**Two Types of Accounts:**
- **EOA (Externally Owned Account)**: Traditional wallet with private key - this is what `--private-key` refers to
- **Smart Account**: Contract-based account deployed by factory - controlled by the EOA

**How It Works:**
1. Your `--private-key` is the **EOA private key** (traditional wallet)  
2. This EOA **owns and controls** the smart account
3. Smart account executes transactions **on behalf of** the EOA
4. All CLI commands use the **EOA private key** to authorize smart account operations

## 🎯 **Essential Commands (TESTED ✅)**

### **Deploy Smart Account via Bundler**
```bash
source ../.env && ./target/debug/aa-client deploy-account \
  --private-key YOUR_EOA_PRIVATE_KEY \  # Private key of EOA that will own the smart account
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61
```

### **Execute Transaction via Smart Account**
```bash
source ../.env && ./target/debug/aa-client submit \
  --private-key YOUR_EOA_PRIVATE_KEY \  # Private key of EOA that owns the smart account
  --target RECIPIENT_ADDRESS \
  --call-data 0x \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --value 100000000000000
```

### **Predict Smart Account Address**
```bash
source ../.env && ./target/debug/aa-client predict-address \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --owner YOUR_EOA_ADDRESS \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA
```

### **Generate Test Wallet**
```bash
./target/debug/aa-client generate-wallet
```

## 🌐 **Network Configs (TESTED)**

### **Sepolia Testnet (✅ WORKING)**
```bash
--chain-id 11155111
--rpc-url $ALCHEMY_HTTP_SEPOLIA  # From .env file
--factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61
--entry-point 0x0000000071727De22E5E9d8BAf0edAc6f37da032
```

### **Local Anvil (✅ WORKING)**
```bash
--chain-id 31337
--rpc-url http://localhost:8545
--factory 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
--entry-point 0x0000000071727De22E5E9d8BAf0edAc6f37da032
```

## ⚙️ **Gas Configuration**

### **Recommended Gas Fees (Sepolia)**
```bash
--max-fee-per-gas 20000000000        # 20 gwei (tested)
--max-priority-fee-per-gas 2000000000  # 2 gwei (tested)
```

### **High Priority (if needed)**
```bash
--max-fee-per-gas 30000000000        # 30 gwei
--max-priority-fee-per-gas 5000000000  # 5 gwei
```

## 🔄 **Complete Workflow (PROVEN)**

### **Step 1: Setup Environment**
```bash
cd client
source ../.env  # Contains ALCHEMY_HTTP_SEPOLIA and PRIVATE_KEY
cargo build    # Build if not already built
```

### **Step 2: Generate or Use Wallet**
```bash
# Option A: Generate new test wallet
./target/debug/aa-client generate-wallet

# Option B: Use existing wallet info
./target/debug/aa-client info --private-key $PRIVATE_KEY
```

### **Step 3: Deploy Smart Account**
```bash
# Deploy via bundler (funds account automatically)
./target/debug/aa-client deploy-account \
  --private-key YOUR_WALLET_PRIVATE_KEY \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61
```

### **Step 4: Execute Transactions**
```bash
# Send ETH through smart account
./target/debug/aa-client submit \
  --private-key YOUR_WALLET_PRIVATE_KEY \
  --target 0xRECIPIENT_ADDRESS \
  --call-data 0x \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --value 100000000000000  # 0.0001 ETH in wei
```

## 🧪 **Testing Commands**

### **Utility Commands**
```bash
# Show all network configurations
./target/debug/aa-client networks

# Run guided demo (requires local setup)
./target/debug/aa-client demo --yes

# Show help for any command
./target/debug/aa-client submit --help
```

### **Verification Commands**
```bash
# Check smart account balance
cast balance SMART_ACCOUNT_ADDRESS --rpc-url $ALCHEMY_HTTP_SEPOLIA --ether

# Verify smart account has code (is deployed)
cast code SMART_ACCOUNT_ADDRESS --rpc-url $ALCHEMY_HTTP_SEPOLIA
```

## 🚨 **Troubleshooting Quick Fixes**

### **Common Issues**

| Error | Quick Fix |
|-------|-----------|
| "Odd number of digits" | Use `--salt 0x00` (not `0x0`) |
| "sender balance is 0" | Fund the **predicted** smart account address |
| "Gas fees too low" | Add `--max-priority-fee-per-gas 2000000000` |
| "Account not deployed" | Run deploy-account first, or let submit auto-deploy |

### **Pre-Flight Checklist**
- ✅ `.env` file exists with `ALCHEMY_HTTP_SEPOLIA`
- ✅ Wallet has ETH for gas fees
- ✅ Using salt `0x00` (aa-sdk-rs default)
- ✅ Correct factory address for network
- ✅ Smart account address is funded (if pre-deployed)

## 🎯 **Working Examples (Live Tested)**

### **Real Transaction on Sepolia**
```bash
# This exact command worked:
source ../.env && ./target/debug/aa-client submit \
  --private-key 0x9ec161507ad1cfd507ae6e6bf012a66d609276782ae64f70ca41174d402d10ae \  # EOA private key
  --target 0xd59c5D74A376f08E3036262F1D59Be24dE138c41 \  # Transaction recipient
  --call-data 0x \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x00 \
  --chain-id 11155111 \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --value 100000000000000  # 0.0001 ETH in wei

# EOA Address: 0x21D541ef2237b2a63076666651238AC8A7cde752 (derived from private key above)
# Smart Account: 0xd710e28ecfb47f55f234513ce3be18a31974590c (controlled by the EOA)
# Result: UserOperation Hash: 0x9decccb00e204f5273a42282e141a035fd1a35e8bebad033b32276e3c0f09eaf
# Status: ✅ SUCCESSFUL - 0.0001 ETH transferred FROM smart account TO recipient
```

## 📋 **Value Reference**

### **ETH Amounts in Wei**
| ETH | Wei | Use Case |
|-----|-----|----------|
| 0.0001 | 100000000000000 | Micro transfers |
| 0.001 | 1000000000000000 | Small transfers |
| 0.01 | 10000000000000000 | Gas funding |
| 0.1 | 100000000000000000 | Larger transfers |

### **Salt Values**
| Salt | Usage | Notes |
|------|-------|-------|
| `0x00` | Default | ✅ Recommended - matches aa-sdk-rs |
| `0x123456` | Custom | ⚠️ CLI only - aa-sdk-rs ignores |

## 🏆 **Success Indicators**

### **Deployment Success**
```
✅ Smart account deployment initiated successfully!
UserOperation Hash: 0x...
The account will be deployed at: 0x...
```

### **Transaction Success**
```
✅ UserOperation submitted successfully!
UserOperation Hash: 0x...
📋 Checking UserOperation execution status...
```

### **Address Prediction Success**
```
✅ Real Predicted Address: 0x...
This address is calculated by the actual SimpleAccountFactory contract
```

---

## 📚 **References**

- **Full Guide**: `USER_GUIDE.md` - Complete documentation
- **Deployment Info**: `DEPLOYMENT_INFO.md` - Network details and contracts
- **Build Instructions**: `README.md` - Setup and installation

---

**🎉 Status**: FULLY FUNCTIONAL - All commands tested on Sepolia testnet  
**🔧 Last Updated**: January 2025  
**🌐 Tested Network**: Sepolia with Alchemy bundler  
**💰 Live Proof**: Multiple successful transactions confirmed on-chain