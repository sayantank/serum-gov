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

    pub fn deposit_srm(ctx: Context<DepositSRM>, amount: u64) -> Result<()> {
        deposit_srm::handler(ctx, amount)
    }

    pub fn deposit_msrm(ctx: Context<DepositMSRM>, amount: u64) -> Result<()> {
        deposit_msrm::handler(ctx, amount)
    }

    pub fn claim(ctx: Context<Claim>, claim_index: u64) -> Result<()> {
        claim::handler(ctx, claim_index)
    }

    pub fn burn_gsrm(ctx: Context<BurnGSRM>, amount: u64, is_msrm: bool) -> Result<()> {
        burn_gsrm::handler(ctx, amount, is_msrm)
    }

    pub fn redeem_srm(ctx: Context<RedeemSRM>, redeem_index: u64) -> Result<()> {
        redeem_srm::handler(ctx, redeem_index)
    }

    pub fn redeem_msrm(ctx: Context<RedeemMSRM>, redeem_index: u64) -> Result<()> {
        redeem_msrm::handler(ctx, redeem_index)
    }

    // pub fn deposit_srm_vest(ctx: Context<DepositSRMVest>, amount: u64) -> Result<()> {
    //     Ok(())
    // }
}
