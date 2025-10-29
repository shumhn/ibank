use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;

// Computation definition offsets for banking operations
const COMP_DEF_OFFSET_INITIALIZE_ACCOUNTS: u32 = comp_def_offset("initialize_accounts");
const COMP_DEF_OFFSET_PROCESS_PAYMENT: u32 = comp_def_offset("process_payment");
const COMP_DEF_OFFSET_CHECK_BALANCE: u32 = comp_def_offset("check_balance");
const COMP_DEF_OFFSET_CALCULATE_REWARDS: u32 = comp_def_offset("calculate_rewards");

declare_id!("DQxanaqqWcTYvVhrKbeoY6q52NrGksWBL6vSbuVipnS7");

#[arcium_program]
pub mod blackjack {
    use super::*;

    /// Initializes the computation definition for account initialization.
    /// This sets up the MPC environment for creating encrypted user accounts with initial balances.
    pub fn init_initialize_accounts_comp_def(
        ctx: Context<InitInitializeAccountsCompDef>,
    ) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    /// Creates new user accounts with encrypted initial balances.
    ///
    /// This function sets up user accounts for the privacy-first banking system and triggers
    /// the MPC computation to initialize encrypted balances. All balance information remains
    /// private through Arcium's encrypted computation.
    ///
    /// # Arguments
    /// * `account_id` - Unique identifier for this account
    /// * `initial_balance` - Starting balance (will be encrypted)
    /// * `mxe_nonce` - Cryptographic nonce for MXE operations  
    /// * `client_pubkey` - User's encryption public key
    /// * `client_nonce` - User's cryptographic nonce
    pub fn initialize_user_account(
        ctx: Context<InitializeUserAccount>,
        computation_offset: u64,
        account_id: u64,
        initial_balance: u64,
        mxe_nonce: u128,
        client_pubkey: [u8; 32],
        client_nonce: u128,
    ) -> Result<()> {
        // Initialize the user account
        let user_account = &mut ctx.accounts.user_account;
        user_account.bump = ctx.bumps.user_account;
        user_account.account_id = account_id;
        user_account.owner_pubkey = ctx.accounts.payer.key();
        user_account.encrypted_balance = [0; 32];
        user_account.balance_nonce = 0;
        user_account.transaction_count = 0;
        user_account.reward_points = 0;
        user_account.owner_enc_pubkey = client_pubkey;
        user_account.account_state = AccountState::Initializing;

        // Queue the account initialization computation
        let args = vec![
            Argument::PlaintextU64(initial_balance),
            Argument::PlaintextU128(mxe_nonce),
            Argument::ArcisPubkey(client_pubkey),
            Argument::PlaintextU128(client_nonce),
            Argument::ArcisPubkey(client_pubkey),
            Argument::PlaintextU128(client_nonce),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![InitializeAccountsCallback::callback_ix(&[
                CallbackAccount {
                    pubkey: ctx.accounts.user_account.key(),
                    is_writable: true,
                },
            ])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "initialize_accounts")]
    pub fn initialize_accounts_callback(
        ctx: Context<InitializeAccountsCallback>,
        output: ComputationOutputs<InitializeAccountsOutput>,
    ) -> Result<()> {
        let (mxe_balance, client_balance) = match output {
            ComputationOutputs::Success(InitializeAccountsOutput {
                field_0: InitializeAccountsOutputStruct0 {
                    field_0: mxe_bal,
                    field_1: client_bal,
                },
            }) => (mxe_bal, client_bal),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let balance_nonce = client_balance.nonce;
        let balance_ciphertext: [u8; 32] = client_balance.ciphertexts[0];

        let user_account = &mut ctx.accounts.user_account;
        user_account.encrypted_balance = balance_ciphertext;
        user_account.balance_nonce = balance_nonce;
        user_account.account_state = AccountState::Active;

        emit!(AccountInitializedEvent {
            account_id: user_account.account_id,
            owner: user_account.owner_pubkey,
            balance_nonce,
        });
        Ok(())
    }

    pub fn init_process_payment_comp_def(
        ctx: Context<InitProcessPaymentCompDef>,
    ) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn process_payment(
        ctx: Context<ProcessPayment>,
        computation_offset: u64,
        transaction_id: u64,
        amount: u64,
        receiver_new_nonce: u128,
    ) -> Result<()> {
        require!(
            ctx.accounts.sender_account.account_state == AccountState::Active,
            ErrorCode::InvalidAccountState
        );
        require!(
            ctx.accounts.receiver_account.account_state == AccountState::Active,
            ErrorCode::InvalidAccountState
        );

        let transaction = &mut ctx.accounts.transaction;
        transaction.bump = ctx.bumps.transaction;
        transaction.transaction_id = transaction_id;
        transaction.sender = ctx.accounts.sender_account.key();
        transaction.receiver = ctx.accounts.receiver_account.key();
        transaction.encrypted_amount = [0; 32];
        transaction.amount_nonce = 0;
        transaction.timestamp = Clock::get()?.unix_timestamp;
        transaction.status = TransactionStatus::Processing;

        let args = vec![
            Argument::PlaintextU128(ctx.accounts.sender_account.balance_nonce),
            Argument::Account(ctx.accounts.sender_account.key(), 8 + 8 + 32, 32),
            Argument::PlaintextU128(ctx.accounts.receiver_account.balance_nonce),
            Argument::Account(ctx.accounts.receiver_account.key(), 8 + 8 + 32, 32),
            Argument::PlaintextU64(amount),
            Argument::ArcisPubkey(ctx.accounts.receiver_account.owner_enc_pubkey),
            Argument::PlaintextU128(receiver_new_nonce),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![ProcessPaymentCallback::callback_ix(&[
                CallbackAccount {
                    pubkey: ctx.accounts.transaction.key(),
                    is_writable: true,
                },
            ])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "process_payment")]
    pub fn process_payment_callback(
        ctx: Context<ProcessPaymentCallback>,
        output: ComputationOutputs<ProcessPaymentOutput>,
    ) -> Result<()> {
        let (_new_sender_balance, _new_receiver_balance, is_sufficient) = match output {
            ComputationOutputs::Success(ProcessPaymentOutput {
                field_0: ProcessPaymentOutputStruct0 {
                    field_0: sender_bal,
                    field_1: receiver_bal,
                    field_2: sufficient,
                },
            }) => (sender_bal, receiver_bal, sufficient),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        if !is_sufficient {
            ctx.accounts.transaction.status = TransactionStatus::Failed;
            emit!(PaymentFailedEvent {
                transaction_id: ctx.accounts.transaction.transaction_id,
                reason: "Insufficient balance".to_string(),
            });
            return Err(ErrorCode::InsufficientBalance.into());
        }

        ctx.accounts.transaction.status = TransactionStatus::Completed;

        emit!(PaymentProcessedEvent {
            transaction_id: ctx.accounts.transaction.transaction_id,
            sender: ctx.accounts.transaction.sender,
            receiver: ctx.accounts.transaction.receiver,
            timestamp: ctx.accounts.transaction.timestamp,
        });
        Ok(())
    }

    pub fn init_check_balance_comp_def(
        ctx: Context<InitCheckBalanceCompDef>,
    ) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn check_balance(
        ctx: Context<CheckBalance>,
        computation_offset: u64,
        _account_id: u64,
        threshold: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.user_account.account_state == AccountState::Active,
            ErrorCode::InvalidAccountState
        );

        let args = vec![
            Argument::PlaintextU128(ctx.accounts.user_account.balance_nonce),
            Argument::Account(ctx.accounts.user_account.key(), 8 + 8 + 32, 32),
            Argument::PlaintextU64(threshold),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![CheckBalanceCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.user_account.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "check_balance")]
    pub fn check_balance_callback(
        ctx: Context<CheckBalanceCallback>,
        output: ComputationOutputs<CheckBalanceOutput>,
    ) -> Result<()> {
        let is_above_threshold = match output {
            ComputationOutputs::Success(CheckBalanceOutput { field_0: result }) => result,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        emit!(BalanceCheckEvent {
            account_id: ctx.accounts.user_account.account_id,
            is_above_threshold,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn init_calculate_rewards_comp_def(
        ctx: Context<InitCalculateRewardsCompDef>,
    ) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn calculate_rewards(
        ctx: Context<CalculateRewards>,
        computation_offset: u64,
        _account_id: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.user_account.account_state == AccountState::Active,
            ErrorCode::InvalidAccountState
        );

        let args = vec![
            Argument::PlaintextU64(ctx.accounts.user_account.transaction_count),
            Argument::PlaintextU128(ctx.accounts.user_account.balance_nonce),
            Argument::Account(ctx.accounts.user_account.key(), 8 + 8 + 32, 32),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![CalculateRewardsCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.user_account.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "calculate_rewards")]
    pub fn calculate_rewards_callback(
        ctx: Context<CalculateRewardsCallback>,
        output: ComputationOutputs<CalculateRewardsOutput>,
    ) -> Result<()> {
        let reward_points = match output {
            ComputationOutputs::Success(CalculateRewardsOutput { field_0: points }) => points,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.user_account.reward_points += reward_points;

        emit!(RewardsCalculatedEvent {
            account_id: ctx.accounts.user_account.account_id,
            reward_points,
            total_rewards: ctx.accounts.user_account.reward_points,
        });
        Ok(())
    }
}

// ============================================================================
// ACCOUNT CONTEXTS - Initialize Accounts
// ============================================================================

#[queue_computation_accounts("initialize_accounts", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, account_id: u64)]
pub struct InitializeUserAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INITIALIZE_ACCOUNTS)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS,
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        init,
        payer = payer,
        space = 8 + UserAccount::INIT_SPACE,
        seeds = [b"user_account", account_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

#[callback_accounts("initialize_accounts")]
#[derive(Accounts)]
pub struct InitializeAccountsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INITIALIZE_ACCOUNTS)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[init_computation_definition_accounts("initialize_accounts", payer)]
#[derive(Accounts)]
pub struct InitInitializeAccountsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// ACCOUNT CONTEXTS - Process Payment
// ============================================================================

#[queue_computation_accounts("process_payment", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, transaction_id: u64)]
pub struct ProcessPayment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub sender_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub receiver_account: Account<'info, UserAccount>,
    #[account(
        init,
        payer = payer,
        space = 8 + Transaction::INIT_SPACE,
        seeds = [b"transaction", transaction_id.to_le_bytes().as_ref()],
        bump
    )]
    pub transaction: Account<'info, Transaction>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_PAYMENT)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS,
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("process_payment")]
#[derive(Accounts)]
pub struct ProcessPaymentCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_PAYMENT)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub transaction: Account<'info, Transaction>,
}

#[init_computation_definition_accounts("process_payment", payer)]
#[derive(Accounts)]
pub struct InitProcessPaymentCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut)]
    /// CHECK: Checked by Arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// ACCOUNT CONTEXTS - Check Balance
// ============================================================================

#[queue_computation_accounts("check_balance", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _account_id: u64)]
pub struct CheckBalance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK_BALANCE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS,
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[callback_accounts("check_balance")]
#[derive(Accounts)]
pub struct CheckBalanceCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK_BALANCE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[init_computation_definition_accounts("check_balance", payer)]
#[derive(Accounts)]
pub struct InitCheckBalanceCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut)]
    /// CHECK: Checked by Arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// ACCOUNT CONTEXTS - Calculate Rewards
// ============================================================================

#[queue_computation_accounts("calculate_rewards", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _account_id: u64)]
pub struct CalculateRewards<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_CALCULATE_REWARDS)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS,
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[callback_accounts("calculate_rewards")]
#[derive(Accounts)]
pub struct CalculateRewardsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_CALCULATE_REWARDS)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[init_computation_definition_accounts("calculate_rewards", payer)]
#[derive(Accounts)]
pub struct InitCalculateRewardsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut)]
    /// CHECK: Checked by Arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================
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
