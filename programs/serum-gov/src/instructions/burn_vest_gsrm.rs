use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::{
    config::parameters::REDEEM_DELAY,
    errors::SerumGovError,
    state::{RedeemTicket, VestAccount},
};

#[derive(Accounts)]
#[instruction(vest_index: u64)]
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
        mut,
        seeds = [b"vest_account", &owner.key().to_bytes()[..], vest_index.to_string().as_bytes()],
        bump,
        constraint = clock.unix_timestamp >= (vest_account.created_at + vest_account.cliff_period) @ SerumGovError::TooEarlyToVest,
    )]
    pub vest_account: Account<'info, VestAccount>,

    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<RedeemTicket>()
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

pub fn handler(ctx: Context<BurnVestGSRM>, _vest_index: u64, amount: u64) -> Result<()> {
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
    let vested_amount = if u64::try_from(vested_amount).unwrap() > vest_account.total_gsrm_amount {
        vest_account.total_gsrm_amount
    } else {
        u64::try_from(vested_amount).unwrap()
    };

    // Accounting for already redeemed gsrm
    let redeemable_amount = vested_amount.checked_sub(vest_account.gsrm_burned).unwrap();

    // If user passed in amount < redeemable_amount, then redeem only that amount
    // redeem_amount = min(redeemable_amount, amount)
    let redeem_amount = if amount < redeemable_amount {
        amount
    } else {
        redeemable_amount
    };

    msg!("Redeeming {} gSRM", redeem_amount);
    token::burn(
        ctx.accounts
            .into_burn_gsrm_context()
            .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
        redeem_amount,
    )?;

    let vest_account = &mut ctx.accounts.vest_account;
    vest_account.gsrm_burned = vest_account.gsrm_burned.checked_add(redeem_amount).unwrap();

    if vest_account.gsrm_burned == vest_account.total_gsrm_amount {
        vest_account.close(ctx.accounts.owner.to_account_info())?;
    }

    let redeem_ticket = &mut ctx.accounts.redeem_ticket;
    redeem_ticket.owner = ctx.accounts.owner.key();
    redeem_ticket.is_msrm = false;
    redeem_ticket.created_at = ctx.accounts.clock.unix_timestamp;
    redeem_ticket.redeem_delay = REDEEM_DELAY;
    redeem_ticket.amount = redeem_amount;

    Ok(())
}
