use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[cfg(not(feature = "test-bpf"))]
use crate::config::mints::SRM;

#[derive(Accounts)]
pub struct DepositSRMVest<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Owner account for which the vest is being created.
    pub owner: AccountInfo<'info>,

    #[cfg_attr(
        not(feature = "test-bpf"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = srm_mint,
        associated_token::authority = payer
    )]
    pub payer_srm_account: Account<'info, TokenAccount>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    // #[account(
    //     seeds = [b"config"],
    //     bump,
    // )]
    // pub config: Account<'info, Config>,
    #[account(
        mut,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
