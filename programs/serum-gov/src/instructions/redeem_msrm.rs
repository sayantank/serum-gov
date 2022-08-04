use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[cfg(not(feature = "test"))]
use crate::config::mints::MSRM;
use crate::errors::*;
use crate::state::RedeemTicket;

#[derive(Accounts)]
#[instruction(redeem_index: u64)]
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
        seeds = [b"redeem", &owner.key().to_bytes()[..], redeem_index.to_string().as_bytes()],
        bump,
        constraint = redeem_ticket.is_msrm == true @ SerumGovError::InvalidRedeemTicket,
        constraint = (redeem_ticket.created_at + redeem_ticket.redeem_delay) <= clock.unix_timestamp @ SerumGovError::TicketNotClaimable,
        close = owner
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = MSRM),
    )]
    pub msrm_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = msrm_mint,
        associated_token::authority = owner
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

pub fn handler(ctx: Context<RedeemMSRM>, _redeem_index: u64) -> Result<()> {
    token::transfer(
        ctx.accounts
            .into_redeem_msrm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        ctx.accounts.redeem_ticket.amount,
    )?;
    Ok(())
}
