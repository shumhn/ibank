# Official Arcium Devnet Deployment Guide

Based on official Arcium documentation: https://docs.arcium.com/developers/deployment

## Prerequisites ✅

- [x] MXE built successfully (`arcium build`) ✅
- [x] Tests passing locally (`arcium test`) ✅
- [x] Solana program deployed to devnet ✅
- [ ] 2-5 SOL in devnet wallet
- [ ] Reliable RPC endpoint (Helius or QuickNode recommended)

## Current Status

✅ **Solana Program:** Deployed on devnet  
Program ID: `Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22`

✅ **MPC Circuits:** Compiled locally (`.arcis` files in `build/`)

⏳ **Arcium Infrastructure:** Not deployed yet

## Step 1: Get Required SOL

Check balance:
```bash
solana balance --url devnet
```

Request airdrop if needed:
```bash
solana airdrop 2 --url devnet
# Repeat 2-3 times to get ~5 SOL
```

## Step 2: Choose Deployment Method

### Option A: Use Helius RPC (Recommended)

1. **Get Free Helius API Key:**
   - Visit: https://helius.dev
   - Sign up for free account
   - Get API key for devnet

2. **Deploy with Helius:**
```bash
arcium deploy --cluster-offset 1078779259 \
  --keypair-path ~/.config/solana/id.json \
  --rpc-url https://devnet.helius-rpc.com/?api-key=YOUR_API_KEY
```

### Option B: Use Default Devnet RPC (Simpler)

```bash
arcium deploy --cluster-offset 1078779259 \
  --keypair-path ~/.config/solana/id.json \
  -u d  # 'd' for devnet
```

### Option C: Use QuickNode RPC

1. Get QuickNode endpoint: https://quicknode.com
2. Deploy:
```bash
arcium deploy --cluster-offset 1078779259 \
  --keypair-path ~/.config/solana/id.json \
  --rpc-url YOUR_QUICKNODE_DEVNET_URL
```

## Step 3: Handle Program Deployment Conflict

**Issue:** `arcium deploy` tries to deploy both:
1. Arcium infrastructure (MXE, cluster, mempool)
2. Your Solana program

**Problem:** Our program is already deployed!

**Solutions:**

### Solution 1: Let It Fail Gracefully
- Run `arcium deploy` as shown above
- It will fail when trying to redeploy the program
- BUT it should still deploy the Arcium infrastructure successfully
- Check output for cluster pubkey

### Solution 2: Use Partial Deployment (If Available)
```bash
# Deploy only Arcium infrastructure, skip program
arcium deploy --cluster-offset 1078779259 \
  --keypair-path ~/.config/solana/id.json \
  -u d \
  --skip-program  # If this flag exists
```

### Solution 3: Modify Arcium.toml
Before deployment, try adding to `Arcium.toml`:
```toml
[deployment]
skip_program_deploy = true
```

## Step 4: After Deployment

### A. Save Cluster Information

From deployment output, save:
- **Cluster Pubkey** (e.g., `GgSqqAyH7AVY3Umcv8NvncrjFaNJuQLmxzxFxPoPW2Yd`)
- **Cluster Offset** (e.g., `1078779259`)
- **MXE Pubkey**

### B. Initialize Computation Definitions

Update your test initialization to use the cluster:

```typescript
// In tests/banking_demo.ts or separate init script

import { getClusterAccAddress } from '@arcium-hq/client';

// Use your deployment's cluster offset
const CLUSTER_OFFSET = 1078779259;
const clusterAccount = getClusterAccAddress(CLUSTER_OFFSET);

// When initializing computation definitions:
await program.methods
  .initInitializeAccountsCompDef()
  .accounts({
    clusterAccount: clusterAccount,  // Use deployed cluster
    // ... other accounts
  })
  .rpc();

// Repeat for all 4 computation definitions:
// - initialize_accounts
// - process_payment
// - check_balance
// - calculate_rewards
```

## Step 5: Update Test Configuration

Create devnet test configuration:

```typescript
// tests/banking_demo_devnet.ts

const USE_DEVNET = true;
const CLUSTER_OFFSET = 1078779259;

let program: Program<Ibank>;
let provider: AnchorProvider;
let clusterAccount: PublicKey;

if (USE_DEVNET) {
  // Devnet configuration
  const connection = new Connection(
    'https://api.devnet.solana.com',
    'confirmed'
  );
  const wallet = new Wallet(owner);
  provider = new AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
  });
  
  // Load program
  const programId = new PublicKey('Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22');
  program = new Program(IDL, programId, provider);
  
  // Use deployed cluster
  clusterAccount = getClusterAccAddress(CLUSTER_OFFSET);
} else {
  // Local configuration
  anchor.setProvider(anchor.AnchorProvider.env());
  provider = anchor.getProvider() as AnchorProvider;
  program = anchor.workspace.ibank as Program<Ibank>;
  
  const arciumEnv = getArciumEnv();
  clusterAccount = arciumEnv.arciumClusterPubkey;
}
```

## Step 6: Run Tests on Devnet

```bash
# First initialize computation definitions
ts-node scripts/init_comp_defs_devnet.ts

# Then run tests
anchor test --skip-local-validator --provider.cluster devnet
```

## Step 7: Verify Everything Works

Check deployed program:
```bash
solana program show Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22 --url devnet
```

Check cluster account:
```bash
# Get cluster pubkey from deployment
solana account <CLUSTER_PUBKEY> --url devnet
```

## Common Issues & Solutions

### Issue 1: "account data too small for instruction"
**Cause:** Trying to redeploy existing program  
**Solution:** This is expected! The Arcium infrastructure should still deploy.

### Issue 2: Transactions dropping
**Cause:** Default devnet RPC rate limits  
**Solution:** Use Helius or QuickNode RPC

### Issue 3: Running out of SOL
**Cause:** Deployment costs ~2-5 SOL  
**Solution:** Request more airdrops: `solana airdrop 2 --url devnet`

### Issue 4: Cluster offset collision
**Cause:** Offset already used  
**Solution:** Try different offset: 3726127828 or 768109697

## Recommended Cluster Offsets (From Docs)

- `1078779259` - Primary choice
- `3726127828` - Alternative 1
- `768109697` - Alternative 2

## What Gets Deployed

When you run `arcium deploy`:

1. **MXE Program** - Core Arcium program (if not exists)
2. **Cluster Account** - Your MPC cluster
3. **Mempool Account** - Transaction queue
4. **Execution Pool** - Computation executor
5. **MXE Account** - Multi-party execution context

## Cost Breakdown

- Program deployment: ~1-2 SOL (skipped if exists)
- Cluster initialization: ~0.5 SOL
- Account rent: ~0.1 SOL
- Computation definitions: ~0.01 SOL each
- **Total:** ~2-5 SOL

## Next Steps After Successful Deployment

1. ✅ Save cluster offset and pubkey
2. ✅ Update test configuration
3. ✅ Initialize all computation definitions
4. ✅ Run end-to-end tests on devnet
5. ✅ Monitor transactions on Solana Explorer
6. ✅ Share devnet endpoint with team

## Success Criteria

✅ Cluster deployed and accessible  
✅ All 4 computation definitions initialized  
✅ Tests pass on devnet  
✅ MPC computations execute successfully  
✅ Callbacks trigger correctly  
✅ Events emitted as expected  

---

**Ready to deploy?** Run the deployment command and follow the steps above!
