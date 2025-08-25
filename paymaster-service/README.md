# Simple Paymaster Service

A minimal paymaster service for ERC-4337 account abstraction that provides gas sponsorship with API key authentication.

## Features

- ✅ **API Key Authentication** - Simple client identification
- ✅ **Gas Sponsorship Signing** - Signs sponsorship requests for valid clients  
- ✅ **Health & Metrics** - Basic monitoring endpoints
- ✅ **Simple Configuration** - TOML-based config with verifier keys and API keys

## Quick Start

### 1. Configuration

Edit `config/default.toml`:

```toml
# Server settings
server_port = 3000
log_level = "info"

# Blockchain settings
chain_id = 1  # Ethereum mainnet (change to 11155111 for Sepolia testnet)
paymaster_address = "0x0000000000000000000000000000000000000000"  # Replace with deployed paymaster address

# Verifier keys (hex format, no 0x prefix)
[verifier_keys]
default = "your_verifier_private_key_here"

# API keys for client authentication
[api_keys]
"your_api_key_123" = "Your App Name"
"another_api_key" = "Another App Name"
```

### 2. Run the Service

```bash
cargo run
```

The service starts on `http://localhost:3000`

### 3. Test with your Client

From your `../client/` directory:

```bash
# Example: Sign a sponsorship request
curl -X POST http://localhost:3000/sign \
  -H "Content-Type: application/json" \
  -d '{
    "api_key": "your_api_key_123",
    "user_operation": {
      "sender": "0x1234567890123456789012345678901234567890",
      "nonce": "1",
      "init_code": "0x",
      "call_data": "0x1234",
      "account_gas_limits": "0x00000000000f424000000000000f4240",
      "pre_verification_gas": "21000",
      "gas_fees": "0x000000000077359400000000003b9aca00",
      "paymaster_and_data": "0x"
    },
    "valid_until": 1735689600,
    "valid_after": 0
  }'
```

## API Endpoints

### `POST /sign`

Signs a sponsorship request after validating the API key.

**Request:**
```json
{
  "api_key": "your_api_key_123",
  "user_operation": {
    "sender": "0x1234567890123456789012345678901234567890",
    "nonce": "1",
    "init_code": "0x",
    "call_data": "0x1234",
    "account_gas_limits": "0x00000000000f424000000000000f4240",
    "pre_verification_gas": "21000",
    "gas_fees": "0x000000000077359400000000003b9aca00",
    "paymaster_and_data": "0x"
  },
  "valid_until": 1735689600,
  "valid_after": 0
}
```

**Response:**
```json
{
  "signature": "0x...",
  "valid_until": 1735689600,
  "valid_after": 0,
  "paymaster_data": "0x..."
}
```

### `GET /health`

Returns `200 OK` if service is healthy.

### `GET /metrics`

Returns service metrics:
```json
{
  "verifier_count": 1,
  "service_status": "healthy"
}
```

## Integration with Client

Add to your client commands:

```bash
# Option 1: Command line flag
aa-client submit \
  --paymaster-api-key "your_api_key_123" \
  --paymaster-url "http://localhost:3000" \
  --private-key "0x..." \
  --target "0x..." \
  --call-data "0x..."

# Option 2: Environment variable
export PAYMASTER_API_KEY="your_api_key_123"
export PAYMASTER_URL="http://localhost:3000"
aa-client submit --private-key "0x..." --target "0x..." --call-data "0x..."
```

## Development

### Run Tests

```bash
cargo test
```

### Generate New Keys

```bash
# Generate a new secp256k1 private key (64 hex chars)
openssl rand -hex 32
```

## Security Notes

⚠️ **Important:**
- Replace example keys with secure, randomly generated keys
- Keep your `config/default.toml` secure and never commit real keys to version control
- Use environment variables or secure key management for production deployments
- The example keys in `config/default.toml` are placeholders only

## What's Different from Before

This implementation is **ERC-4337 compliant** and includes:
- ✅ Proper paymaster signature format (matches AnvilIntegrationTest.s.sol)
- ✅ Compatible with VerifierSignaturePaymaster.sol contract
- ✅ Full `_pmHash` implementation with chain ID and paymaster address binding
- ✅ EIP-191 message formatting for signature verification
- ✅ Simple API key authentication
- ✅ ~400 lines of secure, tested code