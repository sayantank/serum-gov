use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::errors::*;
use crate::state::ClaimTicket;
use crate::MSRM_MULTIPLIER;

#[derive(Accounts)]
#[instruction(claim_index: u64)]
pub struct Claim<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"claim", &owner.key().to_bytes()[..], claim_index.to_string().as_bytes()],
        bump,
        constraint = (ticket.created_at + ticket.claim_delay) <= clock.unix_timestamp @ SerumGovError::TicketNotClaimable,
        close = owner
    )]
    pub ticket: Account<'info, ClaimTicket>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"gSRM"],
        bump,
        mint::decimals = 6,
        mint::authority = authority,
    )]
    pub gsrm_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = gsrm_mint,
        associated_token::authority = owner
    )]
    pub owner_gsrm_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    fn mint_gsrm(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.gsrm_mint.to_account_info().clone(),
            to: self.owner_gsrm_account.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<Claim>, _claim_index: u64) -> Result<()> {
    let ticket = &mut ctx.accounts.ticket;

    let mint_amount = if ticket.is_msrm {
        ticket.amount.checked_mul(MSRM_MULTIPLIER).unwrap()
    } else {
        ticket.amount
    };

    token::mint_to(
        ctx.accounts
            .mint_gsrm()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        mint_amount,
    )?;

    Ok(())
}