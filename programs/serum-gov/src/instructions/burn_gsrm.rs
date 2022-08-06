use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::config::parameters::REDEEM_DELAY;
use crate::errors::*;
use crate::state::{ClaimTicket, RedeemTicket, User};
use crate::MSRM_MULTIPLIER;

#[derive(Accounts)]
pub struct BurnGSRM<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", &owner.key().to_bytes()[..]],
        bump,
    )]
    pub user_account: Account<'info, User>,

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

    #[account(
        init,
        payer = owner,
        seeds = [b"redeem", &owner.key().to_bytes()[..], user_account.redeem_index.to_string().as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<ClaimTicket>()
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BurnGSRM<'info> {
    fn into_burn_gsrm_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.gsrm_mint.to_account_info().clone(),
            from: self.owner_gsrm_account.to_account_info().clone(),
            authority: self.owner.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<BurnGSRM>, amount: u64, is_msrm: bool) -> Result<()> {
    if is_msrm && (amount % MSRM_MULTIPLIER != 0) {
        return err!(SerumGovError::InvalidMSRMAmount);
    }

    token::burn(
        ctx.accounts
            .into_burn_gsrm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        amount,
    )?;

    let redeem_amount = if is_msrm {
        amount / MSRM_MULTIPLIER
    } else {
        amount
    };

    let user_account = &mut ctx.accounts.user_account;

    let ticket = &mut ctx.accounts.redeem_ticket;
    ticket.owner = ctx.accounts.owner.key();
    ticket.is_msrm = is_msrm;
    ticket.bump = *ctx.bumps.get("redeem_ticket").unwrap();
    ticket.created_at = ctx.accounts.clock.unix_timestamp;
    ticket.redeem_delay = REDEEM_DELAY;
    ticket.amount = redeem_amount;
    ticket.redeem_index = user_account.redeem_index;

    user_account.redeem_index += 1;

    Ok(())
}
