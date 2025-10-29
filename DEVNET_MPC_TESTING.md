# Testing MPC on Solana Devnet

## Overview

To test MPC functionality on devnet, we need:
1. ✅ Solana program deployed on devnet (Done!)
2. ⏳ MPC circuits compiled (Done locally, need to upload)
3. ⏳ Arcium devnet infrastructure
4. ⏳ Computation definitions initialized on devnet

## Current Status

**Solana Program:**
- ✅ Deployed to devnet
- Program ID: `Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22`

**MPC Circuits:**
- ✅ Compiled locally (`.arcis` files in `build/` directory)
- ⏳ Need to upload to Arcium network

**Arcium Infrastructure:**
- ⏳ Need to verify if Arcium has a public devnet
- Alternative: Run local Arcium nodes pointing to Solana devnet

## Option 1: Arcium Public Devnet (If Available)

### Step 1: Check Arcium Network Availability
```bash
# Check available networks
arcium config list-networks

# Set network to devnet
arcium config set-network devnet
```

### Step 2: Upload MPC Circuits
```bash
# Upload compiled circuits to Arcium devnet
arcium upload build/initialize_accounts.arcis
arcium upload build/process_payment.arcis
arcium upload build/check_balance.arcis
arcium upload build/calculate_rewards.arcis
```

### Step 3: Initialize Computation Definitions
```bash
# Run initialization script against devnet
# (Would need to modify test script to use devnet)
ts-node scripts/init_comp_defs_devnet.ts
```

### Step 4: Update Test Configuration
```typescript
// In tests/banking_demo.ts
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
const provider = new AnchorProvider(connection, wallet, {
  commitment: 'confirmed'
});
```

### Step 5: Run Tests
```bash
anchor test --skip-local-validator --provider.cluster devnet
```

## Option 2: Local Arcium Nodes + Devnet Solana (Hybrid)

This is more realistic if Arcium doesn't have a public devnet yet.

### Architecture:
```
Frontend/Tests
    ↓
Solana Devnet (RPC: api.devnet.solana.com)
    ↓
Your Deployed Program (Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22)
    ↓
Local Arcium MPC Nodes (Docker)
```

### Setup:

1. **Start Local Arcium Nodes**
```bash
# In one terminal
cd /Users/sumangiri/Desktop/ibank
arcup localnet
```

2. **Configure Solana to Use Devnet**
```bash
solana config set --url devnet
```

3. **Update Arcium Config to Point to Devnet**
Edit `Arcium.toml`:
```toml
[network]
solana_rpc_url = "https://api.devnet.solana.com"
program_id = "Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22"
```

4. **Initialize Computation Definitions on Devnet**
```bash
# Run init script (will use devnet Solana but local Arcium nodes)
ts-node scripts/init_comp_defs.ts
```

5. **Run Tests Against Hybrid Setup**
```bash
# Tests will hit:
# - Solana devnet for blockchain operations
# - Local Arcium nodes for MPC computations
anchor test --skip-local-validator
```

## Option 3: Fully Local Testing (Recommended for Now)

Since devnet MPC might not be available:

```bash
# Use localnet for both Solana and Arcium
cd /Users/sumangiri/Desktop/ibank
arcium test

# This tests the exact same code, just on local infrastructure
# Once working locally, deploying to devnet is straightforward
```

## Checking Arcium Devnet Availability

To verify if Arcium has a public devnet:

```bash
# Check Arcium docs
open https://docs.arcium.com

# Check available endpoints
curl -s https://api.devnet.arcium.com/health 2>&1
# or
curl -s https://devnet.arcium.com/health 2>&1

# Check Arcium Discord/community for devnet info
```

## Important Notes

### MPC Circuit Persistence

**Problem:** Computation definitions are PDAs on Solana. Once deployed to devnet:
- They're permanent (unless closed)
- Tied to specific program IDs
- Cannot be easily "reset"

**Solution:** Use unique computation offsets for testing:
```typescript
// Generate unique offset for each test
const offset = Buffer.from(randomBytes(8));
```

### Cost Considerations

**Devnet Testing Costs:**
- Solana transactions: Free (devnet SOL from faucet)
- Arcium MPC computations: May have fees on devnet
- Storage rent: ~0.001-0.01 SOL per account

**Budget:** Keep ~5-10 devnet SOL for testing

## Next Steps

1. **Determine Arcium Devnet Availability**
   - Check Arcium documentation
   - Contact Arcium team if needed

2. **Choose Testing Strategy**
   - Fully local (easiest, recommended for now)
   - Hybrid (local Arcium + devnet Solana)
   - Fully devnet (if available)

3. **Update Configuration**
   - Modify test scripts for chosen strategy
   - Update Arcium.toml if needed

4. **Run End-to-End Tests**
   - Initialize accounts
   - Process payments
   - Check balances
   - Calculate rewards

## Success Criteria

✅ All 4 MPC circuits execute successfully  
✅ Encrypted data stored on devnet  
✅ Callbacks trigger and update state  
✅ Events emitted correctly  
✅ All tests pass  

---

**Current Recommendation:** Start with local testing (`arcium test`) to verify functionality, then explore devnet MPC options with Arcium team.
