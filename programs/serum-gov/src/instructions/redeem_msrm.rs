use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[cfg(not(feature = "test-bpf"))]
use crate::config::mints::MSRM;
use crate::errors::*;
use crate::state::RedeemTicket;

#[derive(Accounts)]
pub struct RedeemMSRM<'info> {
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
        constraint = redeem_ticket.is_msrm == true @ SerumGovError::InvalidRedeemTicket,
        constraint = (redeem_ticket.created_at + redeem_ticket.redeem_delay) <= clock.unix_timestamp @ SerumGovError::TicketNotClaimable,
        constraint = redeem_ticket.amount > 0 @ SerumGovError::TicketNotClaimable,
        close = owner
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    #[cfg_attr(
        not(feature = "test-bpf"),
        account(address = MSRM),
    )]
    pub msrm_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", &msrm_mint.key().to_bytes()[..]],
        bump,
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = msrm_mint,
        token::authority = owner
    )]
    pub owner_msrm_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RedeemMSRM<'info> {
    fn into_redeem_msrm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.msrm_vault.to_account_info().clone(),
            to: self.owner_msrm_account.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<RedeemMSRM>) -> Result<()> {
    token::transfer(
        ctx.accounts
            .into_redeem_msrm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        ctx.accounts.redeem_ticket.amount,
    )?;

    let redeem_ticket = &mut ctx.accounts.redeem_ticket;
    redeem_ticket.amount = 0;

    Ok(())
}
