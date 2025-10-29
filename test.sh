#!/bin/bash
# Banking Demo Test Runner

echo "🏦 Privacy-First Banking Demo - Test Runner"
echo "==========================================="
echo ""

# Check if arcium localnet is running
if ! pgrep -f "arcium" > /dev/null; then
    echo "❌ ERROR: Arcium localnet is not running!"
    echo ""
    echo "Please start it in another terminal first:"
    echo "  cd /Users/sumangiri/Desktop/blackjack"
    echo "  arcup localnet"
    echo ""
    exit 1
fi

echo "✅ Arcium localnet is running"
echo ""
echo "🧪 Running banking demo tests..."
echo ""

# Run anchor tests
anchor test --skip-local-validator

echo ""
echo "✅ Tests complete!"
