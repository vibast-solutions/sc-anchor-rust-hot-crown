use anchor_lang::prelude::*;

#[error_code]
pub enum HotCrownError {
    #[msg("Game is paused")]
    GamePaused,
    #[msg("Invalid game phase for this action")]
    InvalidPhase,
    #[msg("Bid amount does not match required bid")]
    InvalidBidAmount,
    #[msg("Bidding timer has not expired yet")]
    BiddingNotExpired,
    #[msg("Bidding deadline already expired, must finalize first")]
    BiddingExpired,
    #[msg("No candidate exists")]
    NoCandidate,
    #[msg("Battle timer has not expired yet")]
    BattleNotExpired,
    #[msg("Battle deadline already expired, must finalize first")]
    BattleExpired,
    #[msg("No active battle")]
    NoBattle,
    #[msg("Soldiers per action must be between 1 and 10")]
    InvalidSoldierCount,
    #[msg("Turn restriction: your side is ahead")]
    TurnRestriction,
    #[msg("Cannot defend before first attack")]
    NoAttackYet,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
}
