use anchor_lang::prelude::*;

declare_id!("FLjKvZ1sANZud13kx6p6bKtaRKQtCC2VQaFN2aUysZUb");

pub mod config;
pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;

const MSRM_MULTIPLIER: u64 = 1_000_000_000_000;

#[program]
pub mod serum_gov {

    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        init::handler(ctx)
    }

    pub fn init_user(ctx: Context<InitUser>, owner: Pubkey) -> Result<()> {
        init_user::handler(ctx, owner)
    }

    pub fn deposit_locked_srm(ctx: Context<DepositLockedSRM>, amount: u64) -> Result<()> {
        deposit_locked_srm::handler(ctx, amount)
    }

    pub fn deposit_locked_msrm(ctx: Context<DepositLockedMSRM>, amount: u64) -> Result<()> {
        deposit_locked_msrm::handler(ctx, amount)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        claim::handler(ctx)
    }

    pub fn burn_locked_gsrm(
        ctx: Context<BurnLockedGSRM>,
        lock_index: u64,
        amount: u64,
    ) -> Result<()> {
        burn_locked_gsrm::handler(ctx, lock_index, amount)
    }

    pub fn redeem_srm(ctx: Context<RedeemSRM>) -> Result<()> {
        redeem_srm::handler(ctx)
    }

    pub fn redeem_msrm(ctx: Context<RedeemMSRM>) -> Result<()> {
        redeem_msrm::handler(ctx)
    }

    pub fn deposit_vest_srm(ctx: Context<DepositVestSRM>, amount: u64) -> Result<()> {
        deposit_vest_srm::handler(ctx, amount)
    }

    pub fn burn_vest_gsrm(ctx: Context<BurnVestGSRM>, vest_index: u64, amount: u64) -> Result<()> {
        burn_vest_gsrm::handler(ctx, vest_index, amount)
    }
}
