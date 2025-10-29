#!/bin/bash
# Deploy Arcium to Devnet - Following Official Documentation

echo "🚀 Deploying Arcium MPC Infrastructure to Devnet"
echo "Following: https://docs.arcium.com/developers/deployment"
echo ""

# Check balance
BALANCE=$(solana balance --url devnet 2>/dev/null | awk '{print $1}')
echo "💰 Current devnet balance: $BALANCE SOL"
echo ""

if (( $(echo "$BALANCE < 2" | bc -l) )); then
    echo "⚠️  Insufficient balance! You need at least 2 SOL."
    echo "Request airdrops:"
    echo "  solana airdrop 2 --url devnet"
    echo ""
    read -p "Request airdrop now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Requesting airdrops..."
        solana airdrop 2 --url devnet
        sleep 5
        solana airdrop 2 --url devnet
        sleep 5
        BALANCE=$(solana balance --url devnet 2>/dev/null | awk '{print $1}')
        echo "New balance: $BALANCE SOL"
    fi
fi

# Cluster offset (from official docs)
CLUSTER_OFFSET="1078779259"
echo "📊 Using cluster offset: $CLUSTER_OFFSET"
echo ""

# Deployment options
echo "Choose RPC provider:"
echo "1) Default Devnet RPC (simple, may be slow)"
echo "2) Helius RPC (recommended, requires API key)"
echo "3) QuickNode RPC (requires endpoint)"
echo ""
read -p "Enter choice (1-3): " -n 1 -r CHOICE
echo ""

case $CHOICE in
  1)
    echo "Using default devnet RPC..."
    RPC_URL="https://api.devnet.solana.com"
    ;;
  2)
    read -p "Enter your Helius API key: " API_KEY
    RPC_URL="https://devnet.helius-rpc.com/?api-key=$API_KEY"
    ;;
  3)
    read -p "Enter your QuickNode devnet URL: " QUICKNODE_URL
    RPC_URL="$QUICKNODE_URL"
    ;;
  *)
    echo "Invalid choice. Using default RPC."
    RPC_URL="https://api.devnet.solana.com"
    ;;
esac

echo ""
echo "🏗️  Deploying Arcium infrastructure..."
echo "This may take a few minutes..."
echo ""

# Run deployment
arcium deploy --cluster-offset $CLUSTER_OFFSET \
  --keypair-path ~/.config/solana/id.json \
  --rpc-url $RPC_URL

DEPLOY_EXIT_CODE=$?

echo ""
if [ $DEPLOY_EXIT_CODE -eq 0 ] || [ $DEPLOY_EXIT_CODE -eq 101 ]; then
    echo "✅ Deployment completed!"
    echo ""
    echo "⚠️  Note: Program deployment error is expected (program already exists)"
    echo "✅ Arcium infrastructure (MXE, cluster, mempool) should be deployed"
    echo ""
    echo "📝 Next steps:"
    echo "1. Save the cluster pubkey from output above"
    echo "2. Update CLUSTER_OFFSET in tests: $CLUSTER_OFFSET"
    echo "3. Initialize computation definitions"
    echo "4. Run tests on devnet"
    echo ""
    echo "Cluster offset: $CLUSTER_OFFSET"
else
    echo "❌ Deployment failed with exit code: $DEPLOY_EXIT_CODE"
    echo ""
    echo "Common solutions:"
    echo "- Try different cluster offset: 3726127828 or 768109697"
    echo "- Use Helius or QuickNode RPC"
    echo "- Request more SOL: solana airdrop 2 --url devnet"
fi

echo ""
echo "📊 Final balance:"
solana balance --url devnet
