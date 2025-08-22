#!/bin/bash

# Anvil Integration Test Script
echo "🚀 Starting Anvil Integration Test"
echo "=================================="

# Kill any existing anvil processes
pkill -f anvil || true

# Start Anvil in background
echo "📡 Starting Anvil..."
anvil --host 0.0.0.0 --port 8545 --accounts 10 --balance 1000 > anvil.log 2>&1 &
ANVIL_PID=$!

# Wait for Anvil to start
echo "⏳ Waiting for Anvil to start..."
sleep 3

# Check if Anvil is running
if ! curl -s http://localhost:8545 > /dev/null; then
    echo "❌ Anvil failed to start"
    exit 1
fi

echo "✅ Anvil started successfully"

# Run the integration test
echo "🧪 Running integration test..."
cd contracts

forge script AnvilIntegrationTest \
    --rpc-url http://localhost:8545 \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

TEST_EXIT_CODE=$?

# Cleanup
echo "🧹 Cleaning up..."
kill $ANVIL_PID 2>/dev/null || true

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "✅ Integration test completed successfully!"
else
    echo "❌ Integration test failed"
    echo "📋 Check anvil.log for details"
fi

exit $TEST_EXIT_CODE
