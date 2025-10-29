# THE FIX - Correct Argument Mapping

## Discovery from Reference

Reference `player_hit` MPC circuit:
```rust
pub fn player_hit(
    deck_ctxt: Enc<Mxe, Deck>,           // Param 1
    player_hand_ctxt: Enc<Shared, Hand>, // Param 2
    player_hand_size: u8,                // Param 3
    dealer_hand_size: u8,                // Param 4
)
```

Reference Rust args (7 total):
```rust
vec![
    Argument::PlaintextU128(deck_nonce),         // 1 \
    Argument::Account(key, 8, 32*3),             // 2 / → Enc<Mxe, Deck>
    
    Argument::ArcisPubkey(player_pubkey),        // 3 \
    Argument::PlaintextU128(client_nonce),       // 4 |→ Enc<Shared, Hand>
    Argument::Account(key, 8+32*3, 32),          // 5 /
    
    Argument::PlaintextU8(player_hand_size),     // 6 → u8
    Argument::PlaintextU8(dealer_hand_size),     // 7 → u8
]
```

## The Pattern

**Enc<Shared, T>** needs **3 arguments**, not 2!
- ArcisPubkey (pubkey)
- PlaintextU128 (nonce)
- Account (encrypted data)

## Our process_payment Fix

Current (WRONG):
```rust
vec![
    Argument::PlaintextU128(sender_nonce),
    Argument::Account(sender_key, 48, 32),
    Argument::PlaintextU128(receiver_nonce),
    Argument::Account(receiver_key, 48, 32),
    Argument::PlaintextU64(amount),
    Argument::ArcisPubkey(receiver_pubkey),
    Argument::PlaintextU128(receiver_new_nonce),
]
```

This maps to:
- Enc<Mxe, u64> (sender) ✅
- Enc<Mxe, u64> (receiver) ✅
- u64 (amount) ✅
- Shared (receiver_key) ❌ - Missing Account!

Fixed (CORRECT):
```rust
vec![
    Argument::PlaintextU128(sender_nonce),
    Argument::Account(sender_key, 48, 32),
    Argument::PlaintextU128(receiver_nonce),
    Argument::Account(receiver_key, 48, 32),
    Argument::PlaintextU64(amount),
    Argument::ArcisPubkey(receiver_pubkey),
    Argument::PlaintextU128(receiver_new_nonce),
    Argument::Account(receiver_key, 48, 32),  // ← NEED THIS!
]
```

Wait, that doesn't make sense... we'd be reading the receiver account twice?

## Re-thinking...

Maybe `Shared` for OUTPUT encryption doesn't need the Account data, just pubkey + nonce?

Let me check the MPC circuit return type:
```rust
) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool)
```

The `Enc<Shared, u64>` is an OUTPUT, not an input!

So we only need pubkey + nonce to CREATE it, not to READ it!

So our current args should be correct... but they're not working.

## Alternative Theory

Maybe the issue is the MPC circuit itself? Let me check if we're using the Shared context correctly in the circuit...
