use anchor_lang::prelude::*;

use crate::state::User;

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [b"user", &owner.key().to_bytes()[..]],
        bump,
        space = 8 + std::mem::size_of::<User>()
    )]
    pub user_account: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitUser>) -> Result<()> {
    let user = &mut ctx.accounts.user_account;

    user.owner = ctx.accounts.owner.key();
    user.bump = *ctx.bumps.get("user_account").unwrap();
    user.claim_index = 0;
    user.redeem_index = 0;
    user.vest_index = 0;

    Ok(())
}
