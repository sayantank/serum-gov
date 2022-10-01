use anchor_lang::prelude::*;

use crate::state::Config;

#[derive(Accounts)]
pub struct UpdateConfigParams<'info> {
    pub config_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(
    ctx: Context<UpdateConfigParams>,
    claim_delay: i64,
    redeem_delay: i64,
    cliff_period: i64,
    linear_vesting_period: i64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.claim_delay = claim_delay;
    config.redeem_delay = redeem_delay;
    config.cliff_period = cliff_period;
    config.linear_vesting_period = linear_vesting_period;

    Ok(())
}
