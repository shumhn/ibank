use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    /// Encrypted balance structure for privacy-preserving banking
    /// Stores a single u64 balance value encrypted
    pub struct Balance {
        pub amount: u64,
    }

    /// Initialize user account with encrypted balance
    /// Encrypts an initial balance for a new user account.
    /// Returns the encrypted balance that only the user can decrypt.
    #[instruction]
    pub fn initialize_accounts(
        initial_balance: u64,
        mxe: Mxe,
        client: Shared,
        client_again: Shared,
    ) -> (Enc<Mxe, u64>, Enc<Shared, u64>) {
        // Encrypt the initial balance for MXE (server-side tracking)
        let mxe_balance = mxe.from_arcis(initial_balance);
        // Encrypt the initial balance for client (user can decrypt)
        let client_balance = client_again.from_arcis(initial_balance);
        
        (mxe_balance, client_balance)
    }

    /// Process payment from sender to receiver
    /// Similar to player_hit - updates state and returns new encrypted values
    #[instruction]
    pub fn process_payment(
        sender_balance_ctxt: Enc<Mxe, u64>,
        receiver_balance_ctxt: Enc<Mxe, u64>,
        amount: u64,
        receiver_key: Shared,
    ) -> (Enc<Mxe, u64>, Enc<Shared, u64>, bool) {
        // Decrypt balances within MPC
        let sender_balance = sender_balance_ctxt.to_arcis();
        let receiver_balance = receiver_balance_ctxt.to_arcis();

        // Check if sender has sufficient balance
        let is_sufficient = sender_balance >= amount;

        // Calculate new balances
        let new_sender_balance = if is_sufficient {
            sender_balance - amount
        } else {
            sender_balance // No change if insufficient
        };

        let new_receiver_balance = if is_sufficient {
            receiver_balance + amount
        } else {
            receiver_balance // No change if insufficient
        };

        // Re-encrypt balances
        let sender_encrypted = sender_balance_ctxt.owner.from_arcis(new_sender_balance);
        let receiver_encrypted = receiver_key.from_arcis(new_receiver_balance);

        (sender_encrypted, receiver_encrypted, is_sufficient.reveal())
    }

    /// Check if balance meets threshold for compliance
    /// Similar to player_stand - checks state and returns boolean
    #[instruction]
    pub fn check_balance(
        balance_ctxt: Enc<Mxe, u64>,
        threshold: u64,
    ) -> bool {
        let balance = balance_ctxt.to_arcis();
        (balance >= threshold).reveal()
    }

    /// Calculate rewards based on transaction activity
    /// Calculate reward points based on transaction count and balance
    #[instruction]
    pub fn calculate_rewards(
        transaction_count: u64,
        balance_ctxt: Enc<Mxe, u64>,
    ) -> u64 {
        let balance = balance_ctxt.to_arcis();
        
        // Reward calculation logic:
        // - Base: 10 points per transaction
        // - Bonus: Additional points based on balance tier
        let base_rewards = transaction_count * 10;
        
        let balance_bonus = if balance >= 10000 {
            100 // Premium tier
        } else if balance >= 5000 {
            50 // Gold tier
        } else if balance >= 1000 {
            25 // Silver tier
        } else {
            0 // Basic tier
        };

        let total_rewards = base_rewards + balance_bonus;
        
        total_rewards.reveal()
    }
}
