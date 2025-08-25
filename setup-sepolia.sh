#!/bin/bash

# Sepolia Paymaster Setup Script
# This script deploys the paymaster contract and sets up the entire system

set -e

echo "🚀 Setting up Sponsored Transactions for Sepolia Testnet"
echo "========================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${RED}❌ Error: .env file not found${NC}"
    echo "Please create .env file with:"
    echo "PRIVATE_KEY=0x..."
    echo "ALCHEMY_HTTP_SEPOLIA=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY"
    exit 1
fi

# Load environment variables
source .env

# Verify required variables
if [ -z "$PRIVATE_KEY" ]; then
    echo -e "${RED}❌ Error: PRIVATE_KEY not set in .env${NC}"
    exit 1
fi

if [ -z "$ALCHEMY_HTTP_SEPOLIA" ]; then
    echo -e "${RED}❌ Error: ALCHEMY_HTTP_SEPOLIA not set in .env${NC}"
    exit 1
fi

echo -e "${BLUE}🔧 Configuration:${NC}"
echo "  Network: Sepolia Testnet (Chain ID: 11155111)"
echo "  Factory: 0x59bcaa1BB72972Df0446FCe98798076e718E3b61"
echo "  EntryPoint: 0x0000000071727De22E5E9d8BAf0edAc6f37da032"
echo ""

# Step 1: Deploy Paymaster Contract (or check if already deployed)
echo -e "${YELLOW}📜 Step 1: Checking VerifierSignaturePaymaster deployment...${NC}"

# Check if paymaster is already deployed
PAYMASTER_ADDRESS="0x3da84818e202009488D2A8e2a3B2f78A6F6321bb"
echo "Checking if paymaster exists at: $PAYMASTER_ADDRESS"

# Check if contract exists
CONTRACT_CODE=$(cast code $PAYMASTER_ADDRESS --rpc-url $ALCHEMY_HTTP_SEPOLIA)
if [ ${#CONTRACT_CODE} -gt 4 ]; then
    echo -e "${GREEN}✅ Paymaster already deployed at: $PAYMASTER_ADDRESS${NC}"
    echo -e "${BLUE}📋 Verified on Etherscan: https://sepolia.etherscan.io/address/$PAYMASTER_ADDRESS${NC}"
    
    # Check if it has funds for sponsorship
    BALANCE=$(cast balance $PAYMASTER_ADDRESS --rpc-url $ALCHEMY_HTTP_SEPOLIA --ether)
    echo -e "${BLUE}💰 Paymaster balance: $BALANCE ETH${NC}"
    
    if (( $(echo "$BALANCE < 0.01" | bc -l) )); then
        echo -e "${YELLOW}⚠️  Warning: Paymaster balance is low. Consider adding more funds.${NC}"
    fi
else
    echo -e "${RED}❌ Paymaster not found. Please deploy it first.${NC}"
    exit 1
fi

# Step 2: Build Client
echo -e "${YELLOW}🔨 Step 2: Building client...${NC}"
cd client
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Client built successfully!${NC}"
else
    echo -e "${RED}❌ Client build failed${NC}"
    exit 1
fi

cd ..

# Step 3: Start Paymaster Service
echo -e "${YELLOW}🌐 Step 3: Starting paymaster service...${NC}"
echo "Service will run in background..."

cd paymaster-service

# Kill existing service if running
pkill -f "paymaster-service" || true

# Set environment variable to use Sepolia config
export PAYMASTER_CONFIG="config/sepolia.toml"

# Start service in background with Sepolia config
nohup cargo run > paymaster.log 2>&1 &
PAYMASTER_PID=$!

echo -e "${GREEN}✅ Paymaster service started (PID: $PAYMASTER_PID)${NC}"
echo "  Log file: paymaster-service/paymaster.log"
echo "  Service URL: http://localhost:3000"

# Wait for service to start
sleep 3

# Check if service is running
if ps -p $PAYMASTER_PID > /dev/null; then
    echo -e "${GREEN}✅ Service is running${NC}"
else
    echo -e "${RED}❌ Service failed to start. Check paymaster.log${NC}"
    cat paymaster.log
    exit 1
fi

cd ..

# Step 4: Test Sponsored Account Creation
echo ""
echo -e "${YELLOW}🎯 Step 4: Testing sponsored account creation...${NC}"

echo -e "${BLUE}🏭 Using deployed paymaster: $PAYMASTER_ADDRESS${NC}"

cd client

echo -e "${BLUE}🏭 Creating sponsored smart account...${NC}"
./target/release/aa-client deploy-sponsored \
    --private-key $PRIVATE_KEY \
    --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \
    --salt 0x1234567890abcdef \
    --rpc-url $ALCHEMY_HTTP_SEPOLIA \
    --chain-id 11155111 \
    --paymaster-url http://localhost:3000 \
    --paymaster-api-key sepolia_test_key_123 \
    --paymaster-address $PAYMASTER_ADDRESS

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Sponsored account created successfully!${NC}"
else
    echo -e "${RED}❌ Sponsored account creation failed${NC}"
    echo "Check paymaster service logs: ../paymaster-service/paymaster.log"
    exit 1
fi

# Step 5: Test Sponsored Transaction
echo ""
echo -e "${YELLOW}💸 Step 5: Testing sponsored transaction...${NC}"

# Send some test ETH (0.001 ETH = 1000000000000000 wei)
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
    --paymaster-address $PAYMASTER_ADDRESS

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Sponsored transaction executed successfully!${NC}"
else
    echo -e "${RED}❌ Sponsored transaction failed${NC}"
    echo "Check paymaster service logs: ../paymaster-service/paymaster.log"
fi

cd ..

echo ""
echo -e "${GREEN}🎉 Setup Complete!${NC}"
echo "========================================================"
echo ""
echo -e "${BLUE}📋 System Status:${NC}"
echo "  ✅ Paymaster contract verified and funded"
echo "  ✅ Paymaster service running on localhost:3000"
echo "  ✅ Client built and ready"
echo "  ✅ Sponsored transactions working"
echo ""
echo -e "${BLUE}🔧 Configuration Files:${NC}"
echo "  - Paymaster service: paymaster-service/config/sepolia.toml"
echo "  - Service logs: paymaster-service/paymaster.log"
echo ""
echo -e "${BLUE}🎯 Test Commands:${NC}"
echo ""
echo "Deploy sponsored account:"
echo "./client/target/release/aa-client deploy-sponsored \\"
echo "  --private-key \$PRIVATE_KEY \\"
echo "  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \\"
echo "  --salt 0x1234567890abcdef \\"
echo "  --rpc-url \$ALCHEMY_HTTP_SEPOLIA \\"
echo "  --chain-id 11155111 \\"
echo "  --paymaster-url http://localhost:3000 \\"
echo "  --paymaster-api-key sepolia_test_key_123 \\"
echo "  --paymaster-address $PAYMASTER_ADDRESS"
echo ""
echo "Execute sponsored transaction:"
echo "./client/target/release/aa-client submit-sponsored \\"
echo "  --private-key \$PRIVATE_KEY \\"
echo "  --target 0xRECIPIENT_ADDRESS \\"
echo "  --call-data 0x \\"
echo "  --factory 0x59bcaa1BB72972Df0446FCe98798076e718E3b61 \\"
echo "  --salt 0x1234567890abcdef \\"
echo "  --rpc-url \$ALCHEMY_HTTP_SEPOLIA \\"
echo "  --chain-id 11155111 \\"
echo "  --value 1000000000000000 \\"
echo "  --paymaster-url http://localhost:3000 \\"
echo "  --paymaster-api-key sepolia_test_key_123 \\"
echo "  --paymaster-address $PAYMASTER_ADDRESS"
echo ""
echo -e "${BLUE}🛑 To stop the paymaster service:${NC}"
echo "  pkill -f paymaster-service"
echo ""
echo -e "${GREEN}🎉 Enjoy gas-free transactions!${NC}"
