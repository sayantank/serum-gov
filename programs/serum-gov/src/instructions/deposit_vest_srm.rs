use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[cfg(not(feature = "test-bpf"))]
use crate::config::mints::SRM;
use crate::{
    config::parameters::{CLAIM_DELAY, CLIFF_PERIOD, LINEAR_VESTING_PERIOD},
    state::{ClaimTicket, User, VestAccount},
};

#[derive(Accounts)]
pub struct DepositVestSRM<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Owner account for which the vest is being created.
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"user", &owner.key().to_bytes()[..]],
        bump,
    )]
    pub owner_user_account: Account<'info, User>,

    #[account(
        init,
        payer = payer,
        seeds = [b"vest_account", &owner.key().to_bytes()[..], owner_user_account.vest_index.to_le_bytes().as_ref()],
        bump,
        space = VestAccount::LEN
    )]
    pub vest_account: Account<'info, VestAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [b"claim_ticket", &vest_account.key().to_bytes()[..]],
        bump,
        space = ClaimTicket::LEN
    )]
    pub claim_ticket: Account<'info, ClaimTicket>,

    #[cfg_attr(
        not(feature = "test-bpf"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = srm_mint,
        token::authority = payer
    )]
    pub payer_srm_account: Account<'info, TokenAccount>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"vault", &srm_mint.key().to_bytes()[..]],
        bump,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositVestSRM<'info> {
    fn into_deposit_srm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.payer_srm_account.to_account_info().clone(),
            to: self.srm_vault.to_account_info().clone(),
            authority: self.payer.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<DepositVestSRM>, amount: u64) -> Result<()> {
    token::transfer(ctx.accounts.into_deposit_srm_context(), amount)?;

    let user_account = &mut ctx.accounts.owner_user_account;

    let vest_account = &mut ctx.accounts.vest_account;
    vest_account.owner = ctx.accounts.owner.key();
    vest_account.is_msrm = false;
    vest_account.bump = *ctx.bumps.get("vest_account").unwrap();
    vest_account.vest_index = user_account.vest_index;
    vest_account.created_at = ctx.accounts.clock.unix_timestamp;
    vest_account.cliff_period = CLIFF_PERIOD;
    vest_account.linear_vesting_period = LINEAR_VESTING_PERIOD;
    vest_account.total_gsrm_amount = amount;
    vest_account.gsrm_burned = 0;

    let claim_ticket = &mut ctx.accounts.claim_ticket;
    claim_ticket.owner = ctx.accounts.owner.key();
    claim_ticket.deposit_account = vest_account.key();
    claim_ticket.bump = *ctx.bumps.get("claim_ticket").unwrap();
    claim_ticket.created_at = ctx.accounts.clock.unix_timestamp;
    claim_ticket.claim_delay = CLAIM_DELAY;
    claim_ticket.gsrm_amount = amount;

    user_account.vest_index = user_account.vest_index.checked_add(1).unwrap();

    Ok(())
}
