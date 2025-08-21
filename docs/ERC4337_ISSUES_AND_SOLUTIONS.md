# 🔧 ERC-4337 Implementation Issues & Solutions

**Document Version:** 1.0  
**Date:** January 2025  
**Status:** ✅ All Issues Identified & Solutions Provided  

---

## 📋 **Executive Summary**

This document analyzes critical issues in our ERC-4337 account abstraction implementation and provides comprehensive solutions using aa-sdk-rs functionality. The main problem is that our `submit` command creates UserOperations that appear successful but don't actually execute transactions, while our `deploy-account` command works correctly.

**Key Finding:** We're mixing deployment logic with execution logic and not properly leveraging aa-sdk-rs capabilities.

---

## 🚨 **Issue #1: No Account Existence Validation**

### **Problem Description**
```rust
// ❌ BROKEN: We assume accounts exist without checking
let user_op_request = UserOperationBuilder::new(
    target_addr,        // Directly targeting external contract
    value_amount,       
    call_data_bytes     // Raw call data for external contract
);
```

**Symptoms:**
- UserOperations are accepted by bundler but fail during execution
- Account shows ETH balance but no contract code
- Nonce remains at 0 despite "successful" operations

### **Root Cause**
We never verify if the smart account is actually deployed before trying to use it for transactions.

### **✅ Solution: Use aa-sdk-rs Account State Checking**

```rust
// ✅ FIXED: Always check account deployment status
let simple_account = SimpleAccount::new(/*...*/);
let is_deployed = simple_account.is_account_deployed().await?;

if !is_deployed {
    let predicted_addr = simple_account.get_counterfactual_address().await?;
    return Err(anyhow::anyhow!(
        "❌ Smart account not deployed at {}!\n💡 Run deploy-account first",
        predicted_addr
    ));
}

let account_addr = simple_account.get_account_address().await?;
println!("✅ Using deployed smart account: {}", account_addr);
```

**Implementation Steps:**
1. Add account existence check to all transaction commands
2. Use `simple_account.is_account_deployed().await` before any operations
3. Provide clear error messages with deployment instructions

---

## 🚨 **Issue #2: Incorrect UserOperation Structure**

### **Problem Description**
```rust
// ❌ BROKEN: Wrong UserOperation structure for smart account execution
let user_op_request = UserOperationBuilder::new(
    target_addr,        // External contract address (WRONG!)
    value_amount,       // Direct value transfer (WRONG!)
    call_data_bytes     // Raw external contract call data (WRONG!)
);
```

**What's Wrong:**
- `sender` should be the smart account address, not external contract
- `callData` should be encoded `execute()` call, not raw external call data
- `value` should be 0 (smart account handles internal value transfer)

### **Root Cause**
We're creating UserOperations as if the EOA is directly calling external contracts, instead of the smart account executing calls.

### **✅ Solution: Proper Smart Account UserOperation Structure**

```rust
// ✅ FIXED: Correct UserOperation for smart account execution
use aa_sdk_rs::types::request::ExecuteCall;

// 1. Create ExecuteCall for the external transaction
let execute_call = ExecuteCall::new(
    target_addr,        // External contract to call
    value_amount,       // Value to send in the external call
    call_data_bytes     // Call data for the external contract
);

// 2. Encode it as smart account execute() call
let encoded_call_data = simple_account.encode_execute(execute_call).await?;

// 3. Create UserOperation with smart account as sender
let user_op_request = UserOperationBuilder::new(
    account_addr,                    // ✅ Smart account address as sender
    U256::ZERO,                      // ✅ No direct value transfer
    Bytes::from(encoded_call_data)   // ✅ Encoded execute() call
);
```

**Implementation Steps:**
1. Replace direct UserOperation creation with ExecuteCall encoding
2. Always use smart account address as sender
3. Use aa-sdk-rs `encode_execute()` for proper call data formatting

---

## 🚨 **Issue #3: Manual Nonce Management Conflicts**

### **Problem Description**
```rust
// ❌ BROKEN: Manual nonce setting conflicts with aa-sdk-rs
.with_nonce(U256::from(nonce))  // Manual nonce (probably wrong)
```

**Issues:**
- Manual nonces often conflict with actual account state
- aa-sdk-rs handles nonce automatically and correctly
- Mixed manual/automatic nonce management causes failures

### **Root Cause**
We're manually setting nonces instead of letting aa-sdk-rs handle nonce management automatically.

### **✅ Solution: Let aa-sdk-rs Handle Nonces**

```rust
// ✅ FIXED: Remove manual nonce management
let user_op_request = UserOperationBuilder::new(/*...*/)
    .with_gas_fees(max_fee, priority_fee)
    // ✅ NO .with_nonce() - let aa-sdk-rs handle it automatically
    .build();

// ✅ If you need to check nonce for debugging:
let current_nonce = simple_account.get_nonce().await?;
println!("📊 Current account nonce: {}", current_nonce);
```

**Implementation Steps:**
1. Remove all manual nonce setting from UserOperations
2. Let aa-sdk-rs SmartAccountProvider handle nonce management
3. Use `simple_account.get_nonce()` only for debugging/information

---

## 🚨 **Issue #4: Missing UserOperation Tracking**

### **Problem Description**
```rust
// ❌ BROKEN: No tracking of UserOperation status
match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
    Ok(user_op_hash) => {
        println!("✅ UserOperation submitted successfully!");
        // ❌ No tracking of actual execution status
    }
}
```

**Issues:**
- Can't tell if UserOperation actually executed
- No debugging information for failed operations
- "Success" only means bundler accepted the operation

### **Root Cause**
We're not using aa-sdk-rs tracking capabilities to monitor UserOperation execution.

### **✅ Solution: Full UserOperation Tracking**

```rust
// ✅ FIXED: Complete UserOperation tracking
match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
    Ok(user_op_hash) => {
        println!("✅ UserOperation submitted: {:?}", user_op_hash);
        
        // ✅ Track the actual execution
        match smart_provider.get_user_operation_receipt(user_op_hash).await {
            Ok(Some(receipt)) => {
                println!("✅ Transaction executed successfully!");
                println!("📋 Receipt: {:?}", receipt);
            }
            Ok(None) => {
                println!("⏳ Transaction still pending...");
                
                // ✅ Get more details
                if let Ok(Some(op)) = smart_provider.get_user_operation(user_op_hash).await {
                    println!("📊 UserOperation details: {:?}", op);
                }
            }
            Err(e) => {
                println!("❌ Failed to get receipt: {}", e);
            }
        }
    }
    Err(e) => {
        println!("❌ UserOperation submission failed: {}", e);
    }
}
```

**Implementation Steps:**
1. Always track UserOperation execution status
2. Use `get_user_operation_receipt()` to verify actual execution
3. Provide detailed status information for debugging

---

## 🚨 **Issue #5: Inadequate Gas Estimation**

### **Problem Description**
```rust
// ❌ BROKEN: Manual gas estimation or hardcoded values
.with_gas_fees(U256::from_str_radix("20000000000", 10)?, U256::from_str_radix("2000000000", 10)?)
```

**Issues:**
- Gas fees might be too low for current network conditions
- No dynamic gas estimation
- Missing proper gas limit estimation for UserOperation components

### **Root Cause**
We're not leveraging aa-sdk-rs gas estimation capabilities.

### **✅ Solution: Use aa-sdk-rs Gas Estimation**

```rust
// ✅ FIXED: Proper gas estimation
let mut user_op_request = UserOperationBuilder::new(/*...*/).build();

// ✅ Let aa-sdk-rs estimate gas parameters
let gas_estimates = smart_provider.estimate_user_operation_gas(&user_op_request).await?;
println!("📊 Gas estimates: {:?}", gas_estimates);

// ✅ Fill missing fields automatically
smart_provider.fill_user_operation(&mut user_op_request).await?;

// ✅ Then submit with proper estimates
let user_op_hash = smart_provider.send_user_operation(user_op_request, wallet.signer()).await?;
```

**Implementation Steps:**
1. Use `estimate_user_operation_gas()` before submission
2. Use `fill_user_operation()` to populate missing fields
3. Remove hardcoded gas values

---

## 🚨 **Issue #6: Deployment vs Execution Logic Confusion**

### **Problem Description**
Our `deploy-account` command works because it lets aa-sdk-rs handle everything automatically, but our `submit` command tries to manually manage the process.

### **Root Cause**
We have two different patterns:
- **Working pattern (deploy-account):** Let aa-sdk-rs handle everything
- **Broken pattern (submit):** Manual UserOperation construction

### **✅ Solution: Unified aa-sdk-rs Pattern**

```rust
// ✅ UNIFIED PATTERN: Always use aa-sdk-rs capabilities

// 1. ✅ For deployment (already working):
let user_op_request = UserOperationBuilder::new(
    predicted_address,  // Let aa-sdk-rs detect deployment need
    U256::ZERO,         
    Bytes::new()        
).build();

// 2. ✅ For execution (fixed pattern):
let execute_call = ExecuteCall::new(target, value, data);
let call_data = simple_account.encode_execute(execute_call).await?;
let user_op_request = UserOperationBuilder::new(
    account_address,    // Use deployed account address
    U256::ZERO,         
    Bytes::from(call_data)
).build();

// 3. ✅ Same submission pattern for both:
smart_provider.send_user_operation(user_op_request, wallet.signer()).await
```

---

## 🛠️ **Implementation Action Plan**

### **Phase 1: Fix Core Issues (High Priority)**

1. **✅ Update submit_user_operation function**
   - Add account existence checking
   - Use ExecuteCall encoding
   - Remove manual nonce management
   - Add UserOperation tracking

2. **✅ Update CLI parameters**
   - Add factory and salt parameters to submit command
   - These are needed to identify the smart account

3. **✅ Implement proper error handling**
   - Clear error messages for undeployed accounts
   - Better debugging information

### **Phase 2: Enhance Functionality (Medium Priority)**

1. **✅ Add gas estimation**
   - Use aa-sdk-rs gas estimation instead of hardcoded values
   - Implement dynamic gas price adjustment

2. **✅ Improve tracking**
   - Add receipt polling for pending operations
   - Better status reporting

3. **✅ Add validation**
   - Validate parameters before submission
   - Check account balance before operations

### **Phase 3: Code Cleanup (Low Priority)**

1. **✅ Remove redundant code**
   - Remove custom gas estimation
   - Remove manual nonce management
   - Consolidate UserOperation patterns

2. **✅ Improve error messages**
   - Add helpful suggestions
   - Include relevant addresses and parameters

---

## 📝 **Fixed Implementation Example**

Here's the complete fixed `submit_user_operation` function:

```rust
async fn submit_user_operation_fixed(
    private_key: &str,
    target: &str,
    call_data: &str,
    value: &str,
    factory: &str,      // ✅ Added: Need to identify smart account
    salt: &str,         // ✅ Added: Need to identify smart account
    rpc_url: &str,
    chain_id: u64,
    max_fee_per_gas: &str,
    max_priority_fee_per_gas: &str,
) -> Result<()> {
    // ✅ Setup
    let wallet = Wallet::from_hex(private_key)?;
    let factory_addr = Address::from_str(factory)?;
    let target_addr = Address::from_str(target)?;
    
    let url = url::Url::parse(rpc_url)?;
    let provider = ProviderBuilder::new().on_http(url);
    let entry_point_addr = Address::from_str("0x0000000071727De22E5E9d8BAf0edAc6f37da032")?;
    
    let simple_account = SimpleAccount::new(
        Arc::new(provider.clone()),
        wallet.address(),
        factory_addr,
        entry_point_addr,
        chain_id,
    );
    
    // ✅ 1. Check account deployment
    println!("🔍 Checking smart account deployment status...");
    let is_deployed = simple_account.is_account_deployed().await?;
    if !is_deployed {
        let predicted_addr = simple_account.get_counterfactual_address().await?;
        return Err(anyhow::anyhow!(
            "❌ Smart account not deployed at {}!\n💡 Run deploy-account first with factory {} and salt {}",
            predicted_addr, factory, salt
        ));
    }
    
    let account_addr = simple_account.get_account_address().await?;
    println!("✅ Using smart account: {}", account_addr);
    
    // ✅ 2. Encode transaction properly
    let call_data_bytes = if call_data.starts_with("0x") {
        Bytes::from_str(call_data)?
    } else {
        Bytes::from_str(&format!("0x{}", call_data))?
    };
    let value_amount = U256::from_str_radix(value, 10)?;
    
    let execute_call = ExecuteCall::new(target_addr, value_amount, call_data_bytes);
    let encoded_call_data = simple_account.encode_execute(execute_call).await?;
    
    // ✅ 3. Create proper UserOperation
    let mut user_op_request = UserOperationBuilder::new(
        account_addr,                    // Smart account as sender
        U256::ZERO,                      // No direct value transfer
        Bytes::from(encoded_call_data)   // Encoded execute() call
    )
    .with_gas_fees(
        U256::from_str_radix(max_fee_per_gas, 10)?,
        U256::from_str_radix(max_priority_fee_per_gas, 10)?
    )
    .build();
    
    // ✅ 4. Use aa-sdk-rs capabilities
    let smart_provider = SmartAccountProvider::new(provider, simple_account);
    
    // Optional: Get gas estimates
    if let Ok(estimates) = smart_provider.estimate_user_operation_gas(&user_op_request).await {
        println!("📊 Gas estimates: {:?}", estimates);
    }
    
    // Fill missing fields
    smart_provider.fill_user_operation(&mut user_op_request).await?;
    
    // ✅ 5. Submit with tracking
    println!("🚀 Submitting transaction via smart account...");
    match smart_provider.send_user_operation(user_op_request, wallet.signer()).await {
        Ok(user_op_hash) => {
            println!("✅ UserOperation submitted: {:?}", user_op_hash);
            
            // Track execution
            match smart_provider.get_user_operation_receipt(user_op_hash).await {
                Ok(Some(receipt)) => {
                    println!("✅ Transaction executed successfully!");
                    println!("📋 Receipt: {:?}", receipt);
                }
                Ok(None) => {
                    println!("⏳ Transaction pending - check later with hash: {:?}", user_op_hash);
                }
                Err(e) => {
                    println!("⚠️  Could not verify execution: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Transaction failed: {}", e);
        }
    }
    
    Ok(())
}
```

---

## 📋 **Immediate Next Steps**

1. **✅ Update CLI command structure** to include factory and salt parameters for submit
2. **✅ Implement the fixed submit_user_operation function**
3. **✅ Test with your existing deployed smart account**
4. **✅ Add similar validation to other transaction commands**
5. **✅ Remove redundant custom code** and leverage aa-sdk-rs fully

---

## 🎯 **Success Criteria**

After implementing these fixes, you should see:

- ✅ **submit command works correctly** for deployed accounts
- ✅ **Clear error messages** when account isn't deployed
- ✅ **Actual transaction execution** instead of just bundler acceptance
- ✅ **Proper nonce incrementation** after successful transactions
- ✅ **Real-time status tracking** of UserOperations
- ✅ **Better gas estimation** and fee management

The core principle: **Stop fighting aa-sdk-rs and start leveraging its capabilities fully!**
