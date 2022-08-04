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

    pub fn init(ctx: Context<Init>, claim_delay: i64, redeem_delay: i64) -> Result<()> {
        init::handler(ctx, claim_delay, redeem_delay)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        claim_delay: i64,
        redeem_delay: i64,
    ) -> Result<()> {
        update_config::handler(ctx, claim_delay, redeem_delay)
    }

    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        instructions::init_user::handler(ctx)
    }

    pub fn deposit_srm(ctx: Context<DepositSRM>, amount: u64) -> Result<()> {
        instructions::deposit_srm::handler(ctx, amount)
    }

    pub fn deposit_msrm(ctx: Context<DepositMSRM>, amount: u64) -> Result<()> {
        instructions::deposit_msrm::handler(ctx, amount)
    }

    pub fn claim(ctx: Context<Claim>, claim_index: u64) -> Result<()> {
        instructions::claim::handler(ctx, claim_index)
    }

    pub fn burn_gsrm(ctx: Context<BurnGSRM>, amount: u64, is_msrm: bool) -> Result<()> {
        instructions::burn_gsrm::handler(ctx, amount, is_msrm)
    }

    pub fn redeem_srm(ctx: Context<RedeemSRM>, redeem_index: u64) -> Result<()> {
        instructions::redeem_srm::handler(ctx, redeem_index)
    }

    pub fn redeem_msrm(ctx: Context<RedeemMSRM>, redeem_index: u64) -> Result<()> {
        instructions::redeem_msrm::handler(ctx, redeem_index)
    }
}
