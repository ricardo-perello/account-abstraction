# 🚀 ERC-4337 CLI - Quick Reference Card

**Status**: ✅ **100% WORKING** - Full bundler integration achieved!

## 🎯 **Essential Commands**

### **Deploy Smart Account**
```bash
cargo run -- deploy-account \
  --private-key YOUR_KEY \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x00 \
  --rpc-url YOUR_RPC \
  --chain-id 11155111
```

### **Submit to Bundler (ERC-4337)**
```bash
cargo run -- submit \
  --private-key YOUR_KEY \
  --target 0xTARGET \
  --call-data 0xDATA \
  --nonce 0 \
  --rpc-url YOUR_RPC \
  --chain-id 11155111 \
  --max-fee-per-gas 20000000000 \
  --max-priority-fee-per-gas 2000000000
```

### **Predict Account Address**
```bash
cargo run -- predict-address \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --owner 0xOWNER \
  --salt 0x00 \
  --rpc-url YOUR_RPC \
  --chain-id 11155111
```

## 🌐 **Network Configs**

### **Sepolia (Testnet)**
```bash
--chain-id 11155111
--rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
--entry-point 0x0000000071727De22E5E9dBAf0edAc6f37da032
--factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61
```

### **Local Anvil**
```bash
--chain-id 31337
--rpc-url http://localhost:8545
--entry-point 0x0000000071727De22E5E9dBAf0edAc6f37da032
--factory 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
```

## ⚙️ **Gas Fee Defaults**

```bash
--max-fee-per-gas 20000000000      # 20 gwei
--max-priority-fee-per-gas 2000000000   # 2 gwei
```

## 🔄 **Complete Workflow**

```bash
# 1. Predict address
cargo run -- predict-address --factory 0x... --owner 0x... --salt 0x00 --rpc-url ... --chain-id 11155111

# 2. Fund predicted address with ETH

# 3. Submit (auto-deploys account)
cargo run -- submit --private-key 0x... --target 0x... --call-data 0x --nonce 0 --rpc-url ... --chain-id 11155111
```

## 🧪 **Testing**

```bash
# Demo
cargo run -- demo --yes

# Generate wallet
cargo run -- generate-wallet

# Show networks
cargo run -- networks
```

## 🚨 **Troubleshooting**

- **Gas fees too low**: Use `--max-priority-fee-per-gas 2000000000`
- **Insufficient balance**: Fund the predicted account address
- **Network issues**: Check RPC URL and chain ID

---

**Full Documentation**: See `USER_GUIDE.md` for complete details
**Technical Details**: See `BUNDLER_BREAKTHROUGH.md`
**Development History**: See `CLI_IMPROVEMENTS_NEEDED.md`
