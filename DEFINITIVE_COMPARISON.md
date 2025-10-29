# Definitive Comparison - Reference vs Ours

## Reference: player_hit

### MPC Circuit (4 params):
```rust
pub fn player_hit(
    deck_ctxt: Enc<Mxe, Deck>,           // 1: INPUT encrypted deck
    player_hand_ctxt: Enc<Shared, Hand>, // 2: INPUT encrypted hand
    player_hand_size: u8,                // 3: plaintext size
    dealer_hand_size: u8,                // 4: plaintext size
) -> (Enc<Shared, Hand>, bool)           // OUTPUT: encrypted hand + bool
```

### Rust Args (7 args):
```rust
vec![
    Argument::PlaintextU128(deck_nonce),              // 1
    Argument::Account(game_key, 8, 32*3),             // 2
    Argument::ArcisPubkey(player_enc_pubkey),         // 3
    Argument::PlaintextU128(client_nonce),            // 4
    Argument::Account(game_key, 8+32*3, 32),          // 5
    Argument::PlaintextU8(player_hand_size),          // 6
    Argument::PlaintextU8(dealer_hand_size),          // 7
]
```

### Mapping:
- Args 1+2 → Param 1: `Enc<Mxe, Deck>`
- Args 3+4+5 → Param 2: `Enc<Shared, Hand>`
- Arg 6 → Param 3: `u8`
- Arg 7 → Param 4: `u8`

**Key Insight:** `Enc<Shared, T>` INPUT requires 3 args: pubkey + nonce + account

---

## Ours: process_payment

### MPC Circuit (4 params):
```rust
pub fn process_payment(
    sender_balance_ctxt: Enc<Mxe, u64>,      // 1: INPUT encrypted balance
    receiver_balance_ctxt: Enc<Mxe, u64>,    // 2: INPUT encrypted balance
    amount: u64,                              // 3: plaintext amount
    receiver_key: Shared,                     // 4: for OUTPUT encryption
) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool) // OUTPUT: 2 encrypted + bool
```

### Rust Args (7 args):
```rust
vec![
    Argument::PlaintextU128(sender_nonce),            // 1
    Argument::Account(sender_key, 48, 32),            // 2
    Argument::PlaintextU128(receiver_nonce),          // 3
    Argument::Account(receiver_key, 48, 32),          // 4
    Argument::PlaintextU64(amount),                   // 5
    Argument::ArcisPubkey(receiver_pubkey),           // 6
    Argument::PlaintextU128(receiver_new_nonce),      // 7
]
```

### Mapping:
- Args 1+2 → Param 1: `Enc<Mxe, u64>` ✅
- Args 3+4 → Param 2: `Enc<Mxe, u64>` ✅
- Arg 5 → Param 3: `u64` ✅
- Args 6+7 → Param 4: `Shared` ✅

**This looks correct!**

---

## THE PROBLEM

Wait... in the reference, `Enc<Shared, Hand>` is an INPUT (reading encrypted data from account).
In ours, `Shared` is just for OUTPUT encryption (creating new encrypted data).

These are DIFFERENT use cases!

### Reference Pattern for INPUT:
`Enc<Shared, T>` = pubkey + nonce + account (3 args)

### Reference Pattern for OUTPUT:
Looking at `shuffle_and_deal_cards` which creates outputs:
```rust
pub fn shuffle_and_deal_cards(
    mxe: Mxe,           // For creating Enc<Mxe, Deck> output
    mxe_again: Mxe,     // For creating Enc<Mxe, Hand> output
    client: Shared,     // For creating Enc<Shared, Hand> output
    client_again: Shared, // For creating Enc<Shared, u8> output
)
```

Rust args:
```rust
vec![
    Argument::PlaintextU128(mxe_nonce),              // → Mxe
    Argument::PlaintextU128(mxe_again_nonce),        // → Mxe
    Argument::ArcisPubkey(client_pubkey),            // → Shared (part 1)
    Argument::PlaintextU128(client_nonce),           // → Shared (part 2)
    Argument::ArcisPubkey(client_pubkey),            // → Shared (part 1)
    Argument::PlaintextU128(client_again_nonce),     // → Shared (part 2)
]
```

**Pattern for OUTPUT Shared:** pubkey + nonce (2 args)

So our mapping should be correct!

---

## HYPOTHESIS

Maybe the issue is that we're using `Enc<Mxe, u64>` as INPUTS but the framework expects us to read them differently?

Or maybe the offset calculation is wrong? Let me check:
- Account ID: 8 bytes
- Owner pubkey: 32 bytes
- Encrypted balance: 32 bytes
- Offset to encrypted_balance: 8 + 8 + 32 = 48 bytes ✅

Wait... Account ID is u64 (8 bytes), but what about the account discriminator?

Anchor accounts have an 8-byte discriminator at the start!

So the offset should be:
- Discriminator: 8 bytes
- Account ID: 8 bytes
- Owner pubkey: 32 bytes
- Encrypted balance: 32 bytes
- **Offset to encrypted_balance: 8 + 8 + 32 = 48 bytes** ✅

That's correct...

---

## FINAL THEORY

Maybe we need to check if the MPC circuit is using the contexts correctly?

Or maybe there's a mismatch in how we're constructing the output?

Let me check the MPC circuit implementation...
