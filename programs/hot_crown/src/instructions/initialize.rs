use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = GAME_STATE_SPACE,
        seeds = [GAME_STATE_SEED],
        bump,
    )]
    pub game_state: Account<'info, GameState>,

    /// The SPL token mint used by the game
    /// TODO: Token mint address to be provided
    pub token_mint: Account<'info, Mint>,

    /// PDA-owned token account that holds throne pot + battle pools
    #[account(
        init,
        payer = admin,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
    )]
    pub throne_vault: Account<'info, TokenAccount>,

    /// Dev wallet's associated token account for receiving fees
    #[account(
        token::mint = token_mint,
    )]
    pub dev_wallet_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    game_state.admin = ctx.accounts.admin.key();
    game_state.token_mint = ctx.accounts.token_mint.key();
    game_state.dev_wallet_ata = ctx.accounts.dev_wallet_ata.key();
    game_state.paused = false;

    game_state.phase = GamePhase::Bidding;

    game_state.candidate = Pubkey::default();
    game_state.next_bid_amount = 1;
    game_state.last_bid_amount = 0;
    game_state.bidding_deadline = 0;
    game_state.throne_pot = 0;

    game_state.king = Pubkey::default();
    game_state.battle_active = false;
    game_state.attack_soldiers = 0;
    game_state.defense_soldiers = 0;
    game_state.attack_pool = 0;
    game_state.defense_pool = 0;
    game_state.battle_deadline = 0;

    game_state.bump = ctx.bumps.game_state;

    Ok(())
}
