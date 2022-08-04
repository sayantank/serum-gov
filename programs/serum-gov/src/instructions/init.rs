use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[cfg(not(feature = "test"))]
use crate::config::{
    authority::UPGRADE_AUTHORITY,
    mints::{MSRM, SRM},
};
use crate::state::Config;

#[derive(Accounts)]
pub struct Init<'info> {
    /// NOTE: Could add constraint to restrict authorized payer, but this ix can't be called twice anyway.
    #[account(mut)]
    #[cfg_attr(
        not(feature = "test"),
        account(address = UPGRADE_AUTHORITY),
    )]
    pub payer: Signer<'info>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"config"],
        bump,
        space = 8 + std::mem::size_of::<Config>()
    )]
    pub config: Account<'info, Config>,

    /// NOTE: Decimals have been kept same as SRM.
    #[account(
        init,
        payer = payer,
        seeds = [b"gSRM"],
        bump,
        mint::decimals = 6,
        mint::authority = authority,
    )]
    pub gsrm_mint: Account<'info, Mint>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"vault", &srm_mint.key().to_bytes()[..]],
        bump,
        payer = payer,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = MSRM),
    )]
    pub msrm_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"vault", &msrm_mint.key().to_bytes()[..]],
        bump,
        payer = payer,
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Init>, claim_delay: i64, redeem_delay: i64) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.claim_delay = claim_delay;
    config.redeem_delay = redeem_delay;

    Ok(())
}
