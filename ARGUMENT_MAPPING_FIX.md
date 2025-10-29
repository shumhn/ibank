# Argument Mapping Fix - Initialize Accounts

## THE PROBLEM

Our `initialize_accounts` MPC circuit expects:
```rust
pub fn initialize_accounts(
    initial_balance: u64,
    mxe: Mxe,           // Encryption context (NOT a nonce!)
    client: Shared,     // Encryption context (NOT a nonce!)
) -> (Enc<Mxe, u64>, Enc<Shared, u64>)
```

But we're passing from Rust:
```rust
let args = vec![
    Argument::PlaintextU64(initial_balance),     // ✅ Maps to initial_balance
    Argument::PlaintextU128(mxe_nonce),          // ❌ Wrong! This is for Enc<Mxe, ...> OUTPUTS
    Argument::ArcisPubkey(client_pubkey),        // ✅ Maps to client
    Argument::PlaintextU128(client_nonce),       // ❌ Wrong! This is for Enc<Shared, ...> OUTPUTS
];
```

## THE SOLUTION

The `Mxe` and `Shared` parameters in the MPC circuit are **encryption contexts**, not nonces!

**For INPUT encryption contexts:**
- `Mxe` parameter → Pass NOTHING (it's automatically available in MPC)
- `Shared` parameter → Pass `Argument::ArcisPubkey(pubkey)` ONLY

**For OUTPUT encryption:**
- To create `Enc<Mxe, u64>` output → MPC uses `mxe.from_arcis(value)` automatically
- To create `Enc<Shared, u64>` output → Need `Shared` context + nonce

## CORRECT RUST ARGUMENTS

```rust
let args = vec![
    Argument::PlaintextU64(initial_balance),     // Maps to: initial_balance
    Argument::PlaintextU128(mxe_nonce),          // Maps to: nonce for Enc<Mxe, u64> OUTPUT
    Argument::ArcisPubkey(client_pubkey),        // Maps to: client (Shared context)
    Argument::PlaintextU128(client_nonce),       // Maps to: nonce for Enc<Shared, u64> OUTPUT
];
```

Wait... this is what we already have! So the issue must be in the MPC circuit signature itself.

## ACTUAL FIX NEEDED

The MPC circuit signature should match what we're passing:

```rust
pub fn initialize_accounts(
    initial_balance: u64,
    mxe_nonce: u128,        // For creating Enc<Mxe, u64> output
    client: Shared,         // Client encryption context
    client_nonce: u128,     // For creating Enc<Shared, u64> output
) -> (Enc<Mxe, u64>, Enc<Shared, u64>)
```

Then inside the function:
```rust
let mxe_balance = Mxe::from_nonce(mxe_nonce).from_arcis(initial_balance);
let client_balance = client.from_arcis(initial_balance);
(mxe_balance, client_balance)
```

But wait - we don't have access to `Mxe::from_nonce()`. The `Mxe` type is special and provided by the framework.

## THE REAL ISSUE

Looking at the reference blackjack, they DON'T pass nonces for creating NEW encrypted values in initialization!

The framework automatically handles nonce generation for outputs. We should NOT be passing nonces at all for initialization!

## FINAL FIX

Change MPC circuit to:
```rust
pub fn initialize_accounts(
    initial_balance: u64,
    client: Shared,
) -> (Enc<Mxe, u64>, Enc<Shared, u64>) {
    let mxe_balance = Mxe::default().from_arcis(initial_balance);
    let client_balance = client.from_arcis(initial_balance);
    (mxe_balance, client_balance)
}
```

Change Rust args to:
```rust
let args = vec![
    Argument::PlaintextU64(initial_balance),
    Argument::ArcisPubkey(client_pubkey),
];
```

The framework will automatically generate nonces for the outputs!
