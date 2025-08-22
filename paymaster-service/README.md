# 🚀 Paymaster Service

Admin API service for managing verifier keys and signing gas sponsorship requests.

## 🏗️ Architecture

This service provides:
- **Key Management**: Secure storage of verifier private keys
- **Signature Generation**: ECDSA signing for gas sponsorship
- **API Endpoints**: HTTP interface for client integration

## 🚀 Quick Start

### 1. Build the Service
```bash
cargo build --release
```

### 2. Configure Verifier Keys
Edit `config/default.toml` with your verifier private keys:
```toml
[verifier_keys]
default = "your_verifier_private_key_here"
admin = "your_admin_verifier_key_here"
```

### 3. Run the Service
```bash
cargo run --release
```

The service will start on port 3000.

## 📡 API Endpoints

### Health Check
```bash
GET /health
```

### Sign Sponsorship Request
```bash
POST /sign
Content-Type: application/json

{
  "user_operation_hash": "0x...",
  "max_gas_cost": "1000000",
  "valid_until": 1234567890,
  "verifier": "default"
}
```

### Get Metrics
```bash
GET /metrics
```

## 🔐 Security

- **Private Keys**: Never expose verifier private keys
- **Network Access**: Restrict service to trusted networks
- **Rate Limiting**: Implement abuse prevention
- **Monitoring**: Log all signature requests

## 🚨 Environment Variables

Override configuration with environment variables:
```bash
export PAYMASTER_SERVER_PORT=3000
export PAYMASTER_LOG_LEVEL=debug
export PAYMASTER_VERIFIER_KEYS_DEFAULT="your_key_here"
```

## 🧪 Testing

```bash
cargo test
```

## 📦 Dependencies

- **axum**: HTTP server framework
- **secp256k1**: ECDSA cryptography
- **serde**: Serialization
- **tokio**: Async runtime
