#![allow(unused)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use serum_gov::state::User;
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program,
    transaction::Transaction,
};

pub struct UserAccount {
    pub owner: Keypair,
    pub user_account: Pubkey,
}

impl UserAccount {
    pub fn new() -> Self {
        let owner = Keypair::new();
        let (user_account, _bump) =
            Pubkey::find_program_address(&[b"user", &owner.pubkey().to_bytes()], &serum_gov::id());

        Self {
            owner,
            user_account,
        }
    }

    pub async fn init_user(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        let accounts = serum_gov::accounts::InitUser {
            owner: self.owner.pubkey(),
            user_account: self.user_account,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = serum_gov::instruction::InitUser {}.data();

        let instruction = Instruction {
            program_id: serum_gov::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.owner.pubkey()),
            &[&self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn get_user_account_data(&self, context: &mut ProgramTestContext) -> User {
        let user_account = context
            .banks_client
            .get_account(self.user_account)
            .await
            .unwrap()
            .unwrap();

        User::try_deserialize(&mut user_account.data.as_ref()).unwrap()
    }
}
