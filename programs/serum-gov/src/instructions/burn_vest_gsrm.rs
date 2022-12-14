use std::cmp;

use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::{
    errors::SerumGovError,
    state::{Config, RedeemTicket, VestAccount},
    MSRM_MULTIPLIER,
};

#[derive(Accounts)]
pub struct BurnVestGSRM<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

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
        seeds = [b"vest_account", &owner.key().to_bytes()[..], vest_account.vest_index.to_le_bytes().as_ref()],
        bump,
        constraint = clock.unix_timestamp >= (vest_account.created_at + vest_account.cliff_period) @ SerumGovError::TooEarlyToVest,
    )]
    pub vest_account: Account<'info, VestAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [b"redeem_ticket", &vest_account.key().to_bytes()[..], vest_account.redeem_index.to_le_bytes().as_ref()],
        bump,
        space = RedeemTicket::LEN
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BurnVestGSRM<'info> {
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

pub fn handler(ctx: Context<BurnVestGSRM>, amount: u64) -> Result<()> {
    if amount <= 0 {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let vest_account = &ctx.accounts.vest_account;

    let current_timestamp = ctx.accounts.clock.unix_timestamp;
    let cliff_end = vest_account
        .created_at
        .checked_add(vest_account.cliff_period)
        .unwrap();

    // vested_time = time passed after cliff period ended
    let vested_time = current_timestamp.checked_sub(cliff_end).unwrap();
    msg!("Time vested: {}", vested_time);

    // vested_amount = (vested_time / linear_vest_period) * total_gsrm_amount
    let vested_amount = u128::from(vest_account.total_gsrm_amount)
        .checked_mul(vested_time.try_into().unwrap())
        .unwrap()
        .checked_div(vest_account.linear_vesting_period.try_into().unwrap())
        .unwrap();

    // If vested_time > linear_vest_period, vested_amount will be greater than total_gsrm_amount.
    // Hence, vested_amount = min(vested_amount, total_gsrm_amount)
    let vested_amount = cmp::min(
        u64::try_from(vested_amount).unwrap(),
        vest_account.total_gsrm_amount,
    );

    // Accounting for already redeemed gsrm
    let redeemable_amount = vested_amount.checked_sub(vest_account.gsrm_burned).unwrap();

    // Just another layer of check if closing account remains vulnerable.
    if redeemable_amount <= 0 {
        return err!(SerumGovError::AlreadyRedeemed);
    }

    // CHECK: If MSRM, then amount must be multiple for MSRM
    if vest_account.is_msrm && (amount % MSRM_MULTIPLIER != 0) {
        return err!(SerumGovError::InvalidMSRMAmount);
    }

    // If user passed in amount < redeemable_amount, then redeem only that amount
    // gsrm_amount = min(redeemable_amount, amount)
    let gsrm_amount = cmp::min(redeemable_amount, amount);

    msg!("Redeeming {} gSRM", gsrm_amount);
    token::burn(
        ctx.accounts
            .into_burn_gsrm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        gsrm_amount,
    )?;

    let vest_account = &mut ctx.accounts.vest_account;
    vest_account.gsrm_burned = vest_account.gsrm_burned.checked_add(gsrm_amount).unwrap();

    if vest_account.gsrm_burned == vest_account.total_gsrm_amount {
        vest_account.close(ctx.accounts.owner.to_account_info())?;
    }

    let redeem_ticket = &mut ctx.accounts.redeem_ticket;
    redeem_ticket.owner = ctx.accounts.owner.key();
    redeem_ticket.deposit_account = vest_account.key();
    redeem_ticket.redeem_index = vest_account.redeem_index;
    redeem_ticket.bump = *ctx.bumps.get("redeem_ticket").unwrap();
    redeem_ticket.is_msrm = vest_account.is_msrm;
    redeem_ticket.created_at = ctx.accounts.clock.unix_timestamp;
    redeem_ticket.redeem_delay = ctx.accounts.config.redeem_delay;
    redeem_ticket.amount = if vest_account.is_msrm {
        amount.checked_div(MSRM_MULTIPLIER).unwrap()
    } else {
        amount
    };

    vest_account.redeem_index = vest_account.redeem_index.checked_add(1).unwrap();

    Ok(())
}
