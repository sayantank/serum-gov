use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program,
    sysvar, transaction::Transaction,
};

use crate::utils::helper::create_mint;

pub struct SerumGov {
    pub authority: Pubkey,
    pub gsrm_mint: Pubkey,
    pub srm_mint: Keypair,
    pub srm_vault: Pubkey,
    pub msrm_mint: Keypair,
    pub msrm_vault: Pubkey,
}

impl SerumGov {
    pub fn new() -> Self {
        let (authority, _bump) = Pubkey::find_program_address(&[b"authority"], &serum_gov::id());

        let (gsrm_mint, _bump) = Pubkey::find_program_address(&[b"gSRM"], &serum_gov::id());

        let srm_mint = Keypair::new();
        let msrm_mint = Keypair::new();

        let (srm_vault, _bump) = Pubkey::find_program_address(
            &[b"vault", &srm_mint.pubkey().to_bytes()],
            &serum_gov::id(),
        );
        let (msrm_vault, _bump) = Pubkey::find_program_address(
            &[b"vault", &msrm_mint.pubkey().to_bytes()],
            &serum_gov::id(),
        );

        SerumGov {
            authority,
            gsrm_mint,
            srm_mint,
            srm_vault,
            msrm_mint,
            msrm_vault,
        }
    }

    pub async fn setup_mints(&self, context: &mut ProgramTestContext, authority: &Keypair) {
        create_mint(context, &self.srm_mint, &authority.pubkey(), 6).await;
        create_mint(context, &self.msrm_mint, &authority.pubkey(), 0).await;
    }

    pub async fn init(
        &self,
        context: &mut ProgramTestContext,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let init_accounts = serum_gov::accounts::Init {
            payer: payer.pubkey(),
            authority: self.authority,
            gsrm_mint: self.gsrm_mint,
            srm_mint: self.srm_mint.pubkey(),
            msrm_mint: self.msrm_mint.pubkey(),
            srm_vault: self.srm_vault,
            msrm_vault: self.msrm_vault,
            rent: sysvar::rent::id(),
            token_program: spl_token::id(),
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let init_data = serum_gov::instruction::Init {}.data();

        let init_instruction = Instruction {
            program_id: serum_gov::id(),
            data: init_data,
            accounts: init_accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[init_instruction],
            Some(&payer.pubkey()),
            &[payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
