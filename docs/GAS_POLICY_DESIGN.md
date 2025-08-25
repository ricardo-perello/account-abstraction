# Gas Policy Design & Client Identification

This document explains how gas policies work in the paymaster service and how client applications identify themselves to receive sponsored transactions.

## Overview

The paymaster service now supports **automatic gas policy enforcement** instead of requiring manual admin approval for each transaction. Here's how it works:

1. **Client apps register** with the paymaster service and receive a policy
2. **Client apps identify themselves** in sponsorship requests
3. **Paymaster validates** the request against the policy
4. **If approved**, the paymaster signs the sponsorship

## Gas Policy Structure

A gas policy defines the rules for sponsoring transactions:

```rust
pub struct GasPolicy {
    pub policy_id: String,                    // Unique policy identifier
    pub client_id: String,                    // Client application identifier
    pub name: String,                         // Human-readable name
    pub max_gas_cost_per_tx: U256,           // Max gas per single transaction
    pub max_total_gas_cost: U256,            // Max total gas per time window
    pub time_window_seconds: u64,            // Time window (e.g., 24 hours)
    pub max_transactions_per_window: u64,    // Max transactions per window
    pub allowed_targets: Vec<Address>,        // Allowed contract addresses (empty = all)
    pub allowed_function_selectors: Vec<[u8; 4]>, // Allowed functions (empty = all)
    pub expires_at: DateTime<Utc>,           // Policy expiration
    pub is_active: bool,                     // Whether policy is active
    pub metadata: HashMap<String, String>,   // Additional metadata
}
```

## Client Identification Methods

The paymaster supports multiple ways for client apps to identify themselves:

### 1. API Key Based (Recommended)

**Best for:** Backend services, server-side applications

```rust
ClientIdentification::ApiKey {
    api_key: "pk_live_1234567890abcdef",
    client_name: "My DeFi App",
}
```

**How it works:**
- Admin generates API key for client
- Client includes API key in all requests
- Paymaster maps API key to policy

### 2. Domain Based

**Best for:** Frontend web applications

```rust
ClientIdentification::Domain {
    domain: "app.mydefi.com",
    verification_method: DomainVerificationMethod::DnsTxt,
}
```

**How it works:**
- Client registers their domain
- Domain ownership verified via DNS TXT record or HTTPS callback
- Paymaster checks domain against policy

### 3. Smart Account Based

**Best for:** Account abstraction apps where the smart account identifies the app

```rust
ClientIdentification::SmartAccount {
    account_address: "0x1234...abcd",
    app_identifier: "mydefi_v2",
}
```

**How it works:**
- Specific smart account addresses are mapped to app policies
- Useful when one app controls specific sets of smart accounts

### 4. Signature Based

**Best for:** Apps that can maintain cryptographic keys

```rust
ClientIdentification::SignatureBased {
    public_key: "0x04abcd...",
    client_name: "Mobile Wallet",
}
```

**How it works:**
- Client signs the request with their private key
- Paymaster verifies signature against registered public key

## Example Usage

### 1. Register a Policy (Admin)

```bash
curl -X POST http://localhost:3000/policies \
  -H "Content-Type: application/json" \
  -d '{
    "policy_id": "web_mydefi_com",
    "client_id": "mydefi.com",
    "name": "MyDeFi Web App Policy",
    "max_gas_cost_per_tx": "500000000000000",
    "max_total_gas_cost": "5000000000000000",
    "time_window_seconds": 86400,
    "max_transactions_per_window": 100,
    "allowed_targets": [],
    "allowed_function_selectors": [],
    "expires_at": "2024-12-31T23:59:59Z",
    "is_active": true,
    "metadata": {}
  }'
```

### 2. Request Sponsorship (Client App)

```bash
curl -X POST http://localhost:3000/sign-sponsored \
  -H "Content-Type: application/json" \
  -d '{
    "user_operation_hash": "0xabcd1234...",
    "max_gas_cost": "300000000000000",
    "valid_until": 1703721600,
    "target_contract": "0x1234567890abcdef1234567890abcdef12345678",
    "call_data": "0xa9059cbb000000000000000000000000742d35cc...",
    "client_identification": {
      "ApiKey": {
        "api_key": "pk_live_1234567890abcdef",
        "client_name": "MyDeFi App"
      }
    },
    "smart_account": "0xabcdef1234567890abcdef1234567890abcdef12",
    "context": {}
  }'
```

### 3. Check Policy Usage

```bash
curl http://localhost:3000/policies/web_mydefi_com/usage
```

## Implementation in Client

Here's how you would integrate this into your client application:

### Rust Client Example

```rust
use serde_json::json;

// Create sponsorship request
let request = json!({
    "user_operation_hash": user_op_hash,
    "max_gas_cost": estimated_gas,
    "valid_until": (chrono::Utc::now().timestamp() + 3600) as u64, // 1 hour from now
    "target_contract": target_address,
    "call_data": hex::encode(&call_data),
    "client_identification": {
        "ApiKey": {
            "api_key": "your_api_key_here",
            "client_name": "Your App Name"
        }
    },
    "smart_account": smart_account_address,
    "context": {}
});

// Send request to paymaster
let response = reqwest::Client::new()
    .post("http://your-paymaster-service.com/sign-sponsored")
    .json(&request)
    .send()
    .await?;

if response.status().is_success() {
    let sponsorship: SponsorshipResponse = response.json().await?;
    // Use sponsorship.signature and sponsorship.paymaster_data in your UserOperation
    println!("Sponsorship approved! Signature: {}", sponsorship.signature);
} else {
    println!("Sponsorship denied: {}", response.text().await?);
}
```

## Policy Examples

### Conservative Web App Policy
```rust
let policy = PolicyEngine::create_web_app_policy(
    "myapp.com",
    U256::from(100_000_000_000_000_u64), // 0.0001 ETH per tx
    U256::from(1_000_000_000_000_000_u64), // 0.001 ETH per day
    20, // 20 transactions per day
);
```

### DeFi App with Specific Contract Access
```rust
let allowed_contracts = vec![
    Address::from_str("0x1234567890abcdef1234567890abcdef12345678")?, // DEX
    Address::from_str("0xabcdef1234567890abcdef1234567890abcdef12")?, // Token
];

let allowed_functions = vec![
    [0xa9, 0x05, 0x9c, 0xbb], // transfer(address,uint256)
    [0x23, 0xb8, 0x72, 0xdd], // transferFrom(address,address,uint256)
];

let policy = PolicyEngine::create_defi_app_policy(
    "defi_app_v1",
    allowed_contracts,
    allowed_functions,
);
```

## Benefits

1. **Automated**: No manual approval needed for each transaction
2. **Flexible**: Multiple identification methods support different app architectures
3. **Secure**: Policies enforce spending limits and contract restrictions
4. **Scalable**: Can handle many client apps with different policies
5. **Auditable**: Full usage tracking and analytics

## Next Steps

1. **Policy Storage**: Currently in-memory, should be persisted to database
2. **Admin UI**: Web interface for managing policies
3. **Analytics**: Dashboard for policy usage and spending
4. **Rate Limiting**: Additional protection against abuse
5. **Multi-signature**: Require multiple approvals for high-value policies

This design gives you a robust foundation for automated gas sponsorship while maintaining security and control over spending.
