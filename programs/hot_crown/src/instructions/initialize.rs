use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + GameState::INIT_SPACE,
        seeds = [GAME_STATE_SEED],
        bump,
    )]
    pub game_state: Account<'info, GameState>,

    /// The Token-2022 mint used by the game
    pub token_mint: InterfaceAccount<'info, Mint>,

    /// PDA-owned token account that holds throne pot + battle pools
    #[account(
        init,
        payer = admin,
        associated_token::mint = token_mint,
        associated_token::authority = game_state,
        associated_token::token_program = token_program,
    )]
    pub throne_vault: InterfaceAccount<'info, TokenAccount>,

    /// Dev wallet's associated token account for receiving fees
    #[account(
        token::mint = token_mint,
        token::token_program = token_program,
    )]
    pub dev_wallet_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    game_state.admin = ctx.accounts.admin.key();
    game_state.token_mint = ctx.accounts.token_mint.key();
    game_state.dev_wallet_ata = ctx.accounts.dev_wallet_ata.key();
    game_state.paused = false;
    game_state.one_token = ONE_TOKEN;

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
