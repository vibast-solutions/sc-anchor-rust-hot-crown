use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::HotCrownError;
use crate::helpers::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Defend<'info> {
    #[account(mut)]
    pub defender: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = defender,
    )]
    pub defender_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
    )]
    pub throne_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = dev_wallet_ata.key() == game_state.dev_wallet_ata @ HotCrownError::Unauthorized,
    )]
    pub dev_wallet_ata: Account<'info, TokenAccount>,

    #[account(
        constraint = token_mint.key() == game_state.token_mint @ HotCrownError::InvalidPhase,
    )]
    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Defend>, soldiers: u64) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    // Validations
    require!(!game_state.paused, HotCrownError::GamePaused);
    require!(game_state.phase == GamePhase::Battle, HotCrownError::InvalidPhase);
    require!(game_state.king != Pubkey::default(), HotCrownError::InvalidPhase);
    require!(game_state.battle_active, HotCrownError::NoAttackYet);
    require!(game_state.attack_soldiers > 0, HotCrownError::NoAttackYet);
    validate_soldiers(soldiers)?;

    // Check timer
    if game_state.battle_deadline != 0 {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp <= game_state.battle_deadline,
            HotCrownError::BattleExpired
        );
    }

    // Turn restriction: defend allowed only if attack >= defense
    require!(
        game_state.attack_soldiers >= game_state.defense_soldiers,
        HotCrownError::TurnRestriction
    );

    // Calculate amounts
    let total_raw = soldiers
        .checked_mul(ONE_TOKEN)
        .ok_or(HotCrownError::Overflow)?;
    let dev_fee = calc_dev_fee(total_raw)?;
    let army_contribution = total_raw
        .checked_sub(dev_fee)
        .ok_or(HotCrownError::Overflow)?;

    // Transfer dev fee: defender -> dev wallet
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.defender_token_account.to_account_info(),
                to: ctx.accounts.dev_wallet_ata.to_account_info(),
                authority: ctx.accounts.defender.to_account_info(),
            },
        ),
        dev_fee,
    )?;

    // Transfer army contribution: defender -> throne vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.defender_token_account.to_account_info(),
                to: ctx.accounts.throne_vault.to_account_info(),
                authority: ctx.accounts.defender.to_account_info(),
            },
        ),
        army_contribution,
    )?;

    // Update state
    let clock = Clock::get()?;
    game_state.defense_soldiers = game_state
        .defense_soldiers
        .checked_add(soldiers)
        .ok_or(HotCrownError::Overflow)?;
    game_state.defense_pool = game_state
        .defense_pool
        .checked_add(army_contribution)
        .ok_or(HotCrownError::Overflow)?;
    game_state.battle_deadline = clock
        .unix_timestamp
        .checked_add(TIMER_DURATION_SECONDS)
        .ok_or(HotCrownError::Overflow)?;

    Ok(())
}
