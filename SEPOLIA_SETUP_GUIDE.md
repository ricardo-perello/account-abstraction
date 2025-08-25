# 🌐 Sepolia Sponsored Transactions Setup Guide

This guide sets up the complete sponsored transaction system on Sepolia testnet using your existing deployed contracts.

## 📋 Prerequisites

You already have:
- ✅ **EntryPoint**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032`
- ✅ **AAAccountFactory**: `0x59bcaa1BB72972Df0446FCe98798076e718E3b61`
- ✅ Working smart accounts and transactions

We need to add:
- 🚀 **VerifierSignaturePaymaster** contract
- 🌐 **Paymaster service** running on Sepolia
- 🎯 **Sponsored transaction commands**

## 🚀 Quick Setup (Automated)

```bash
# Run the automated setup script
chmod +x setup-sepolia.sh
./setup-sepolia.sh
```

## 📖 Manual Setup Steps

### Step 1: Deploy Paymaster Contract

```bash
cd contracts

# Deploy to Sepolia
forge script script/DeployPaymasterSepolia.s.sol \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_API_KEY \
  -vvvv
```

**Expected Output:**
```
✅ Paymaster deployed at: 0xYOUR_PAYMASTER_ADDRESS
✅ Paymaster funded with 0.05 ETH
✅ EntryPoint deposit: 0.05 ETH
```

### Step 2: Update Paymaster Service Config

Edit `paymaster-service/config/sepolia.toml`:

```toml
# Update this line with your deployed address
paymaster_address = "0xYOUR_PAYMASTER_ADDRESS_FROM_STEP_1"
```

### Step 3: Build Client

```bash
cd client
cargo build --release
```

### Step 4: Start Paymaster Service

```bash
cd paymaster-service

# Start service with Sepolia config
cargo run -- --config config/sepolia.toml
```

Service will run on `http://localhost:3000`

### Step 5: Test Sponsored Account Creation

```bash
cd client

# Deploy sponsored smart account (zero gas fees!)
./target/release/aa-client deploy-sponsored \
  --private-key $PRIVATE_KEY \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x1234567890abcdef \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --chain-id 11155111 \
  --paymaster-url http://localhost:3000 \
  --paymaster-api-key sepolia_test_key_123 \
  --paymaster-address YOUR_DEPLOYED_PAYMASTER_ADDRESS
```

### Step 6: Execute Sponsored Transaction

```bash
# Send sponsored transaction (zero gas fees!)
./target/release/aa-client submit-sponsored \
  --private-key $PRIVATE_KEY \
  --target 0xd59c5D74A376f08E3036262F1D59Be24dE138c41 \
  --call-data 0x \
  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
  --salt 0x1234567890abcdef \
  --rpc-url $ALCHEMY_HTTP_SEPOLIA \
  --chain-id 11155111 \
  --value 1000000000000000 \
  --paymaster-url http://localhost:3000 \
  --paymaster-api-key sepolia_test_key_123 \
  --paymaster-address YOUR_DEPLOYED_PAYMASTER_ADDRESS
```

## 🔑 Configuration Details

### Sepolia Network Config
- **Chain ID**: `11155111`
- **RPC URL**: `$ALCHEMY_HTTP_SEPOLIA`
- **EntryPoint**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032`
- **Factory**: `0x59bcaa1BB72972Df0446FCe98798076e718E3b61`

### Test Credentials
- **Test Private Key**: `0x9ec161507ad1cfd507ae6e6bf012a66d609276782ae64f70ca41174d402d10ae`
- **Test Address**: `0x21D541ef2237b2a63076666651238AC8A7cde752`
- **⚠️ WARNING**: Test key only - never use with real funds!

### API Keys Available
- `sepolia_test_key_123` - General testing
- `demo_key_456` - Demo applications  
- `ricardo_bot_789` - Your trading bot

## 📊 Expected Results

### ✅ Successful Deployment
```
🎉 Deploying sponsored smart account via paymaster...
✅ Deployment sponsorship approved!
📋 Paymaster will cover all deployment costs
🚀 Submitting sponsored deployment...
✅ Sponsored deployment initiated successfully!
💰 Deployment costs are being sponsored!
🎉 Your smart account is being deployed with zero gas fees!
```

### ✅ Successful Transaction
```
🎉 Submitting sponsored transaction via paymaster...
💰 Requesting paymaster sponsorship...
✅ Paymaster sponsorship obtained!
🚀 Submitting sponsored UserOperation...
✅ Sponsored transaction submitted successfully!
💰 Gas fees are being sponsored by the paymaster!
🎉 Gas-free transaction completed!
```

## 🔧 Troubleshooting

### Paymaster Service Issues
```bash
# Check service logs
tail -f paymaster-service/paymaster.log

# Restart service
pkill -f paymaster-service
cd paymaster-service && cargo run -- --config config/sepolia.toml
```

### Contract Verification
```bash
# Verify paymaster is deployed
cast code YOUR_PAYMASTER_ADDRESS --rpc-url $ALCHEMY_HTTP_SEPOLIA

# Check paymaster balance
cast balance YOUR_PAYMASTER_ADDRESS --rpc-url $ALCHEMY_HTTP_SEPOLIA --ether
```

### Common Errors

**"Invalid verifier signature"**
- Ensure paymaster address in config matches deployed contract
- Verify test private key is correct

**"Paymaster deposit too low"**
- Fund paymaster with more ETH: `cast send YOUR_PAYMASTER_ADDRESS --value 0.1ether --rpc-url $ALCHEMY_HTTP_SEPOLIA --private-key $PRIVATE_KEY`

**"Smart account not deployed"**
- Run `deploy-sponsored` command first before `submit-sponsored`

## 🎯 Test Scenarios

### Scenario 1: Deploy Sponsored Account
- ✅ User pays **zero gas fees**
- ✅ Paymaster covers deployment costs
- ✅ Smart account created successfully

### Scenario 2: Execute Sponsored Transaction
- ✅ User pays **zero gas fees**
- ✅ Paymaster covers execution costs
- ✅ Transaction executes successfully

### Scenario 3: Multiple Sponsored Operations
- ✅ Deploy account with zero gas
- ✅ Execute multiple transactions with zero gas
- ✅ All costs sponsored by paymaster

## 🛡️ Security Notes

- 🔴 **Test Environment Only**: This setup uses test keys
- 🟡 **Paymaster Funds**: Monitor balance for continued operation
- 🟢 **API Keys**: Change default keys for production
- 🔵 **Rate Limiting**: Consider implementing request limits

## 📈 Monitoring

### Service Health
```bash
# Check service status
curl http://localhost:3000/health

# View metrics
curl http://localhost:3000/metrics
```

### Transaction Monitoring
```bash
# Monitor Sepolia transactions
# Use Etherscan Sepolia: https://sepolia.etherscan.io/
```

---

## 🎉 Success!

Once setup is complete, you'll have:
- ✅ **Sponsored account deployment** - Users pay zero gas
- ✅ **Sponsored transactions** - Users pay zero gas  
- ✅ **Full ERC-4337 compliance** - Works with any bundler
- ✅ **Production-ready architecture** - Scalable and secure

**Enjoy gas-free transactions on Sepolia!** 🚀
