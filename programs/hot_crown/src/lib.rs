use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod helpers;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("DwtDoUcKTCfkw2hSLwkYf6HVNFmtoMk7VYBjAJek5ixb");

#[program]
pub mod hot_crown {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        instructions::update_config::handler(ctx, params)
    }

    pub fn place_throne_bid(ctx: Context<PlaceThroneBid>) -> Result<()> {
        instructions::place_throne_bid::handler(ctx)
    }

    pub fn finalize_king_election(ctx: Context<FinalizeKingElection>) -> Result<()> {
        instructions::finalize_king_election::handler(ctx)
    }

    pub fn attack(ctx: Context<Attack>, soldiers: u64) -> Result<()> {
        instructions::attack::handler(ctx, soldiers)
    }

    pub fn defend(ctx: Context<Defend>, soldiers: u64) -> Result<()> {
        instructions::defend::handler(ctx, soldiers)
    }

    pub fn finalize_battle(ctx: Context<FinalizeBattle>) -> Result<()> {
        instructions::finalize_battle::handler(ctx)
    }
}
