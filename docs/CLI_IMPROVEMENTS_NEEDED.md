# 🔧 CLI Improvements Needed for Bundler Integration

**Status**: Required changes to support proper ERC-4337 bundler integration

## 🎯 **Critical Issues Identified**

### **1. Missing Gas Fee Parameters**

**Problem**: The bundler rejected our UserOperation with:
```
maxPriorityFeePerGas is 1000000 but must be at least 100000000
```

**Current CLI**: No gas fee configuration options
```rust
Submit {
    // ... existing fields ...
    // ❌ MISSING: max_fee_per_gas
    // ❌ MISSING: max_priority_fee_per_gas
}
```

**Required Addition**:
```rust
Submit {
    // ... existing fields ...
    
    /// Maximum fee per gas (in wei)
    #[arg(long, default_value = "20000000000")]  // 20 gwei
    max_fee_per_gas: String,
    
    /// Maximum priority fee per gas (in wei)  
    #[arg(long, default_value = "2000000000")]   // 2 gwei
    max_priority_fee_per_gas: String,
}
```

### **2. UserOperation Builder Not Using Gas Fees**

**Problem**: We have `with_gas_fees()` but it's marked as `#[allow(dead_code)]` and never used.

**Current Code**:
```rust
// This function exists but is never called
pub fn with_gas_fees(mut self, max_fee_per_gas: U256, max_priority_fee_per_gas: U256) -> Self {
    // Implementation exists but unused
}
```

**Required Fix**: Actually use the gas fees in the submit function.

### **3. Default Gas Values Too Low**

**Problem**: Default values in aa-sdk-rs are too low for current network conditions.

**Current**: Defaults to very low gas fees
**Required**: Network-appropriate defaults

## 🛠 **Required Changes**

### **Change 1: Add Gas Fee CLI Arguments**

```rust
// In main.rs Submit struct
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
    #[arg(short, long, default_value = "0x0000000071727De22E5E9d8BAf0edAc6f37da032")]
    entry_point: String,
    
    /// Chain ID
    #[arg(short, long, default_value = "31337")]
    chain_id: u64,
    
    // NEW: Gas fee parameters
    /// Maximum fee per gas (in wei)
    #[arg(long, default_value = "20000000000")]  // 20 gwei
    max_fee_per_gas: String,
    
    /// Maximum priority fee per gas (in wei)
    #[arg(long, default_value = "2000000000")]   // 2 gwei  
    max_priority_fee_per_gas: String,
},
```

### **Change 2: Update submit_user_operation Function**

```rust
// Update function signature
async fn submit_user_operation(
    private_key: &str,
    target: &str,
    call_data: &str,
    nonce: u64,
    rpc_url: &str,
    entry_point: &str,
    chain_id: u64,
    max_fee_per_gas: &str,           // NEW
    max_priority_fee_per_gas: &str,  // NEW
) -> Result<()> {
    // Parse gas fees
    let max_fee = U256::from_str_radix(max_fee_per_gas, 10)?;
    let priority_fee = U256::from_str_radix(max_priority_fee_per_gas, 10)?;
    
    // Use gas fees in UserOperation
    let user_op_request = UserOperationBuilder::new(
        target_addr,
        U256::ZERO,
        call_data_bytes.clone()
    )
    .with_nonce(U256::from(nonce))
    .with_gas_fees(max_fee, priority_fee)  // ADD THIS
    .build();
    
    // ... rest of function
}
```

### **Change 3: Update CLI Command Mapping**

```rust
// In main() function
Commands::Submit { 
    private_key, 
    target, 
    call_data, 
    nonce, 
    rpc_url, 
    entry_point, 
    chain_id,
    max_fee_per_gas,           // NEW
    max_priority_fee_per_gas,  // NEW
} => {
    submit_user_operation(
        private_key, 
        target, 
        call_data, 
        *nonce, 
        rpc_url, 
        entry_point, 
        *chain_id,
        max_fee_per_gas,         // NEW
        max_priority_fee_per_gas // NEW
    ).await?;
}
```

### **Change 4: Remove Dead Code Warning**

```rust
// In userop.rs - remove #[allow(dead_code)]
pub fn with_gas_fees(mut self, max_fee_per_gas: U256, max_priority_fee_per_gas: U256) -> Self {
    self.request = self.request
        .max_fee_per_gas(max_fee_per_gas)
        .max_priority_fee_per_gas(max_priority_fee_per_gas);
    self
}
```

## 🧪 **Testing the Fix**

After implementing these changes, test with:

```bash
cargo run -- submit \
  --private-key 0x5e8e908927541a91aaef9287b95af7cc6097850164f1a1323065e0013d893552 \
  --target 0xd59c5D74A376f08E3036262F1D59Be24dE138c41 \
  --call-data 0x \
  --nonce 0 \
  --rpc-url https://eth-sepolia.g.alchemy.com/v2/fR-hJP75SP2jofREXQ092 \
  --chain-id 11155111 \
  --max-fee-per-gas 20000000000 \
  --max-priority-fee-per-gas 2000000000
```

## 📋 **Network-Specific Defaults**

Consider adding network-specific gas defaults:

| Network | Max Fee (gwei) | Priority Fee (gwei) |
|---------|----------------|-------------------|
| Sepolia | 20 | 2 |
| Mainnet | 30 | 2 |
| Local/Anvil | 1 | 0.1 |

## 🎯 **Priority Level**

**Priority**: 🔴 **CRITICAL** - This blocks the final bundler integration test

**Estimated Time**: 30 minutes to implement

**Impact**: Enables complete ERC-4337 bundler flow functionality

---

**Note**: Once these changes are implemented, we should have a fully working ERC-4337 Account Abstraction system! 🚀
