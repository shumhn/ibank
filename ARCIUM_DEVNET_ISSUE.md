# Arcium Devnet Deployment Guide

## Issue: Program Already Deployed

The error "account data too small for instruction" occurs because:
- Our Solana program is **already deployed** on devnet (Program ID: `Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22`)
- `arcium deploy` tries to deploy both MXE program AND our program
- Our program account already exists, causing the conflict

## Solution: Two Options

### Option 1: Use Existing Program (Recommended)

Since our program is already deployed, we can:

1. **Skip program deployment** in Arcium config
2. **Use existing program ID** for MPC circuits
3. **Initialize only the MPC infrastructure**

### Option 2: Redeploy with New Program ID

1. Generate new program keypair
2. Deploy fresh program with Arcium
3. Update all references

## Current Status

✅ **Solana Program:** Deployed on devnet  
❌ **Arcium MPC:** Not deployed (needs cluster + MXE)  
❌ **Computation Definitions:** Not initialized  

## Recommended Approach

Since our program is working perfectly on devnet, let's:

1. **Configure Arcium to use existing program**
2. **Deploy only the MPC infrastructure**
3. **Initialize computation definitions**
4. **Test MPC on devnet**

## Configuration Changes Needed

### 1. Update Arcium.toml
```toml
[network]
solana_rpc_url = "https://api.devnet.solana.com"
program_id = "Hcmhr2Leu8S6XgsjCjXX4yqgHFYP4X7Rvc23kUmmDJ22"

[deployment]
deploy_program = false  # Don't redeploy our program
```

### 2. Deploy Only MPC Infrastructure
```bash
# Deploy MXE and cluster only
arcium deploy --cluster-offset 1078779259 \
  --keypair-path ~/.config/solana/id.json \
  --rpc-url https://api.devnet.solana.com \
  --skip-program-deploy
```

### 3. Initialize Computation Definitions
After getting cluster pubkey from deployment:
```bash
# Update test scripts with cluster pubkey
# Initialize comp defs for each circuit
```

### 4. Test MPC Functionality
```bash
# Run tests against devnet with MPC
anchor test --skip-local-validator --provider.cluster devnet
```

## Alternative: Local Arcium + Devnet Solana

For immediate testing:

```bash
# Start local Arcium nodes
arcup localnet

# In another terminal, configure Solana for devnet
solana config set --url devnet

# Run tests (local Arcium + devnet Solana)
anchor test --skip-local-validator
```

This hybrid approach gives us MPC functionality with devnet blockchain persistence.
