// Data structures for privacy-first banking
use anchor_lang::prelude::*;

/// User account storing encrypted balance and transaction history.
#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    /// Unique account identifier
    pub account_id: u64,
    /// Owner's Solana public key
    pub owner_pubkey: Pubkey,
    /// Encrypted balance (32 bytes ciphertext)
    pub encrypted_balance: [u8; 32],
    /// Nonce for balance encryption
    pub balance_nonce: u128,
    /// Total number of transactions
    pub transaction_count: u64,
    /// Accumulated reward points
    pub reward_points: u64,
    /// Owner's Arcium encryption public key
    pub owner_enc_pubkey: [u8; 32],
    /// Current account state
    pub account_state: AccountState,
    /// PDA bump seed
    pub bump: u8,
}

/// Transaction record with encrypted amount.
#[account]
#[derive(InitSpace)]
pub struct Transaction {
    /// Unique transaction identifier
    pub transaction_id: u64,
    /// Sender account public key
    pub sender: Pubkey,
    /// Receiver account public key
    pub receiver: Pubkey,
    /// Encrypted transaction amount
    pub encrypted_amount: [u8; 32],
    /// Nonce for amount encryption
    pub amount_nonce: u128,
    /// Transaction timestamp
    pub timestamp: i64,
    /// Transaction status
    pub status: TransactionStatus,
    /// PDA bump seed
    pub bump: u8,
}

#[repr(u8)]
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccountState {
    Initializing = 0,
    Active = 1,
    Frozen = 2,
    Closed = 3,
}

#[repr(u8)]
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionStatus {
    Processing = 0,
    Completed = 1,
    Failed = 2,
}

// Events
#[event]
pub struct AccountInitializedEvent {
    pub account_id: u64,
    pub owner: Pubkey,
    pub balance_nonce: u128,
}

#[event]
pub struct PaymentProcessedEvent {
    pub transaction_id: u64,
    pub sender: Pubkey,
    pub receiver: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PaymentFailedEvent {
    pub transaction_id: u64,
    pub reason: String,
}

#[event]
pub struct RewardsCalculatedEvent {
    pub account_id: u64,
    pub reward_points: u64,
    pub total_rewards: u64,
}

#[event]
pub struct BalanceCheckEvent {
    pub account_id: u64,
    pub is_above_threshold: bool,
    pub timestamp: i64,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Invalid account state")]
    InvalidAccountState,
    #[msg("Insufficient balance for transaction")]
    InsufficientBalance,
    #[msg("Invalid encryption pubkey")]
    InvalidEncryptionPubkey,
    #[msg("Cluster not set")]
    ClusterNotSet,
}
