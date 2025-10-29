# Systematic Fix - Exact Argument Mapping

## Reference: shuffle_and_deal_cards

### MPC Circuit (4 params):
```rust
pub fn shuffle_and_deal_cards(
    mxe: Mxe,           // 1
    mxe_again: Mxe,     // 2
    client: Shared,     // 3
    client_again: Shared, // 4
)
```

### Rust Args (6 args):
```rust
vec![
    Argument::PlaintextU128(mxe_nonce),              // 1 → mxe
    Argument::PlaintextU128(mxe_again_nonce),        // 2 → mxe_again
    Argument::ArcisPubkey(client_pubkey),            // 3 → client (part 1)
    Argument::PlaintextU128(client_nonce),           // 4 → client (part 2)
    Argument::ArcisPubkey(client_pubkey),            // 5 → client_again (part 1)
    Argument::PlaintextU128(client_again_nonce),     // 6 → client_again (part 2)
]
```

### Mapping Rules:
- `Mxe` parameter ← `PlaintextU128` (nonce)
- `Shared` parameter ← `ArcisPubkey` + `PlaintextU128` (pubkey + nonce)

---

## Our: process_payment

### Current MPC Circuit (4 params):
```rust
pub fn process_payment(
    sender_balance_ctxt: Enc<Mxe, u64>,      // 1
    receiver_balance_ctxt: Enc<Mxe, u64>,    // 2
    amount: u64,                              // 3
    receiver_key: Shared,                     // 4
)
```

### Current Rust Args (7 args):
```rust
vec![
    Argument::PlaintextU128(sender_nonce),           // 1 → Enc<Mxe, u64> (part 1)
    Argument::Account(sender_key, offset, size),     // 2 → Enc<Mxe, u64> (part 2)
    Argument::PlaintextU128(receiver_nonce),         // 3 → Enc<Mxe, u64> (part 1)
    Argument::Account(receiver_key, offset, size),   // 4 → Enc<Mxe, u64> (part 2)
    Argument::PlaintextU64(amount),                  // 5 → u64
    Argument::ArcisPubkey(receiver_pubkey),          // 6 → Shared (part 1)
    Argument::PlaintextU128(receiver_new_nonce),     // 7 → Shared (part 2)
]
```

### Mapping Rules:
- `Enc<Mxe, T>` parameter ← `PlaintextU128` (nonce) + `Account` (data)
- `u64` parameter ← `PlaintextU64`
- `Shared` parameter ← `ArcisPubkey` + `PlaintextU128`

This should work! 7 Rust args → 4 MPC params.

---

## The Problem

We're getting `InvalidArguments` which means the Arcium framework can't map our 7 Rust arguments to the 4 MPC parameters.

## Hypothesis

Maybe the issue is that we're creating an OUTPUT `Enc<Shared, u64>` but the MPC circuit doesn't know which nonce to use for it?

Looking at the reference output:
```rust
) -> (
    Enc<Mxe, Deck>,    // Uses mxe
    Enc<Mxe, Hand>,    // Uses mxe_again
    Enc<Shared, Hand>, // Uses client
    Enc<Shared, u8>,   // Uses client_again
)
```

They have SEPARATE contexts for each output!

Our output:
```rust
) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool)
```

We're creating `Enc<Shared, u64>` but we only have ONE `Shared` parameter (`receiver_key`). The framework doesn't know which nonce to use!

## THE FIX

We need a SECOND `Shared` parameter for the output encryption!

### Fixed MPC Circuit (5 params):
```rust
pub fn process_payment(
    sender_balance_ctxt: Enc<Mxe, u64>,      // 1
    receiver_balance_ctxt: Enc<Mxe, u64>,    // 2
    amount: u64,                              // 3
    receiver_key: Shared,                     // 4 (for creating Enc<Shared, u64> output)
    receiver_key_again: Shared,               // 5 (NEED THIS - maybe for something else?)
)
```

Wait, that doesn't make sense either...

## ALTERNATIVE: Check Reference player_hit Output

```rust
pub fn player_hit(
    deck_ctxt: Enc<Mxe, Deck>,
    player_hand_ctxt: Enc<Shared, Hand>,  // INPUT is Enc<Shared, Hand>
    player_hand_size: u8,
    dealer_hand_size: u8,
) -> (Enc<Shared, Hand>, bool)  // OUTPUT is also Enc<Shared, Hand>
```

The INPUT `player_hand_ctxt` is `Enc<Shared, Hand>` and it's used to create the OUTPUT `Enc<Shared, Hand>`!

So the `Shared` context from the INPUT is reused for the OUTPUT!

## OUR ISSUE

Our MPC circuit:
```rust
pub fn process_payment(
    sender_balance_ctxt: Enc<Mxe, u64>,      // INPUT
    receiver_balance_ctxt: Enc<Mxe, u64>,    // INPUT
    amount: u64,
    receiver_key: Shared,                     // For OUTPUT encryption
) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool)
```

We're creating `Enc<Shared, u64>` output but we don't have a `Shared` INPUT to reuse!

The `receiver_key: Shared` parameter is just a pubkey, not an encrypted context!

## THE REAL FIX

We need to use `receiver_balance_ctxt.owner` or create the Shared context properly!

Looking at our MPC circuit implementation:
```rust
let receiver_encrypted = receiver_key.from_arcis(new_receiver_balance);
```

This should work! `receiver_key` is a `Shared` context that can encrypt values.

So the MPC circuit is correct. The issue must be in the Rust argument mapping!

## FINAL HYPOTHESIS

Maybe we need to pass the receiver's pubkey TWICE - once for reading the input and once for creating the output?

Or maybe the issue is that we're not passing enough arguments to construct the `Shared` parameter properly?

Let me check if we need to pass the pubkey BEFORE the nonce for Shared contexts...
