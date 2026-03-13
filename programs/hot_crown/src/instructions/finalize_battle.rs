use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::HotCrownError;
use crate::state::*;

#[derive(Accounts)]
pub struct FinalizeBattle<'info> {
    /// Anyone can call this (permissionless)
    pub anyone: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
    )]
    pub throne_vault: Account<'info, TokenAccount>,

    /// King's token account (receives payout if king survives)
    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub king_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_mint.key() == game_state.token_mint @ HotCrownError::InvalidPhase,
    )]
    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<FinalizeBattle>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    // Validations
    require!(game_state.phase == GamePhase::Battle, HotCrownError::InvalidPhase);
    require!(game_state.battle_active, HotCrownError::NoBattle);
    require!(game_state.battle_deadline != 0, HotCrownError::NoBattle);

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp > game_state.battle_deadline,
        HotCrownError::BattleNotExpired
    );

    // Validate king token account belongs to the king
    require!(
        ctx.accounts.king_token_account.owner == game_state.king,
        HotCrownError::Unauthorized
    );

    let seeds = &[GAME_STATE_SEED, &[game_state.bump]];
    let signer_seeds = &[&seeds[..]];

    let king_survives = game_state.defense_soldiers >= game_state.attack_soldiers;

    if king_survives {
        // King survives: 50% defense pool to king, rest burned
        let king_payout = game_state.defense_pool / 2;
        let defense_burn = game_state
            .defense_pool
            .checked_sub(king_payout)
            .ok_or(HotCrownError::Overflow)?;
        let total_burn = defense_burn
            .checked_add(game_state.attack_pool)
            .ok_or(HotCrownError::Overflow)?;

        // Transfer payout to king
        if king_payout > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.throne_vault.to_account_info(),
                        to: ctx.accounts.king_token_account.to_account_info(),
                        authority: game_state.to_account_info(),
                    },
                    signer_seeds,
                ),
                king_payout,
            )?;
        }

        // Burn the rest
        if total_burn > 0 {
            token::burn(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.token_mint.to_account_info(),
                        from: ctx.accounts.throne_vault.to_account_info(),
                        authority: game_state.to_account_info(),
                    },
                    signer_seeds,
                ),
                total_burn,
            )?;
        }

        // Reset battle, king stays
        game_state.battle_active = false;
        game_state.attack_soldiers = 0;
        game_state.defense_soldiers = 0;
        game_state.attack_pool = 0;
        game_state.defense_pool = 0;
        game_state.battle_deadline = 0;
    } else {
        // King defeated: burn everything
        let total_burn = game_state
            .attack_pool
            .checked_add(game_state.defense_pool)
            .ok_or(HotCrownError::Overflow)?;

        if total_burn > 0 {
            token::burn(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.token_mint.to_account_info(),
                        from: ctx.accounts.throne_vault.to_account_info(),
                        authority: game_state.to_account_info(),
                    },
                    signer_seeds,
                ),
                total_burn,
            )?;
        }

        // Full reset to bidding
        game_state.king = Pubkey::default();
        game_state.battle_active = false;
        game_state.attack_soldiers = 0;
        game_state.defense_soldiers = 0;
        game_state.attack_pool = 0;
        game_state.defense_pool = 0;
        game_state.battle_deadline = 0;
        game_state.phase = GamePhase::Bidding;
        game_state.next_bid_amount = 1;
        game_state.last_bid_amount = 0;
        game_state.bidding_deadline = 0;
    }

    Ok(())
}
