use anchor_lang::prelude::*;

#[account]
pub struct User {
    pub owner: Pubkey,
    pub bump: u8,
    pub lock_index: u64,
    pub vest_index: u64,
}

impl User {
    pub const LEN: usize = 8 + 32 + 1 + 8 + 8;
}

#[account]
pub struct LockedAccount {
    pub owner: Pubkey,
    pub bump: u8,
    pub lock_index: u64,
    pub redeem_index: u64,
    pub is_msrm: bool,
    pub created_at: i64,
    pub total_gsrm_amount: u64,
    pub gsrm_burned: u64,
}

impl LockedAccount {
    pub const LEN: usize = 8 + 32 + 1 + 8 + 8 + 1 + 8 + 8 + 8;
}

#[account]
pub struct VestAccount {
    pub owner: Pubkey,
    pub bump: u8,
    pub vest_index: u64,
    pub redeem_index: u64,
    pub is_msrm: bool,
    pub created_at: i64,
    pub cliff_period: i64,
    pub linear_vesting_period: i64,
    pub total_gsrm_amount: u64,
    pub gsrm_burned: u64,
}

impl VestAccount {
    pub const LEN: usize = 8 + 32 + 1 + 8 + 8 + 1 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct ClaimTicket {
    pub owner: Pubkey,
    pub deposit_account: Pubkey,
    pub bump: u8,
    pub created_at: i64,
    pub claim_delay: i64,
    pub gsrm_amount: u64,
}

impl ClaimTicket {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 8 + 8;
}

#[account]
pub struct RedeemTicket {
    pub owner: Pubkey,
    pub deposit_account: Pubkey,
    pub redeem_index: u64,
    pub bump: u8,
    pub is_msrm: bool,
    pub created_at: i64,
    pub redeem_delay: i64,
    pub amount: u64,
}

impl RedeemTicket {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 1 + 8 + 8 + 8;
}
