use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("G8aGmsybmGqQisBuRrDqiGfdz8YcCJcU3agWDFmTkf8S");

pub mod utils;

pub use utils::mints::{MSRM, SRM};

#[program]
pub mod serum_gov {
    use anchor_spl::token;

    use super::*;

    pub fn init(_ctx: Context<Init>) -> Result<()> {
        Ok(())
    }

    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        let user = &mut ctx.accounts.user_account;

        user.owner = ctx.accounts.owner.key();
        user.bump = *ctx.bumps.get("user_account").unwrap();
        user.locker_index = 0;

        Ok(())
    }

    pub fn deposit_srm(
        ctx: Context<DepositSRM>,
        amount: u64,
        collect_delay: i64,
        redeem_delay: i64,
    ) -> Result<()> {
        token::transfer(ctx.accounts.into_deposit_srm_context(), amount)?;

        let user_account = &mut ctx.accounts.user_account;

        let locker = &mut ctx.accounts.locker;
        locker.owner = ctx.accounts.owner.key();
        locker.is_msrm = false;
        locker.locker_index = user_account.locker_index;
        locker.bump = *ctx.bumps.get("locker").unwrap();
        locker.created_at = ctx.accounts.clock.unix_timestamp;
        locker.amount = amount;
        locker.claim_delay = collect_delay;
        locker.redeem_delay = redeem_delay;
        locker.redeemable_at = None;
        locker.is_claimed = false;

        user_account.locker_index += 1;

        Ok(())
    }

    pub fn deposit_msrm(
        ctx: Context<DepositMSRM>,
        amount: u64,
        collect_delay: i64,
        redeem_delay: i64,
    ) -> Result<()> {
        token::transfer(ctx.accounts.into_deposit_msrm_context(), amount)?;

        let user_account = &mut ctx.accounts.user_account;

        let locker = &mut ctx.accounts.locker;
        locker.owner = ctx.accounts.owner.key();
        locker.is_msrm = true;
        locker.locker_index = user_account.locker_index;
        locker.bump = *ctx.bumps.get("locker").unwrap();
        locker.created_at = ctx.accounts.clock.unix_timestamp;
        locker.amount = amount;
        locker.claim_delay = collect_delay;
        locker.redeem_delay = redeem_delay;
        locker.redeemable_at = None;
        locker.is_claimed = false;

        user_account.locker_index += 1;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, _locker_index: u64) -> Result<()> {
        let locker = &mut ctx.accounts.locker;

        locker.is_claimed = true;

        let mint_amount = if locker.is_msrm {
            locker
                .amount
                .checked_mul(1_000_000)
                .unwrap()
                .checked_mul(1_000_000)
                .unwrap()
        } else {
            locker.amount
        };

        token::mint_to(
            ctx.accounts
                .mint_gsrm()
                .with_signer(&[&[b"authority", &[*ctx.bumps.get("authority").unwrap()]]]),
            mint_amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(locker_index: u64)]
pub struct Claim<'info> {
    // #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"locker", &owner.key().to_bytes()[..], locker_index.to_string().as_bytes()],
        bump,
        constraint = locker.is_claimed == false @ SerumGovError::LockerAlreadyClaimed,
        constraint = (locker.created_at + locker.claim_delay) <= clock.unix_timestamp @ SerumGovError::LockerNotClaimable,
    )]
    pub locker: Account<'info, Locker>,

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
        mut,
        token::mint = msrm_mint,
        token::authority = authority,
    )]
    pub msrm_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [b"locker", &owner.key().to_bytes()[..], user_account.locker_index.to_string().as_bytes()],
        bump,
        space =  8 + std::mem::size_of::<Locker>()
    )]
    pub locker: Account<'info, Locker>,

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
        mut,
        token::mint = srm_mint,
        token::authority = authority,
    )]
    pub srm_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [b"locker", &owner.key().to_bytes()[..], user_account.locker_index.to_string().as_bytes()],
        bump,
        space =  8 + std::mem::size_of::<Locker>()
    )]
    pub locker: Account<'info, Locker>,

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
pub struct Init<'info> {
    /// NOTE: Could add constraint to restrict authorized payer, but this ix can't be called twice anyway.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Just a PDA for vault authorities.
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: AccountInfo<'info>,

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
    // pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct User {
    pub owner: Pubkey,
    pub bump: u8,
    pub locker_index: u64,
}

#[account]
pub struct Locker {
    pub owner: Pubkey,
    pub is_msrm: bool,
    pub locker_index: u64,
    pub bump: u8,
    pub created_at: i64,
    pub amount: u64,
    pub claim_delay: i64,
    pub redeem_delay: i64,
    pub is_claimed: bool,
    pub redeemable_at: Option<i64>,
}

#[error_code]
pub enum SerumGovError {
    #[msg("Locker has already been claimed.")]
    LockerAlreadyClaimed,

    #[msg("Locker is not currently claimable.")]
    LockerNotClaimable,
}
