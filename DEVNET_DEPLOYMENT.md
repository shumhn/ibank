# Devnet Deployment Guide

## Prerequisites
✅ Solana CLI configured for devnet
✅ Wallet with SOL (need ~2 SOL for deployment)
✅ Arcium CLI installed

## Current Status
- Wallet: /Users/sumangiri/.config/solana/devnet-keypair.json
- Balance: 1.98 SOL
- RPC: https://api.devnet.solana.com

## Deployment Steps

### 1. Build the Program
```bash
cd /Users/sumangiri/Desktop/blackjack
anchor build
```

### 2. Deploy to Solana Devnet
```bash
anchor deploy --provider.cluster devnet
```

This will:
- Upload your program to devnet
- Generate a new program ID (or use existing)
- Cost ~0.5-1 SOL

### 3. Update Program ID
After deployment, update the program ID in:
- `programs/blackjack/src/lib.rs` (declare_id!)
- `Anchor.toml` ([programs.localnet])

### 4. Rebuild with New Program ID
```bash
anchor build
```

### 5. Deploy MPC Circuits to Arcium Devnet
```bash
# Set Arcium to devnet
arcium config set-network devnet

# Upload circuits
arcium upload encrypted-ixs/target/wasm32-unknown-unknown/release/encrypted_ixs.wasm

# Or use the arcium deploy command if available
```

### 6. Initialize Computation Definitions on Devnet
Run the init script against devnet:
```bash
# Update test configuration to use devnet
# Then run initialization
ts-node scripts/init_comp_defs_devnet.ts
```

### 7. Update Test Configuration
In `tests/banking_demo.ts`, change:
```typescript
// FROM: localnet
const provider = AnchorProvider.local();

// TO: devnet
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
const provider = new AnchorProvider(connection, wallet, {
  commitment: 'confirmed'
});
```

### 8. Run Tests Against Devnet
```bash
anchor test --skip-local-validator --provider.cluster devnet
```

## Important Notes

⚠️ **Arcium Devnet**: 
- Arcium may or may not have a public devnet
- You might need to run Arcium nodes locally and point to devnet Solana
- Alternative: Deploy Solana program to devnet but keep MPC local

⚠️ **Program Upgrades**:
- After first deploy, you can upgrade with: `anchor upgrade`
- Keep the program keypair safe!

⚠️ **Gas Costs**:
- Each computation costs SOL
- Budget 0.1-0.5 SOL per test run
- Airdrop more if needed: `solana airdrop 2`

## Verification

Check your deployed program:
```bash
solana program show <PROGRAM_ID> --url devnet
```

Check account state:
```bash
solana account <ACCOUNT_ADDRESS> --url devnet
```

## Next Steps After Deployment

1. Share program ID with frontend developers
2. Update frontend to use devnet RPC
3. Test with real wallets (Phantom/Solflare on devnet)
4. Monitor transactions on Solana Explorer (devnet)
