use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TransferChecked};

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
        token::token_program = token_program,
    )]
    pub defender_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
        associated_token::token_program = token_program,
    )]
    pub throne_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = dev_wallet_ata.key() == game_state.dev_wallet_ata @ HotCrownError::Unauthorized,
    )]
    pub dev_wallet_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        constraint = token_mint.key() == game_state.token_mint @ HotCrownError::InvalidPhase,
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token2022>,
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
        .checked_mul(game_state.one_token)
        .ok_or(HotCrownError::Overflow)?;
    let dev_fee = calc_dev_fee(total_raw)?;
    let army_contribution = total_raw
        .checked_sub(dev_fee)
        .ok_or(HotCrownError::Overflow)?;

    // Transfer dev fee: defender -> dev wallet
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.defender_token_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.dev_wallet_ata.to_account_info(),
                authority: ctx.accounts.defender.to_account_info(),
            },
        ),
        dev_fee,
        TOKEN_DECIMALS,
    )?;

    // Transfer army contribution: defender -> throne vault
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.defender_token_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.throne_vault.to_account_info(),
                authority: ctx.accounts.defender.to_account_info(),
            },
        ),
        army_contribution,
        TOKEN_DECIMALS,
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
