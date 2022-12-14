use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    state::{ClaimTicket, Config, User, VestAccount},
    MSRM_MULTIPLIER,
};

#[derive(Accounts)]
pub struct DepositVestMSRM<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Owner account for which the vest is being created.
    pub owner: AccountInfo<'info>,

    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Box<Account<'info, Config>>,

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

    #[account(address = config.msrm_mint)]
    pub msrm_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = msrm_mint,
        token::authority = payer
    )]
    pub payer_msrm_account: Account<'info, TokenAccount>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"vault", &msrm_mint.key().to_bytes()[..]],
        bump,
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositVestMSRM<'info> {
    fn into_deposit_msrm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.payer_msrm_account.to_account_info().clone(),
            to: self.msrm_vault.to_account_info().clone(),
            authority: self.payer.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn handler(ctx: Context<DepositVestMSRM>, amount: u64) -> Result<()> {
    if amount <= 0 {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    token::transfer(ctx.accounts.into_deposit_msrm_context(), amount)?;

    let config = &ctx.accounts.config;
    let user_account = &mut ctx.accounts.owner_user_account;

    let gsrm_amount = amount.checked_mul(MSRM_MULTIPLIER).unwrap();

    let vest_account = &mut ctx.accounts.vest_account;
    vest_account.bump = *ctx.bumps.get("vest_account").unwrap();
    vest_account.owner = ctx.accounts.owner.key();
    vest_account.vest_index = user_account.vest_index;
    vest_account.redeem_index = 0;
    vest_account.is_msrm = true;
    vest_account.created_at = ctx.accounts.clock.unix_timestamp;
    vest_account.cliff_period = config.cliff_period;
    vest_account.linear_vesting_period = config.linear_vesting_period;
    vest_account.total_gsrm_amount = gsrm_amount;
    vest_account.gsrm_burned = 0;

    let claim_ticket = &mut ctx.accounts.claim_ticket;
    claim_ticket.owner = ctx.accounts.owner.key();
    claim_ticket.deposit_account = vest_account.key();
    claim_ticket.bump = *ctx.bumps.get("claim_ticket").unwrap();
    claim_ticket.created_at = ctx.accounts.clock.unix_timestamp;
    claim_ticket.claim_delay = config.claim_delay;
    claim_ticket.gsrm_amount = gsrm_amount;

    user_account.vest_index = user_account.vest_index.checked_add(1).unwrap();

    Ok(())
}
