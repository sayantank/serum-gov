use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::{id as meta_id, instruction::create_metadata_accounts_v3, state::PREFIX};
use solana_program::program::invoke_signed;

#[cfg(not(feature = "test-bpf"))]
use crate::config::mints::{MSRM, SRM};
use crate::state::Config;

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
        seeds = [b"config"],
        bump,
        space = Config::LEN
    )]
    pub config: Box<Account<'info, Config>>,

    /// NOTE: Decimals have been kept same as SRM.
    #[account(
        init,
        payer = payer,
        seeds = [b"gSRM"],
        bump,
        mint::decimals = srm_mint.decimals,
        mint::authority = authority,
    )]
    pub gsrm_mint: Account<'info, Mint>,

    /// CHECK: Metaplex Token Metadata account intialized via CPI
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), &meta_id().to_bytes()[..], &gsrm_mint.key().to_bytes()[..]],
        bump,
        seeds::program = meta_id()
    )]
    pub gsrm_metadata: AccountInfo<'info>,

    #[cfg_attr(
        not(feature = "test-bpf"),
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
        not(feature = "test-bpf"),
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

    /// CHECK: MPL Token Metadata Program for CPI
    #[account(
        executable,
        address = meta_id()
    )]
    mpl_token_metadata_program: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<Init>,
    name: String,
    symbol: String,
    config_authority: Pubkey,
    claim_delay: i64,
    redeem_delay: i64,
    cliff_period: i64,
    linear_vesting_period: i64,
) -> Result<()> {
    msg!("Initializing Serum Gov");
    msg!(
        "Config params: {:?}",
        (
            claim_delay,
            redeem_delay,
            cliff_period,
            linear_vesting_period
        )
    );

    let config = &mut ctx.accounts.config;
    config.config_authority = config_authority;
    config.srm_mint = ctx.accounts.srm_mint.key();
    config.msrm_mint = ctx.accounts.msrm_mint.key();
    config.claim_delay = claim_delay;
    config.redeem_delay = redeem_delay;
    config.cliff_period = cliff_period;
    config.linear_vesting_period = linear_vesting_period;

    let ix = create_metadata_accounts_v3(
        meta_id(),
        ctx.accounts.gsrm_metadata.key(),
        ctx.accounts.gsrm_mint.key(),
        ctx.accounts.authority.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.authority.key(),
        name,
        symbol,
        "".to_string(),
        None,
        0,
        true,
        true,
        None,
        None,
        None,
    );

    let cpi_account_infos = vec![
        ctx.accounts.gsrm_metadata.clone(),
        ctx.accounts.gsrm_mint.to_account_info(),
        ctx.accounts.authority.clone(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.authority.clone(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.clone(),
    ];

    let auth_bump = *ctx.bumps.get("authority").unwrap();

    invoke_signed(&ix, &cpi_account_infos, &[&[b"authority", &[auth_bump]]])?;

    Ok(())
}
