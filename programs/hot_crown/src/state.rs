use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Bidding,
    Battle,
}

#[account]
pub struct GameState {
    // --- Config ---
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub dev_wallet_ata: Pubkey,
    pub paused: bool,

    // --- Phase ---
    pub phase: GamePhase,

    // --- Bidding state ---
    pub candidate: Pubkey,        // Pubkey::default() = no candidate
    pub next_bid_amount: u64,     // whole tokens (starts at 1)
    pub last_bid_amount: u64,     // whole tokens
    pub bidding_deadline: i64,    // unix timestamp, 0 = no deadline
    pub throne_pot: u64,          // raw token units

    // --- Battle state ---
    pub king: Pubkey,             // Pubkey::default() = no king
    pub battle_active: bool,
    pub attack_soldiers: u64,     // whole soldier count
    pub defense_soldiers: u64,    // whole soldier count
    pub attack_pool: u64,         // raw token units (after dev fee)
    pub defense_pool: u64,        // raw token units (after dev fee)
    pub battle_deadline: i64,     // unix timestamp, 0 = no deadline

    // --- PDA ---
    pub bump: u8,
}
