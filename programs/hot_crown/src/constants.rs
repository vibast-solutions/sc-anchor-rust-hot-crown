pub const GAME_STATE_SEED: &[u8] = b"game_state";
pub const THRONE_VAULT_SEED: &[u8] = b"throne_vault";

pub const TIMER_DURATION_SECONDS: i64 = 180; // 3 minutes

// Basis points (out of 10,000)
pub const DEV_FEE_BPS: u64 = 1_000;   // 10%
pub const BURN_BPS: u64 = 1_000;       // 10% (bidding only)
pub const POT_BPS: u64 = 8_000;        // 80% (bidding only)
pub const ARMY_BPS: u64 = 9_000;       // 90% (battle only)
pub const BPS_DENOMINATOR: u64 = 10_000;

pub const MIN_SOLDIERS_PER_ACTION: u64 = 1;
pub const MAX_SOLDIERS_PER_ACTION: u64 = 10;

// Account space: 8 (discriminator) + struct fields
// Config: admin(32) + token_mint(32) + dev_wallet_ata(32) + paused(1) + one_token(8)
// Phase: phase(1)
// Bidding: candidate(32) + next_bid(8) + last_bid(8) + deadline(8) + pot(8)
// Battle: king(32) + active(1) + atk_soldiers(8) + def_soldiers(8) + atk_pool(8) + def_pool(8) + deadline(8)
// PDA: bump(1)
pub const GAME_STATE_SPACE: usize = 8 + 32 + 32 + 32 + 1 + 8 + 1 + 32 + 8 + 8 + 8 + 8 + 32 + 1 + 8 + 8 + 8 + 8 + 8 + 1;
