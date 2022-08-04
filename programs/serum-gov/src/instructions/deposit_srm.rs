use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[cfg(not(feature = "test"))]
use crate::config::mints::SRM;
use crate::state::{ClaimTicket, Config, User};

#[derive(Accounts)]
pub struct DepositSRM<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", &owner.key().to_bytes()[..]],
        bump,
    )]
    pub user_account: Account<'info, User>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = srm_mint,
        associated_token::authority = owner
    )]
    pub owner_srm_account: Account<'info, TokenAccount>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [b"claim", &owner.key().to_bytes()[..], user_account.claim_index.to_string().as_bytes()],
        bump,
        space =  8 + std::mem::size_of::<ClaimTicket>()
    )]
    pub claim_ticket: Account<'info, ClaimTicket>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositSRM<'info> {
    fn into_deposit_srm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.owner_srm_account.to_account_info().clone(),
            to: self.srm_vault.to_account_info().clone(),
            authority: self.owner.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<DepositSRM>, amount: u64) -> Result<()> {
    token::transfer(ctx.accounts.into_deposit_srm_context(), amount)?;

    let user_account = &mut ctx.accounts.user_account;

    let ticket = &mut ctx.accounts.claim_ticket;
    ticket.owner = ctx.accounts.owner.key();
    ticket.is_msrm = false;
    ticket.bump = *ctx.bumps.get("claim_ticket").unwrap();
    ticket.created_at = ctx.accounts.clock.unix_timestamp;
    ticket.claim_delay = ctx.accounts.config.claim_delay;
    ticket.amount = amount;
    ticket.claim_index = user_account.claim_index;

    user_account.claim_index += 1;

    Ok(())
}
