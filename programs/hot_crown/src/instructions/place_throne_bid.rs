use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::HotCrownError;
use crate::helpers::*;
use crate::state::*;

#[derive(Accounts)]
pub struct PlaceThroneBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
    )]
    pub game_state: Account<'info, GameState>,

    /// Bidder's token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = bidder,
    )]
    pub bidder_token_account: Account<'info, TokenAccount>,

    /// PDA-owned vault holding throne pot + battle pools
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
    )]
    pub throne_vault: Account<'info, TokenAccount>,

    /// Dev wallet's token account for fee collection
    #[account(
        mut,
        constraint = dev_wallet_ata.key() == game_state.dev_wallet_ata @ HotCrownError::Unauthorized,
    )]
    pub dev_wallet_ata: Account<'info, TokenAccount>,

    /// Token mint (needed for burn)
    #[account(
        mut,
        constraint = token_mint.key() == game_state.token_mint @ HotCrownError::InvalidPhase,
    )]
    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<PlaceThroneBid>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    // Validations
    require!(!game_state.paused, HotCrownError::GamePaused);
    require!(game_state.phase == GamePhase::Bidding, HotCrownError::InvalidPhase);

    // If deadline exists and has expired, must finalize first
    if game_state.bidding_deadline != 0 {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp <= game_state.bidding_deadline,
            HotCrownError::BiddingExpired
        );
    }

    let bid_whole = game_state.next_bid_amount;
    let bid_raw = bid_whole
        .checked_mul(game_state.one_token)
        .ok_or(HotCrownError::Overflow)?;

    // Calculate splits
    let dev_fee = calc_dev_fee(bid_raw)?;
    let burn_amount = calc_burn(bid_raw)?;
    let pot_amount = bid_raw
        .checked_sub(dev_fee)
        .and_then(|v| v.checked_sub(burn_amount))
        .ok_or(HotCrownError::Overflow)?;

    // Transfer dev fee: bidder -> dev wallet
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.bidder_token_account.to_account_info(),
                to: ctx.accounts.dev_wallet_ata.to_account_info(),
                authority: ctx.accounts.bidder.to_account_info(),
            },
        ),
        dev_fee,
    )?;

    // Burn: from bidder's token account
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.bidder_token_account.to_account_info(),
                authority: ctx.accounts.bidder.to_account_info(),
            },
        ),
        burn_amount,
    )?;

    // Transfer pot amount: bidder -> throne vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.bidder_token_account.to_account_info(),
                to: ctx.accounts.throne_vault.to_account_info(),
                authority: ctx.accounts.bidder.to_account_info(),
            },
        ),
        pot_amount,
    )?;

    // Update state
    let clock = Clock::get()?;
    game_state.candidate = ctx.accounts.bidder.key();
    game_state.last_bid_amount = bid_whole;
    game_state.next_bid_amount = bid_whole
        .checked_add(1)
        .ok_or(HotCrownError::Overflow)?;
    game_state.throne_pot = game_state
        .throne_pot
        .checked_add(pot_amount)
        .ok_or(HotCrownError::Overflow)?;
    game_state.bidding_deadline = clock
        .unix_timestamp
        .checked_add(TIMER_DURATION_SECONDS)
        .ok_or(HotCrownError::Overflow)?;

    Ok(())
}
