use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

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

        let locker = &mut ctx.accounts.locker;
        locker.owner = ctx.accounts.owner.key();
        locker.is_msrm = false;
        locker.bump = *ctx.bumps.get("locker").unwrap();
        locker.created_at = ctx.accounts.clock.unix_timestamp;
        locker.amount = amount;
        locker.collect_delay = collect_delay;
        locker.redeem_delay = redeem_delay;

        let user_account = &mut ctx.accounts.user_account;
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

        let locker = &mut ctx.accounts.locker;
        locker.owner = ctx.accounts.owner.key();
        locker.is_msrm = true;
        locker.bump = *ctx.bumps.get("locker").unwrap();
        locker.created_at = ctx.accounts.clock.unix_timestamp;
        locker.amount = amount;
        locker.collect_delay = collect_delay;
        locker.redeem_delay = redeem_delay;

        let user_account = &mut ctx.accounts.user_account;
        user_account.locker_index += 1;

        Ok(())
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

    #[account(
        init,
        payer = payer,
        seeds = [b"gSRM"],
        bump,
        mint::decimals = 9,
        mint::authority = payer,
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
    pub bump: u8,
    pub created_at: i64,
    pub amount: u64,
    pub collect_delay: i64,
    pub redeem_delay: i64,
}
