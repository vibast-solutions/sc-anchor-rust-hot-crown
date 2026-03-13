use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::errors::HotCrownError;
use crate::state::*;

#[derive(Accounts)]
pub struct FinalizeKingElection<'info> {
    /// Anyone can call this (permissionless)
    pub anyone: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
    )]
    pub game_state: Account<'info, GameState>,

    /// PDA-owned vault
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
        associated_token::token_program = token_program,
    )]
    pub throne_vault: InterfaceAccount<'info, TokenAccount>,

    /// The candidate's (soon-to-be king's) token account to receive the throne pot
    #[account(
        mut,
        token::mint = token_mint,
        token::token_program = token_program,
    )]
    pub king_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Token mint (needed for transfer_checked)
    #[account(
        constraint = token_mint.key() == game_state.token_mint @ HotCrownError::InvalidPhase,
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<FinalizeKingElection>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    // Validations
    require!(game_state.phase == GamePhase::Bidding, HotCrownError::InvalidPhase);
    require!(game_state.candidate != Pubkey::default(), HotCrownError::NoCandidate);
    require!(game_state.bidding_deadline != 0, HotCrownError::NoCandidate);

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp > game_state.bidding_deadline,
        HotCrownError::BiddingNotExpired
    );

    // Validate the king token account belongs to the candidate
    require!(
        ctx.accounts.king_token_account.owner == game_state.candidate,
        HotCrownError::Unauthorized
    );

    // Transfer throne pot from vault to king
    let pot_amount = game_state.throne_pot;
    if pot_amount > 0 {
        let seeds = &[GAME_STATE_SEED, &[game_state.bump]];
        let signer_seeds = &[&seeds[..]];

        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.throne_vault.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.king_token_account.to_account_info(),
                    authority: game_state.to_account_info(),
                },
                signer_seeds,
            ),
            pot_amount,
            TOKEN_DECIMALS,
        )?;
    }

    // Transition to battle phase
    game_state.king = game_state.candidate;
    game_state.candidate = Pubkey::default();
    game_state.throne_pot = 0;
    game_state.next_bid_amount = 1;
    game_state.last_bid_amount = 0;
    game_state.bidding_deadline = 0;
    game_state.phase = GamePhase::Battle;
    game_state.battle_active = false;
    game_state.attack_soldiers = 0;
    game_state.defense_soldiers = 0;
    game_state.attack_pool = 0;
    game_state.defense_pool = 0;
    game_state.battle_deadline = 0;

    Ok(())
}
