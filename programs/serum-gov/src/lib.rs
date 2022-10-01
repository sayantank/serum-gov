use anchor_lang::prelude::*;

declare_id!("RE3xgnnxDjhXoPMqEzeKLj8ThrdXwdDa168GCEGoY6Y");

pub mod config;
pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;

const MSRM_MULTIPLIER: u64 = 1_000_000_000_000;

#[program]
pub mod serum_gov {

    use super::*;

    pub fn init(
        ctx: Context<Init>,
        name: String,
        symbol: String,
        claim_delay: i64,
        redeem_delay: i64,
        cliff_period: i64,
        linear_vesting_period: i64,
    ) -> Result<()> {
        init::handler(
            ctx,
            name,
            symbol,
            claim_delay,
            redeem_delay,
            cliff_period,
            linear_vesting_period,
        )
    }

    pub fn update_config_authority(
        ctx: Context<UpdateConfigAuthority>,
        new_config_authority: Pubkey,
    ) -> Result<()> {
        update_config_authority::handler(ctx, new_config_authority)
    }

    pub fn update_config_params(
        ctx: Context<UpdateConfigParams>,
        claim_delay: i64,
        redeem_delay: i64,
        cliff_period: i64,
        linear_vesting_period: i64,
    ) -> Result<()> {
        update_config_params::handler(
            ctx,
            claim_delay,
            redeem_delay,
            cliff_period,
            linear_vesting_period,
        )
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

    pub fn burn_locked_gsrm(ctx: Context<BurnLockedGSRM>, amount: u64) -> Result<()> {
        burn_locked_gsrm::handler(ctx, amount)
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

    pub fn deposit_vest_msrm(ctx: Context<DepositVestMSRM>, amount: u64) -> Result<()> {
        deposit_vest_msrm::handler(ctx, amount)
    }

    pub fn burn_vest_gsrm(ctx: Context<BurnVestGSRM>, amount: u64) -> Result<()> {
        burn_vest_gsrm::handler(ctx, amount)
    }
}
