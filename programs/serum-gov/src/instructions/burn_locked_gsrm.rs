use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::config::parameters::REDEEM_DELAY;
use crate::errors::*;
use crate::state::{LockedAccount, RedeemTicket};
use crate::MSRM_MULTIPLIER;

#[derive(Accounts)]
pub struct BurnLockedGSRM<'info> {
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
        seeds = [b"gSRM"],
        bump,
        mint::decimals = 6,
        mint::authority = authority,
    )]
    pub gsrm_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = gsrm_mint,
        token::authority = owner
    )]
    pub owner_gsrm_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"locked_account", &owner.key().to_bytes()[..], locked_account.lock_index.to_le_bytes().as_ref()],
        bump,
    )]
    pub locked_account: Account<'info, LockedAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [b"redeem_ticket", &locked_account.key().to_bytes()[..], locked_account.redeem_index.to_le_bytes().as_ref()],
        bump,
        space = RedeemTicket::LEN
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BurnLockedGSRM<'info> {
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

pub fn handler(ctx: Context<BurnLockedGSRM>, amount: u64) -> Result<()> {
    // Doesn't matter if placed here, or below since txs are atomic.
    token::burn(
        ctx.accounts
            .into_burn_gsrm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        amount,
    )?;

    let locked_account = &mut ctx.accounts.locked_account;

    // CHECK: User can only burn gSRM tokens received from this LockedAccount.
    if amount + locked_account.gsrm_burned > locked_account.total_gsrm_amount {
        return err!(SerumGovError::InvalidGSRMAmount);
    }

    // CHECK: Amount must be multiple of MSRM_MULTIPLIER if LockedAccount was created on depositing MSRM tokens.
    if locked_account.is_msrm && (amount % MSRM_MULTIPLIER != 0) {
        return err!(SerumGovError::InvalidMSRMAmount);
    }

    locked_account.gsrm_burned = locked_account.gsrm_burned.checked_add(amount).unwrap();

    // Closing LockedAccount if all gSRM tokens were burned.
    if locked_account.gsrm_burned == locked_account.total_gsrm_amount {
        locked_account.close(ctx.accounts.owner.to_account_info())?;
    }

    // Calculating amount of SRM or gSRM tokens to be redeemed back.
    // NOTE: User cannot redeem SRM from a MSRM LockedAccount.
    let redeem_amount = if locked_account.is_msrm {
        amount / MSRM_MULTIPLIER
    } else {
        amount
    };

    let redeem_ticket = &mut ctx.accounts.redeem_ticket;
    redeem_ticket.owner = ctx.accounts.owner.key();
    redeem_ticket.deposit_account = locked_account.key();
    redeem_ticket.redeem_index = locked_account.redeem_index;
    redeem_ticket.bump = *ctx.bumps.get("redeem_ticket").unwrap();
    redeem_ticket.is_msrm = locked_account.is_msrm; // This decides whether amount is SRM or gSRM.
    redeem_ticket.created_at = ctx.accounts.clock.unix_timestamp;
    redeem_ticket.redeem_delay = REDEEM_DELAY;
    redeem_ticket.amount = redeem_amount;

    locked_account.redeem_index = locked_account.redeem_index.checked_add(1).unwrap();

    Ok(())
}
