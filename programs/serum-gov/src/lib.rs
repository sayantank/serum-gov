use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("G8aGmsybmGqQisBuRrDqiGfdz8YcCJcU3agWDFmTkf8S");

pub mod config;

pub use config::{
    authority::UPGRADE_AUTHORITY,
    mints::{MSRM, SRM},
};

const MSRM_MULTIPLIER: u64 = 1_000_000_000_000;

#[program]
pub mod serum_gov {
    use anchor_spl::token;

    use super::*;

    pub fn init(ctx: Context<Init>, claim_delay: i64, redeem_delay: i64) -> Result<()> {
        let config = &mut ctx.accounts.config;

        config.claim_delay = claim_delay;
        config.redeem_delay = redeem_delay;

        Ok(())
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        claim_delay: i64,
        redeem_delay: i64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        config.claim_delay = claim_delay;
        config.redeem_delay = redeem_delay;

        Ok(())
    }

    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        let user = &mut ctx.accounts.user_account;

        user.owner = ctx.accounts.owner.key();
        user.bump = *ctx.bumps.get("user_account").unwrap();
        user.claim_index = 0;
        user.redeem_index = 0;

        Ok(())
    }

    pub fn deposit_srm(ctx: Context<DepositSRM>, amount: u64) -> Result<()> {
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

    pub fn deposit_msrm(ctx: Context<DepositMSRM>, amount: u64) -> Result<()> {
        token::transfer(ctx.accounts.into_deposit_msrm_context(), amount)?;

        let user_account = &mut ctx.accounts.user_account;

        let ticket = &mut ctx.accounts.claim_ticket;
        ticket.owner = ctx.accounts.owner.key();
        ticket.is_msrm = true;
        ticket.bump = *ctx.bumps.get("claim_ticket").unwrap();
        ticket.created_at = ctx.accounts.clock.unix_timestamp;
        ticket.claim_delay = ctx.accounts.config.claim_delay;
        ticket.amount = amount;
        ticket.claim_index = user_account.claim_index;

        user_account.claim_index += 1;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, _claim_index: u64) -> Result<()> {
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

    pub fn burn_gsrm(ctx: Context<BurnGSRM>, amount: u64, is_msrm: bool) -> Result<()> {
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
        ticket.redeem_delay = ctx.accounts.config.redeem_delay;
        ticket.amount = redeem_amount;
        ticket.redeem_index = user_account.redeem_index;

        user_account.redeem_index += 1;

        Ok(())
    }

    pub fn redeem_srm(ctx: Context<RedeemSRM>, _redeem_index: u64) -> Result<()> {
        token::transfer(
            ctx.accounts
                .into_redeem_srm_context()
                .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
            ctx.accounts.redeem_ticket.amount,
        )?;
        Ok(())
    }

    pub fn redeem_msrm(ctx: Context<RedeemMSRM>, _redeem_index: u64) -> Result<()> {
        token::transfer(
            ctx.accounts
                .into_redeem_msrm_context()
                .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
            ctx.accounts.redeem_ticket.amount,
        )?;
        Ok(())
    }
}

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

#[derive(Accounts)]
#[instruction(redeem_index: u64)]
pub struct RedeemSRM<'info> {
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
        constraint = redeem_ticket.is_msrm == false @ SerumGovError::InvalidRedeemTicket,
        constraint = (redeem_ticket.created_at + redeem_ticket.redeem_delay) <= clock.unix_timestamp @ SerumGovError::TicketNotClaimable,
        close = owner
    )]
    pub redeem_ticket: Account<'info, RedeemTicket>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = srm_mint,
        associated_token::authority = owner
    )]
    pub owner_srm_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RedeemSRM<'info> {
    fn into_redeem_srm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.srm_vault.to_account_info().clone(),
            to: self.owner_srm_account.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

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

    #[account(
        seeds = [b"config"],
        bump,
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

#[derive(Accounts)]
pub struct DepositMSRM<'info> {
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
        account(address = MSRM),
    )]
    pub msrm_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = msrm_mint,
        associated_token::authority = owner
    )]
    pub owner_msrm_account: Account<'info, TokenAccount>,

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
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

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

impl<'info> DepositMSRM<'info> {
    fn into_deposit_msrm_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.owner_msrm_account.to_account_info().clone(),
            to: self.msrm_vault.to_account_info().clone(),
            authority: self.owner.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

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

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [b"user", &owner.key().to_bytes()[..]],
        bump,
        space = 8 + std::mem::size_of::<User>()
    )]
    pub user_account: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[cfg_attr(
        not(feature = "test"),
        account(address = UPGRADE_AUTHORITY),
    )]
    pub upgrade_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Init<'info> {
    /// NOTE: Could add constraint to restrict authorized payer, but this ix can't be called twice anyway.
    #[account(mut)]
    #[cfg_attr(
        not(feature = "test"),
        account(address = UPGRADE_AUTHORITY),
    )]
    pub payer: Signer<'info>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"config"],
        bump,
        space = 8 + std::mem::size_of::<Config>()
    )]
    pub config: Account<'info, Config>,

    /// NOTE: Decimals have been kept same as SRM.
    #[account(
        init,
        payer = payer,
        seeds = [b"gSRM"],
        bump,
        mint::decimals = 6,
        mint::authority = authority,
    )]
    pub gsrm_mint: Account<'info, Mint>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = SRM),
    )]
    pub srm_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"vault", &srm_mint.key().to_bytes()[..]],
        bump,
        payer = payer,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    #[cfg_attr(
        not(feature = "test"),
        account(address = MSRM),
    )]
    pub msrm_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"vault", &msrm_mint.key().to_bytes()[..]],
        bump,
        payer = payer,
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Config {
    pub claim_delay: i64,
    pub redeem_delay: i64,
}

#[account]
pub struct User {
    pub owner: Pubkey,
    pub bump: u8,
    pub claim_index: u64,
    pub redeem_index: u64,
}

#[account]
pub struct ClaimTicket {
    pub owner: Pubkey,
    pub is_msrm: bool,
    pub bump: u8,
    pub created_at: i64,
    pub claim_delay: i64,
    pub amount: u64,
    pub claim_index: u64,
}

#[account]
pub struct RedeemTicket {
    pub owner: Pubkey,
    pub is_msrm: bool,
    pub bump: u8,
    pub created_at: i64,
    pub redeem_delay: i64,
    pub amount: u64,
    pub redeem_index: u64,
}

#[error_code]
pub enum SerumGovError {
    #[msg("Ticket is not currently claimable.")]
    TicketNotClaimable,

    #[msg("Invalid amount for redeeming MSRM.")]
    InvalidMSRMAmount,

    #[msg("Invalid RedeemTicket")]
    InvalidRedeemTicket,
}
