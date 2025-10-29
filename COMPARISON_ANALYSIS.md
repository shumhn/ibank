# Deep Analysis: Reference Blackjack vs Our Banking Implementation

## THE CORE ISSUE

Our banking implementation is failing with `InvalidArguments` error because the MPC circuit signature doesn't match what we're passing from Rust.

---

## REFERENCE BLACKJACK - HOW IT WORKS

### Reference MPC Circuit (`player_hit`):
```rust
pub fn player_hit(
    deck_ctxt: Enc<Mxe, Deck>,           // 1. Encrypted deck (nonce + account data)
    player_hand_ctxt: Enc<Shared, Hand>, // 2. Encrypted player hand (pubkey + nonce + account data)
    player_hand_size: u8,                // 3. Plaintext size
    dealer_hand_size: u8,                // 4. Plaintext size
) -> (Enc<Shared, Hand>, bool)
```

### Reference Rust Arguments (7 total):
```rust
let args = vec![
    // Deck (Enc<Mxe, Deck>)
    Argument::PlaintextU128(deck_nonce),              // 1
    Argument::Account(game_key, 8, 32 * 3),          // 2
    
    // Player hand (Enc<Shared, Hand>)
    Argument::ArcisPubkey(player_enc_pubkey),        // 3
    Argument::PlaintextU128(client_nonce),           // 4
    Argument::Account(game_key, 8 + 32*3, 32),       // 5
    
    // Plaintext values
    Argument::PlaintextU8(player_hand_size),         // 6
    Argument::PlaintextU8(dealer_hand_size),         // 7
];
```

### How Arguments Map to MPC Parameters:

**Enc<Mxe, Deck>** = PlaintextU128 (nonce) + Account (data)
- Args 1 + 2 → Parameter 1

**Enc<Shared, Hand>** = ArcisPubkey (key) + PlaintextU128 (nonce) + Account (data)
- Args 3 + 4 + 5 → Parameter 2

**u8** = PlaintextU8
- Arg 6 → Parameter 3
- Arg 7 → Parameter 4

---

## OUR BANKING IMPLEMENTATION

### Our MPC Circuit (`process_payment`):
```rust
pub fn process_payment(
    sender_balance_nonce: u128,              // 1. Plaintext nonce
    sender_balance_ctxt: Enc<Mxe, u64>,      // 2. Encrypted balance
    receiver_balance_nonce: u128,            // 3. Plaintext nonce
    receiver_balance_ctxt: Enc<Mxe, u64>,    // 4. Encrypted balance
    amount: u64,                             // 5. Plaintext amount
    receiver_key: Shared,                    // 6. Receiver's pubkey
    receiver_new_nonce: u128,                // 7. New nonce for output
) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool)
```

### Our Rust Arguments (7 total):
```rust
let args = vec![
    Argument::PlaintextU128(sender_balance_nonce),      // 1
    Argument::Account(sender_key, 48, 32),              // 2
    Argument::PlaintextU128(receiver_balance_nonce),    // 3
    Argument::Account(receiver_key, 48, 32),            // 4
    Argument::PlaintextU64(amount),                     // 5
    Argument::ArcisPubkey(receiver_enc_pubkey),         // 6
    Argument::PlaintextU128(receiver_new_nonce),        // 7
];
```

---

## THE PROBLEM - ARGUMENT MAPPING MISMATCH

### What We're Passing:
1. PlaintextU128 (sender nonce)
2. Account (sender data)
3. PlaintextU128 (receiver nonce)
4. Account (receiver data)
5. PlaintextU64 (amount)
6. ArcisPubkey (receiver key)
7. PlaintextU128 (receiver new nonce)

### What MPC Circuit Expects:
1. **u128** (sender nonce) ✅
2. **Enc<Mxe, u64>** (sender encrypted balance) - expects nonce + account ❌
3. **u128** (receiver nonce) ✅
4. **Enc<Mxe, u64>** (receiver encrypted balance) - expects nonce + account ❌
5. **u64** (amount) ✅
6. **Shared** (receiver key) ✅
7. **u128** (receiver new nonce) ✅

---

## THE ROOT CAUSE

**In the reference:**
- `Enc<Mxe, Deck>` is created from: **nonce + account data** (2 args)
- `Enc<Shared, Hand>` is created from: **pubkey + nonce + account data** (3 args)

**In our implementation:**
- We're passing **nonce SEPARATELY** from the encrypted data
- But `Enc<Mxe, u64>` should be constructed from **nonce + account together**

**The MPC circuit signature is WRONG!**

Our circuit says:
```rust
sender_balance_nonce: u128,              // Separate nonce
sender_balance_ctxt: Enc<Mxe, u64>,      // Encrypted value
```

But it should be:
```rust
sender_balance_ctxt: Enc<Mxe, u64>,      // This INCLUDES the nonce!
```

---

## THE FIX

### Option 1: Fix MPC Circuit Signature (RECOMMENDED)
Change the MPC circuit to NOT take nonces as separate parameters:

```rust
pub fn process_payment(
    sender_balance_ctxt: Enc<Mxe, u64>,      // Includes nonce
    receiver_balance_ctxt: Enc<Mxe, u64>,    // Includes nonce
    amount: u64,
    receiver_key: Shared,
    receiver_nonce: u128,                    // Only for NEW encryption
) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool)
```

Then Rust args become:
```rust
let args = vec![
    // sender_balance_ctxt (Enc<Mxe, u64>)
    Argument::PlaintextU128(sender_nonce),
    Argument::Account(sender_key, 48, 32),
    
    // receiver_balance_ctxt (Enc<Mxe, u64>)
    Argument::PlaintextU128(receiver_nonce),
    Argument::Account(receiver_key, 48, 32),
    
    // amount (u64)
    Argument::PlaintextU64(amount),
    
    // receiver_key (Shared)
    Argument::ArcisPubkey(receiver_pubkey),
    
    // receiver_nonce (u128) - for output
    Argument::PlaintextU128(receiver_new_nonce),
];
```

This gives us 7 Rust arguments mapping to 5 MPC parameters:
- Args 1+2 → Enc<Mxe, u64> (sender)
- Args 3+4 → Enc<Mxe, u64> (receiver)
- Arg 5 → u64 (amount)
- Arg 6 → Shared (receiver key)
- Arg 7 → u128 (new nonce)

---

## NEXT STEPS

1. Update MPC circuit signature in `encrypted-ixs/src/lib.rs`
2. Rebuild circuits: `arcium build`
3. Test should pass!
