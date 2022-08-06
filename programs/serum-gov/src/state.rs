use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub claim_delay: i64,
    pub redeem_delay: i64,
}

#[account]
pub struct User {
    pub owner: Pubkey,
    pub bump: u8,
    pub claim_index: u64,
    pub redeem_index: u64,
    pub vest_index: u64,
}

#[account]
pub struct ClaimTicket {
    pub owner: Pubkey,
    pub is_msrm: bool,
    pub bump: u8,
    pub created_at: i64,
    pub claim_delay: i64,
    pub amount: u64,
    pub claim_index: u64,
}

#[account]
pub struct RedeemTicket {
    pub owner: Pubkey,
    pub is_msrm: bool,
    pub bump: u8,
    pub created_at: i64,
    pub redeem_delay: i64,
    pub amount: u64,
    pub redeem_index: u64,
}

#[account]
pub struct VestTicket {
    pub owner: Pubkey,
    pub is_msrm: bool, // Always false for now, might be used for future features.
    pub bump: bool,
    pub amount: u64,
    pub created_at: i64,
    pub claim_delay: i64,
    pub redeem_delay: i64,
    pub cliff_period: i64,
    pub linear_vesting_period: i64,
    pub vest_index: u64,
}
