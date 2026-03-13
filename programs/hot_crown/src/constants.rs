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

pub const TOKEN_DECIMALS: u8 = 6;
pub const ONE_TOKEN: u64 = 1_000_000; // 10^6
