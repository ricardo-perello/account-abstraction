# Account Abstraction Deployment Info

## 🌐 Network Information

- **Network**: Anvil Local Testnet
- **RPC URL**: http://localhost:8545
- **Chain ID**: 31337
- **Current Block**: 6+

## 📜 Deployed Smart Contracts

### Core Contracts

| Contract | Address | Purpose | Deployment Tx |
|----------|---------|---------|---------------|
| **EntryPoint** | `0x5FbDB2315678afecb367f032d93F642f64180aa3` | ERC-4337 core contract | `0x09b1d91cdf70a4e0acc5790350a45c8526ff71bf8c9ead0084ef8f46daa6d086` |
| **AAAccountFactory** | `0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512` | Smart wallet factory | `0x8ca3f8a7f1b00efae769f63c6cf82911497b03052848bfd53c5d5296d7542d9d` |

### Deployed Smart Wallets

| Wallet Type | Address | Owners | Deployment Tx |
|-------------|---------|--------|---------------|
| **Single-Owner** | `0x5501533DfA9bca0E7A0b7Ef7bacb7DED62298909` | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `0xb29b8c4ea1a54943ecc2577f0ec3b9b57aaee9e30896befa0c735233e131abfa` |
| **Multi-Owner** | `0x24D031c83cCEA25714d85Ab724f6553F370af028` | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8`<br/>`0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | `0x6c81ada75bcd3ac460786797d567c0014392951401c72195ad1a0a56da4afcec` |

## 🔐 Test Accounts & Private Keys

### Pre-funded Anvil Accounts

| Role | Address | Private Key | Balance |
|------|---------|-------------|---------|
| **Deployer (Account #0)** | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80` | ~9999.99 ETH |
| **Owner1 (Account #1)** | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d` | 10000 ETH |
| **Owner2 (Account #2)** | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | `0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a` | 10000 ETH |
| **Available (Account #3)** | `0x90F79bf6EB2c4f870365E785982E1f101E93b906` | `0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6` | 10000 ETH |
| **Available (Account #4)** | `0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65` | `0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a` | 10000 ETH |

## ⚡ Usage Examples

### Using Cast Commands

#### Create a New Single-Owner Wallet
```bash
cast send --rpc-url http://localhost:8545 \
  --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
  0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
  "createAccount(address,bytes32)" \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  0x123456
```

#### Create a Multi-Owner Wallet
```bash
# Note: This requires encoding the address array - use the factory script instead
forge script script/Interact.s.sol --rpc-url http://localhost:8545 --broadcast
```

#### Check Wallet Owner Count
```bash
cast call --rpc-url http://localhost:8545 \
  0x5501533DfA9bca0E7A0b7Ef7bacb7DED62298909 \
  "ownerCount()"
```

#### Check if Address is Owner
```bash
cast call --rpc-url http://localhost:8545 \
  0x5501533DfA9bca0E7A0b7Ef7bacb7DED62298909 \
  "owners(address)" \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

#### Add Owner to Existing Wallet
```bash
cast send --rpc-url http://localhost:8545 \
  --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
  0x5501533DfA9bca0E7A0b7Ef7bacb7DED62298909 \
  "addOwner(address)" \
  0x90F79bf6EB2c4f870365E785982E1f101E93b906
```

#### Remove Owner from Wallet
```bash
cast send --rpc-url http://localhost:8545 \
  --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
  0x5501533DfA9bca0E7A0b7Ef7bacb7DED62298909 \
  "removeOwner(address)" \
  0x90F79bf6EB2c4f870365E785982E1f101E93b906
```

### Using Forge Scripts

#### Deploy Fresh Contracts
```bash
forge script script/Deploy.s.sol \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

#### Run Interactive Demo
```bash
forge script script/Interact.s.sol \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

## 🧪 Testing

### Run All Tests
```bash
forge test --rpc-url http://localhost:8545 -vv
```

### Test Results
- ✅ **10/10 tests passing**
- ✅ All core functionality verified
- ✅ Owner management working
- ✅ Multi-signature capabilities tested
- ✅ CREATE2 deterministic addresses working

## 🔍 Contract Verification

### Check Contract Deployment
```bash
# Verify EntryPoint is deployed
cast code --rpc-url http://localhost:8545 0x5FbDB2315678afecb367f032d93F642f64180aa3

# Verify Factory is deployed  
cast code --rpc-url http://localhost:8545 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512

# Verify Smart Wallet is deployed
cast code --rpc-url http://localhost:8545 0x5501533DfA9bca0E7A0b7Ef7bacb7DED62298909
```

### Check Network Status
```bash
# Current block number
cast block-number --rpc-url http://localhost:8545

# Account balances
cast balance --rpc-url http://localhost:8545 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --ether
```

## 🚀 Next Steps

### For Development
1. **Test more features** - Try different owner combinations
2. **Add transaction execution** - Implement `execute()` calls
3. **Test gas optimization** - Measure deployment and execution costs
4. **Add batch operations** - Test `executeBatch()` functionality

### For Production
1. **Deploy to testnet** - Use Sepolia or Goerli
2. **Verify contracts** - Get verified on Etherscan
3. **Create frontend** - Build a UI for wallet management
4. **Add security features** - Implement spending limits, time delays

## 📁 Project Structure

```
contracts/
├── src/
│   ├── AAAccount.sol           # Main smart wallet contract
│   └── AAAccountFactory.sol    # Factory for creating wallets
├── script/
│   ├── Deploy.s.sol           # Deployment script
│   └── Interact.s.sol         # Interactive demo script
├── test/
│   └── AAAccount.t.sol        # Comprehensive test suite
└── DEPLOYMENT_INFO.md         # This file
```

## ⚠️ Security Notes

- 🔴 **NEVER use these private keys on mainnet or with real funds**
- 🟡 **These are test keys for Anvil development only**
- 🟢 **Always generate new keys for production deployments**
- 🔵 **Consider hardware wallets for production key management**

---

**Generated**: $(date)  
**Status**: ✅ Fully deployed and tested on Anvil local network
