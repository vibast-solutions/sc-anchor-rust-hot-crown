use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::HotCrownError;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.admin == admin.key() @ HotCrownError::Unauthorized,
    )]
    pub game_state: Account<'info, GameState>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateConfigParams {
    pub new_admin: Option<Pubkey>,
    pub new_dev_wallet_ata: Option<Pubkey>,
    pub paused: Option<bool>,
}

pub fn handler(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    if let Some(new_admin) = params.new_admin {
        game_state.admin = new_admin;
    }
    if let Some(new_dev_wallet_ata) = params.new_dev_wallet_ata {
        game_state.dev_wallet_ata = new_dev_wallet_ata;
    }
    if let Some(paused) = params.paused {
        game_state.paused = paused;
    }

    Ok(())
}
