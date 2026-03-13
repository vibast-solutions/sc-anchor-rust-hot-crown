use crate::constants::*;
use crate::errors::HotCrownError;
use anchor_lang::prelude::*;

/// Calculate dev fee from a raw token amount (10%)
pub fn calc_dev_fee(raw_amount: u64) -> Result<u64> {
    raw_amount
        .checked_mul(DEV_FEE_BPS)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR))
        .ok_or_else(|| error!(HotCrownError::Overflow))
}

/// Calculate burn amount from a raw token amount (10%, bidding only)
pub fn calc_burn(raw_amount: u64) -> Result<u64> {
    raw_amount
        .checked_mul(BURN_BPS)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR))
        .ok_or_else(|| error!(HotCrownError::Overflow))
}

/// Validate soldier count is between 1 and 10
pub fn validate_soldiers(soldiers: u64) -> Result<()> {
    require!(
        soldiers >= MIN_SOLDIERS_PER_ACTION && soldiers <= MAX_SOLDIERS_PER_ACTION,
        HotCrownError::InvalidSoldierCount
    );
    Ok(())
}
