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
pub mod privacy_banking {
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


        let args = vec![
            // Deck
            Argument::PlaintextU128(ctx.accounts.blackjack_game.deck_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8, 32 * 3),
            // Player hand
            Argument::ArcisPubkey(ctx.accounts.blackjack_game.player_enc_pubkey),
            Argument::PlaintextU128(ctx.accounts.blackjack_game.client_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8 + 32 * 3, 32),
            // Player hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.player_hand_size),
            // Dealer hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.dealer_hand_size),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![PlayerHitCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.blackjack_game.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "player_hit")]
    pub fn player_hit_callback(
        ctx: Context<PlayerHitCallback>,
        output: ComputationOutputs<PlayerHitOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(PlayerHitOutput {
                field_0:
                    PlayerHitOutputStruct0 {
                        field_0: player_hand,
                        field_1: is_bust,
                    },
            }) => (player_hand, is_bust),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let client_nonce = o.0.nonce;

        let player_hand: [u8; 32] = o.0.ciphertexts[0];

        let is_bust: bool = o.1;

        let blackjack_game = &mut ctx.accounts.blackjack_game;
        blackjack_game.player_hand = player_hand;
        blackjack_game.client_nonce = client_nonce;

        if is_bust {
            blackjack_game.game_state = GameState::DealerTurn;
            emit!(PlayerBustEvent {
                client_nonce,
                game_id: blackjack_game.game_id,
            });
        } else {
            blackjack_game.game_state = GameState::PlayerTurn;
            emit!(PlayerHitEvent {
                player_hand,
                client_nonce,
                game_id: blackjack_game.game_id,
            });
            blackjack_game.player_hand_size += 1;
        }

        Ok(())
    }

    pub fn init_player_double_down_comp_def(
        ctx: Context<InitPlayerDoubleDownCompDef>,
    ) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn player_double_down(
        ctx: Context<PlayerDoubleDown>,
        computation_offset: u64,
        _game_id: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.blackjack_game.game_state == GameState::PlayerTurn,
            ErrorCode::InvalidGameState
        );
        require!(
            !ctx.accounts.blackjack_game.player_has_stood,
            ErrorCode::InvalidMove
        );

        let args = vec![
            // Deck
            Argument::PlaintextU128(ctx.accounts.blackjack_game.deck_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8, 32 * 3),
            // Player hand
            Argument::ArcisPubkey(ctx.accounts.blackjack_game.player_enc_pubkey),
            Argument::PlaintextU128(ctx.accounts.blackjack_game.client_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8 + 32 * 3, 32),
            // Player hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.player_hand_size),
            // Dealer hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.dealer_hand_size),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![PlayerDoubleDownCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.blackjack_game.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "player_double_down")]
    pub fn player_double_down_callback(
        ctx: Context<PlayerDoubleDownCallback>,
        output: ComputationOutputs<PlayerDoubleDownOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(PlayerDoubleDownOutput {
                field_0:
                    PlayerDoubleDownOutputStruct0 {
                        field_0: player_hand,
                        field_1: is_bust,
                    },
            }) => (player_hand, is_bust),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let client_nonce = o.0.nonce;

        let player_hand: [u8; 32] = o.0.ciphertexts[0];

        let is_bust: bool = o.1;

        let blackjack_game = &mut ctx.accounts.blackjack_game;
        blackjack_game.player_hand = player_hand;
        blackjack_game.client_nonce = client_nonce;
        blackjack_game.player_has_stood = true;

        if is_bust {
            blackjack_game.game_state = GameState::DealerTurn;
            emit!(PlayerBustEvent {
                client_nonce,
                game_id: blackjack_game.game_id,
            });
        } else {
            blackjack_game.game_state = GameState::DealerTurn;
            emit!(PlayerDoubleDownEvent {
                player_hand,
                client_nonce,
                game_id: blackjack_game.game_id,
            });
        }

        Ok(())
    }

    pub fn init_player_stand_comp_def(ctx: Context<InitPlayerStandCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn player_stand(
        ctx: Context<PlayerStand>,
        computation_offset: u64,
        _game_id: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.blackjack_game.game_state == GameState::PlayerTurn,
            ErrorCode::InvalidGameState
        );
        require!(
            !ctx.accounts.blackjack_game.player_has_stood,
            ErrorCode::InvalidMove
        );

        let args = vec![
            // Player hand
            Argument::ArcisPubkey(ctx.accounts.blackjack_game.player_enc_pubkey),
            Argument::PlaintextU128(ctx.accounts.blackjack_game.client_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8 + 32 * 3, 32),
            // Player hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.player_hand_size),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![PlayerStandCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.blackjack_game.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "player_stand")]
    pub fn player_stand_callback(
        ctx: Context<PlayerStandCallback>,
        output: ComputationOutputs<PlayerStandOutput>,
    ) -> Result<()> {
        let is_bust = match output {
            ComputationOutputs::Success(PlayerStandOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let blackjack_game = &mut ctx.accounts.blackjack_game;
        blackjack_game.player_has_stood = true;

        if is_bust {
            // This should never happen
            blackjack_game.game_state = GameState::PlayerTurn;
            emit!(PlayerBustEvent {
                client_nonce: blackjack_game.client_nonce,
                game_id: blackjack_game.game_id,
            });
        } else {
            blackjack_game.game_state = GameState::DealerTurn;
            emit!(PlayerStandEvent {
                is_bust,
                game_id: blackjack_game.game_id
            });
        }

        Ok(())
    }

    pub fn init_dealer_play_comp_def(ctx: Context<InitDealerPlayCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn dealer_play(
        ctx: Context<DealerPlay>,
        computation_offset: u64,
        _game_id: u64,
        nonce: u128,
    ) -> Result<()> {
        require!(
            ctx.accounts.blackjack_game.game_state == GameState::DealerTurn,
            ErrorCode::InvalidGameState
        );

        let args = vec![
            // Deck
            Argument::PlaintextU128(ctx.accounts.blackjack_game.deck_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8, 32 * 3),
            // Dealer hand
            Argument::PlaintextU128(ctx.accounts.blackjack_game.dealer_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8 + 32 * 3 + 32, 32),
            // Client nonce
            Argument::ArcisPubkey(ctx.accounts.blackjack_game.player_enc_pubkey),
            Argument::PlaintextU128(nonce),
            // Player hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.player_hand_size),
            // Dealer hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.dealer_hand_size),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![DealerPlayCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.blackjack_game.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "dealer_play")]
    pub fn dealer_play_callback(
        ctx: Context<DealerPlayCallback>,
        output: ComputationOutputs<DealerPlayOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(DealerPlayOutput {
                field_0:
                    DealerPlayOutputStruct0 {
                        field_0: dealer_hand,
                        field_1: dealer_client_hand,
                        field_2: dealer_hand_size,
                    },
            }) => (dealer_hand, dealer_client_hand, dealer_hand_size),
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        let dealer_nonce = o.0.nonce;
        let dealer_hand = o.0.ciphertexts[0];
        let dealer_client_hand = o.1.ciphertexts[0];
        let dealer_hand_size = o.2;
        let client_nonce = o.1.nonce;

        let blackjack_game = &mut ctx.accounts.blackjack_game;
        blackjack_game.dealer_hand = dealer_hand;
        blackjack_game.dealer_nonce = dealer_nonce;
        blackjack_game.dealer_hand_size = dealer_hand_size;
        blackjack_game.game_state = GameState::Resolving;

        emit!(DealerPlayEvent {
            dealer_hand: dealer_client_hand,
            dealer_hand_size,
            client_nonce,
            game_id: ctx.accounts.blackjack_game.game_id,
        });

        Ok(())
    }

    pub fn init_resolve_game_comp_def(ctx: Context<InitResolveGameCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn resolve_game(
        ctx: Context<ResolveGame>,
        computation_offset: u64,
        _game_id: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.blackjack_game.game_state == GameState::Resolving,
            ErrorCode::InvalidGameState
        );

        let args = vec![
            // Player hand
            Argument::ArcisPubkey(ctx.accounts.blackjack_game.player_enc_pubkey),
            Argument::PlaintextU128(ctx.accounts.blackjack_game.client_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8 + 32 * 3, 32),
            // Dealer hand
            Argument::PlaintextU128(ctx.accounts.blackjack_game.dealer_nonce),
            Argument::Account(ctx.accounts.blackjack_game.key(), 8 + 32 * 3 + 32, 32),
            // Player hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.player_hand_size),
            // Dealer hand size
            Argument::PlaintextU8(ctx.accounts.blackjack_game.dealer_hand_size),
        ];

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![ResolveGameCallback::callback_ix(&[CallbackAccount {
                pubkey: ctx.accounts.blackjack_game.key(),
                is_writable: true,
            }])],
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "resolve_game")]
    pub fn resolve_game_callback(
        ctx: Context<ResolveGameCallback>,
        output: ComputationOutputs<ResolveGameOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(ResolveGameOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        if result == 0 {
            // Player busts (dealer wins)
            emit!(ResultEvent {
                winner: "Dealer".to_string(),
                game_id: ctx.accounts.blackjack_game.game_id,
            });
        } else if result == 1 {
            // Dealer busts (player wins)
            emit!(ResultEvent {
                winner: "Player".to_string(),
                game_id: ctx.accounts.blackjack_game.game_id,
            });
        } else if result == 2 {
            // Player wins
            emit!(ResultEvent {
                winner: "Player".to_string(),
                game_id: ctx.accounts.blackjack_game.game_id,
            });
        } else if result == 3 {
            // Dealer wins
            emit!(ResultEvent {
                winner: "Dealer".to_string(),
                game_id: ctx.accounts.blackjack_game.game_id,
            });
        } else {
            // Push (tie)
            emit!(ResultEvent {
                winner: "Tie".to_string(),
                game_id: ctx.accounts.blackjack_game.game_id,
            });
        }

        let blackjack_game = &mut ctx.accounts.blackjack_game;
        blackjack_game.game_state = GameState::Resolved;

        Ok(())
    }
}

#[queue_computation_accounts("shuffle_and_deal_cards", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, game_id: u64)]
pub struct InitializeBlackjackGame<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_SHUFFLE_AND_DEAL_CARDS)
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
        space = 8 + BlackjackGame::INIT_SPACE,
        seeds = [b"blackjack_game".as_ref(), game_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[callback_accounts("shuffle_and_deal_cards")]
#[derive(Accounts)]
pub struct ShuffleAndDealCardsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_SHUFFLE_AND_DEAL_CARDS)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[init_computation_definition_accounts("shuffle_and_deal_cards", payer)]
#[derive(Accounts)]
pub struct InitShuffleAndDealCardsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("player_hit", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _game_id: u64)]
pub struct PlayerHit<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLAYER_HIT)
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
        mut,
        seeds = [b"blackjack_game".as_ref(), _game_id.to_le_bytes().as_ref()],
        bump = blackjack_game.bump,
    )]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[callback_accounts("player_hit")]
#[derive(Accounts)]
pub struct PlayerHitCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLAYER_HIT)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[init_computation_definition_accounts("player_hit", payer)]
#[derive(Accounts)]
pub struct InitPlayerHitCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("player_double_down", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _game_id: u64)]
pub struct PlayerDoubleDown<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLAYER_DOUBLE_DOWN)
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
        mut,
        seeds = [b"blackjack_game".as_ref(), _game_id.to_le_bytes().as_ref()],
        bump = blackjack_game.bump,
    )]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[callback_accounts("player_double_down")]
#[derive(Accounts)]
pub struct PlayerDoubleDownCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLAYER_DOUBLE_DOWN)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[init_computation_definition_accounts("player_double_down", payer)]
#[derive(Accounts)]
pub struct InitPlayerDoubleDownCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("player_stand", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _game_id: u64)]
pub struct PlayerStand<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLAYER_STAND)
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
        mut,
        seeds = [b"blackjack_game".as_ref(), _game_id.to_le_bytes().as_ref()],
        bump = blackjack_game.bump,
    )]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[callback_accounts("player_stand")]
#[derive(Accounts)]
pub struct PlayerStandCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLAYER_STAND)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[init_computation_definition_accounts("player_stand", payer)]
#[derive(Accounts)]
pub struct InitPlayerStandCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("dealer_play", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _game_id: u64)]
pub struct DealerPlay<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_DEALER_PLAY)
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
        mut,
        seeds = [b"blackjack_game".as_ref(), _game_id.to_le_bytes().as_ref()],
        bump = blackjack_game.bump,
    )]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[callback_accounts("dealer_play")]
#[derive(Accounts)]
pub struct DealerPlayCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_DEALER_PLAY)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[init_computation_definition_accounts("dealer_play", payer)]
#[derive(Accounts)]
pub struct InitDealerPlayCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("resolve_game", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _game_id: u64)]
pub struct ResolveGame<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_RESOLVE_GAME)
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
        mut,
        seeds = [b"blackjack_game".as_ref(), _game_id.to_le_bytes().as_ref()],
        bump = blackjack_game.bump,
    )]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[callback_accounts("resolve_game")]
#[derive(Accounts)]
pub struct ResolveGameCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_RESOLVE_GAME)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub blackjack_game: Account<'info, BlackjackGame>,
}

#[init_computation_definition_accounts("resolve_game", payer)]
#[derive(Accounts)]
pub struct InitResolveGameCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

/// Represents a single blackjack game session.
///
/// This account stores all the game state including encrypted hands, deck information,
/// and game progress. The deck is stored as three 32-byte encrypted chunks that together
/// represent all 52 cards in shuffled order. Hands are stored encrypted and only
/// decryptable by their respective owners (player) or the MPC network (dealer).
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
