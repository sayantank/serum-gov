use anchor_lang::prelude::*;

use crate::state::Config;

#[derive(Accounts)]
pub struct UpdateConfigAuthority<'info> {
    pub config_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = config_authority
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(ctx: Context<UpdateConfigAuthority>, new_config_authority: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.config_authority = new_config_authority;
    Ok(())
}
