use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::HotCrownError;
use crate::helpers::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Attack<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = attacker,
    )]
    pub attacker_token_account: Account<'info, TokenAccount>,

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

pub fn handler(ctx: Context<Attack>, soldiers: u64) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    // Validations
    require!(!game_state.paused, HotCrownError::GamePaused);
    require!(game_state.phase == GamePhase::Battle, HotCrownError::InvalidPhase);
    require!(game_state.king != Pubkey::default(), HotCrownError::InvalidPhase);
    validate_soldiers(soldiers)?;

    // If battle is active, check timer and turn restriction
    if game_state.battle_active {
        if game_state.battle_deadline != 0 {
            let clock = Clock::get()?;
            require!(
                clock.unix_timestamp <= game_state.battle_deadline,
                HotCrownError::BattleExpired
            );
        }
        // Turn restriction: attack allowed only if defense >= attack
        require!(
            game_state.defense_soldiers >= game_state.attack_soldiers,
            HotCrownError::TurnRestriction
        );
    }
    // If battle not active, this is the first attack — no restriction

    // Calculate amounts
    let total_raw = soldiers
        .checked_mul(game_state.one_token)
        .ok_or(HotCrownError::Overflow)?;
    let dev_fee = calc_dev_fee(total_raw)?;
    let army_contribution = total_raw
        .checked_sub(dev_fee)
        .ok_or(HotCrownError::Overflow)?;

    // Transfer dev fee: attacker -> dev wallet
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.attacker_token_account.to_account_info(),
                to: ctx.accounts.dev_wallet_ata.to_account_info(),
                authority: ctx.accounts.attacker.to_account_info(),
            },
        ),
        dev_fee,
    )?;

    // Transfer army contribution: attacker -> throne vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.attacker_token_account.to_account_info(),
                to: ctx.accounts.throne_vault.to_account_info(),
                authority: ctx.accounts.attacker.to_account_info(),
            },
        ),
        army_contribution,
    )?;

    // Update state
    let clock = Clock::get()?;
    game_state.attack_soldiers = game_state
        .attack_soldiers
        .checked_add(soldiers)
        .ok_or(HotCrownError::Overflow)?;
    game_state.attack_pool = game_state
        .attack_pool
        .checked_add(army_contribution)
        .ok_or(HotCrownError::Overflow)?;
    game_state.battle_active = true;
    game_state.battle_deadline = clock
        .unix_timestamp
        .checked_add(TIMER_DURATION_SECONDS)
        .ok_or(HotCrownError::Overflow)?;

    Ok(())
}
