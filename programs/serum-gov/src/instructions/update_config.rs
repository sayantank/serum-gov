use anchor_lang::prelude::*;

#[cfg(not(feature = "test"))]
use crate::config::authority::UPGRADE_AUTHORITY;
use crate::state::Config;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[cfg_attr(
        not(feature = "test"),
        account(address = UPGRADE_AUTHORITY),
    )]
    pub upgrade_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(ctx: Context<UpdateConfig>, claim_delay: i64, redeem_delay: i64) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.claim_delay = claim_delay;
    config.redeem_delay = redeem_delay;

    Ok(())
}
