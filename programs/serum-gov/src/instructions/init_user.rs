use anchor_lang::prelude::*;

use crate::state::User;

#[derive(Accounts)]
#[instruction(owner: Pubkey)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"user", &owner.to_bytes()[..]],
        bump,
        space = 8 + std::mem::size_of::<User>()
    )]
    pub user_account: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitUser>, owner: Pubkey) -> Result<()> {
    let user = &mut ctx.accounts.user_account;

    user.owner = owner;
    user.bump = *ctx.bumps.get("user_account").unwrap();
    user.lock_index = 0;
    user.vest_index = 0;

    Ok(())
}
