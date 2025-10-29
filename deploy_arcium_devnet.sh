#!/bin/bash
# Deploy Arcium MPC circuits to devnet

echo "🚀 Deploying Arcium MPC circuits to devnet..."

# Check SOL balance
BALANCE=$(solana balance --url devnet 2>/dev/null | awk '{print $1}')
echo "💰 Devnet SOL balance: $BALANCE SOL"

if (( $(echo "$BALANCE < 2" | bc -l) )); then
    echo "⚠️  Low balance! Requesting airdrop..."
    solana airdrop 2 --url devnet
    sleep 5
fi

# Choose cluster offset (using one from the suggested list)
CLUSTER_OFFSET="1078779259"
echo "📊 Using cluster offset: $CLUSTER_OFFSET"

# Deploy Arcium infrastructure
echo ""
echo "🏗️  Deploying Arcium cluster..."
arcium deploy --cluster-offset $CLUSTER_OFFSET \
  --keypair-path ~/.config/solana/id.json \
  --rpc-url https://api.devnet.solana.com

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Arcium deployment successful!"
    echo ""
    echo "Next steps:"
    echo "1. Note the cluster pubkey from deployment output"
    echo "2. Update Arcium.toml with the cluster pubkey"
    echo "3. Initialize computation definitions"
    echo "4. Run tests with devnet MPC"
else
    echo ""
    echo "❌ Arcium deployment failed!"
    echo ""
    echo "Try different RPC or cluster offset"
fi
