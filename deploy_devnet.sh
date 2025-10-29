#!/bin/bash
# Deploy ibank to Solana Devnet

echo "🚀 Deploying ibank to Solana Devnet..."
echo ""

# Check balance
BALANCE=$(solana balance --url devnet 2>/dev/null | awk '{print $1}')
echo "💰 Current balance: $BALANCE SOL"

if (( $(echo "$BALANCE < 1" | bc -l) )); then
    echo "⚠️  Low balance! Requesting airdrop..."
    solana airdrop 2 --url devnet
    sleep 5
fi

echo ""
echo "📦 Program size:"
ls -lh target/deploy/ibank.so

echo ""
echo "🔑 Using keypair: /Users/sumangiri/.config/solana/id.json"

# Try to extend the program account first if it exists
echo ""
echo "📏 Attempting to extend program buffer (if needed)..."
solana program extend DQxanaqqWcTYvVhrKbeoY6q52NrGksWBL6vSbuVipnS7 0 --url devnet 2>/dev/null || echo "No extension needed or program doesn't exist yet"

echo ""
echo "🚀 Deploying program..."
anchor deploy --provider.cluster devnet --program-name ibank --program-keypair target/deploy/ibank-keypair.json

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Deployment successful!"
    echo ""
    echo "Program ID: DQxanaqqWcTYvVhrKbeoY6q52NrGksWBL6vSbuVipnS7"
    echo "View on Solana Explorer:"
    echo "https://explorer.solana.com/address/DQxanaqqWcTYvVhrKbeoY6q52NrGksWBL6vSbuVipnS7?cluster=devnet"
    echo ""
else
    echo ""
    echo "❌ Deployment failed!"
    echo ""
    echo "Possible solutions:"
    echo "1. Generate new program keypair: solana-keygen new -o target/deploy/ibank-keypair.json"
    echo "2. Request more SOL: solana airdrop 2 --url devnet"
    echo "3. Deploy with new program ID"
fi
