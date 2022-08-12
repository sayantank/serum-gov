use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[cfg(not(feature = "test-bpf"))]
use crate::config::mints::SRM;
use crate::errors::*;
use crate::state::RedeemTicket;

#[derive(Accounts)]
pub struct RedeemSRM<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = redeem_ticket.owner.key() == owner.key() @ SerumGovError::InvalidTicketOwner,
        constraint = redeem_ticket.is_msrm == false @ SerumGovError::InvalidRedeemTicket,
        constraint = (redeem_ticket.created_at + redeem_ticket.redeem_delay) <= clock.unix_timestamp @ SerumGovError::TicketNotClaimable,
        close = owner
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    #[cfg_attr(
        not(feature = "test-bpf"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", &srm_mint.key().to_bytes()[..]],
        bump,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = srm_mint,
        token::authority = owner
    )]
    pub owner_srm_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RedeemSRM<'info> {
    fn into_redeem_srm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.srm_vault.to_account_info().clone(),
            to: self.owner_srm_account.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<RedeemSRM>) -> Result<()> {
    token::transfer(
        ctx.accounts
            .into_redeem_srm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        ctx.accounts.redeem_ticket.amount,
    )?;
    Ok(())
}
