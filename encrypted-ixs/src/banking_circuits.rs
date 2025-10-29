use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    /// Encrypts an initial balance for a new user account.
    /// Returns the encrypted balance that only the user can decrypt.
    #[instruction]
    pub fn initialize_accounts(
        initial_balance: u64,
        mxe: Mxe,
        client: Shared,
    ) -> Enc<Shared, u64> {
        // Encrypt the initial balance for the client (user)
        client.from_arcis(initial_balance)
    }

    /// Processes a payment from sender to receiver.
    /// 
    /// Takes encrypted sender/receiver balances and plaintext amount.
    /// Returns new encrypted balances and whether the transaction succeeded.
    /// 
    /// # Arguments
    /// * `sender_balance_ctxt` - Sender's encrypted balance
    /// * `receiver_balance_ctxt` - Receiver's encrypted balance
    /// * `amount` - Payment amount (plaintext, will be encrypted in result)
    /// * `receiver_key` - Receiver's encryption public key
    /// 
    /// # Returns
    /// * New sender balance (encrypted)
    /// * New receiver balance (encrypted for receiver)
    /// * Whether sender had sufficient balance
    #[instruction]
    pub fn process_payment(
        sender_balance_nonce: u128,
        sender_balance_ctxt: Enc<Mxe, u64>,
        receiver_balance_nonce: u128,
        receiver_balance_ctxt: Enc<Mxe, u64>,
        amount: u64,
        receiver_key: Shared,
        receiver_new_nonce: u128,
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

    /// Calculates reward points based on transaction activity.
    /// 
    /// Rewards are calculated as: transaction_count * 10
    /// This demonstrates anonymous reward distribution without revealing balances.
    /// 
    /// # Arguments
    /// * `transaction_count` - Number of transactions performed
    /// * `current_balance_ctxt` - Current encrypted balance (for future reward logic)
    /// 
    /// # Returns
    /// * Reward points earned (plaintext)
    #[instruction]
    pub fn calculate_rewards(
        transaction_count: u64,
        balance_nonce: u128,
        current_balance_ctxt: Enc<Mxe, u64>,
    ) -> u64 {
        // Simple reward calculation: 10 points per transaction
        let reward_points = transaction_count * 10;

        // Could add balance-based bonuses here without revealing the actual balance
        // For now, just return transaction-based rewards
        reward_points.reveal()
    }

    /// Checks if balance is above a threshold for compliance/auditing.
    /// 
    /// Returns a boolean without revealing the actual balance amount.
    /// This enables regulatory compliance while maintaining privacy.
    /// 
    /// # Arguments
    /// * `balance_ctxt` - Encrypted balance
    /// * `threshold` - Minimum balance threshold
    /// 
    /// # Returns
    /// * Whether balance >= threshold (without revealing actual balance)
    #[instruction]
    pub fn check_balance(
        balance_nonce: u128,
        balance_ctxt: Enc<Mxe, u64>,
        threshold: u64,
    ) -> bool {
        let balance = balance_ctxt.to_arcis();
        let is_above_threshold = balance >= threshold;
        is_above_threshold.reveal()
    }
}
