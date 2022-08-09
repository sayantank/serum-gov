use anchor_lang::prelude::*;

declare_id!("G8aGmsybmGqQisBuRrDqiGfdz8YcCJcU3agWDFmTkf8S");

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

    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        init_user::handler(ctx)
    }

    pub fn deposit_locked_srm(ctx: Context<DepositSRM>, amount: u64) -> Result<()> {
        deposit_locked_srm::handler(ctx, amount)
    }

    pub fn deposit_locked_msrm(ctx: Context<DepositMSRM>, amount: u64) -> Result<()> {
        deposit_locked_msrm::handler(ctx, amount)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        claim::handler(ctx)
    }

    pub fn burn_locked_gsrm(ctx: Context<BurnGSRM>, lock_index: u64, amount: u64) -> Result<()> {
        burn_locked_gsrm::handler(ctx, lock_index, amount)
    }

    pub fn redeem_srm(ctx: Context<RedeemSRM>) -> Result<()> {
        redeem_srm::handler(ctx)
    }

    pub fn redeem_msrm(ctx: Context<RedeemMSRM>) -> Result<()> {
        redeem_msrm::handler(ctx)
    }

    // pub fn deposit_srm_vest(ctx: Context<DepositSRMVest>, amount: u64) -> Result<()> {
    //     Ok(())
    // }
}
