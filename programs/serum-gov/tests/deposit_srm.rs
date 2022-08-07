mod core;
mod utils;

mod deposit_srm {
    use crate::{
        core::{serum_gov::SerumGov, user::UserAccount},
        utils::helper::{airdrop, create_associated_token_account, mint_tokens},
    };
    use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
    use serum_gov::state::ClaimTicket;
    use solana_program_test::*;
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
        system_program, sysvar, transaction::Transaction,
    };

    #[tokio::test]
    async fn success_deposit_srm() {
        let program = ProgramTest::new("serum_gov", serum_gov::id(), None);

        let context = &mut program.start_with_context().await;

        let sbf = Keypair::new();
        airdrop(context, &sbf.pubkey(), 10_000_000_000).await;

        let serum_gov = SerumGov::new();

        serum_gov.setup_mints(context, &sbf).await;
        serum_gov.init(context, &sbf).await.unwrap();

        let user = UserAccount::new();
        airdrop(context, &user.owner.pubkey(), 10_000_000_000).await;
        user.init_user(context).await.unwrap();

        let user_srm_account =
            create_associated_token_account(context, &user.owner, &serum_gov.srm_mint.pubkey())
                .await;

        mint_tokens(
            context,
            &sbf,
            &serum_gov.srm_mint.pubkey(),
            &user_srm_account,
            200_000_000,
            None,
        )
        .await
        .unwrap();

        let user_account_data = user.get_user_account_data(context).await;

        let (claim_ticket, _bump) = Pubkey::find_program_address(
            &[
                b"claim",
                &user.owner.pubkey().to_bytes(),
                &user_account_data.claim_index.to_string().as_bytes(),
            ],
            &serum_gov::id(),
        );

        let accounts = serum_gov::accounts::DepositSRM {
            owner: user.owner.pubkey(),
            user_account: user.user_account,
            srm_mint: serum_gov.srm_mint.pubkey(),
            owner_srm_account: user_srm_account,
            authority: serum_gov.authority,
            srm_vault: serum_gov.srm_vault,
            claim_ticket,
            clock: sysvar::clock::id(),
            token_program: spl_token::id(),
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = serum_gov::instruction::DepositSrm {
            amount: 100_000_000,
        }
        .data();

        let instruction = Instruction {
            program_id: serum_gov::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&user.owner.pubkey()),
            &[&user.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let claim_ticket_account = context
            .banks_client
            .get_account(claim_ticket)
            .await
            .unwrap()
            .unwrap();

        let claim_ticket_data =
            ClaimTicket::try_deserialize(&mut claim_ticket_account.data.as_ref()).unwrap();

        assert_eq!(claim_ticket_data.amount, 100_000_000);
        assert_eq!(claim_ticket_data.is_msrm, false);
        assert_eq!(claim_ticket_data.owner, user.owner.pubkey());
        assert_eq!(claim_ticket_data.claim_index, 0);
        assert_eq!(claim_ticket_data.claim_delay, 1000);

        let user_account_data = user.get_user_account_data(context).await;
        assert_eq!(user_account_data.claim_index, 1);
    }

    #[tokio::test]
    pub async fn fail_invalid_claim_index() {
        let program = ProgramTest::new("serum_gov", serum_gov::id(), None);

        let context = &mut program.start_with_context().await;

        let sbf = Keypair::new();
        airdrop(context, &sbf.pubkey(), 10_000_000_000).await;

        let serum_gov = SerumGov::new();

        serum_gov.setup_mints(context, &sbf).await;
        serum_gov.init(context, &sbf).await.unwrap();

        let user = UserAccount::new();
        airdrop(context, &user.owner.pubkey(), 10_000_000_000).await;
        user.init_user(context).await.unwrap();

        let user_srm_account =
            create_associated_token_account(context, &user.owner, &serum_gov.srm_mint.pubkey())
                .await;

        mint_tokens(
            context,
            &sbf,
            &serum_gov.srm_mint.pubkey(),
            &user_srm_account,
            200_000_000,
            None,
        )
        .await
        .unwrap();

        let (claim_ticket, _bump) = Pubkey::find_program_address(
            &[b"claim", &user.owner.pubkey().to_bytes(), "1".as_bytes()],
            &serum_gov::id(),
        );

        let accounts = serum_gov::accounts::DepositSRM {
            owner: user.owner.pubkey(),
            user_account: user.user_account,
            srm_mint: serum_gov.srm_mint.pubkey(),
            owner_srm_account: user_srm_account,
            authority: serum_gov.authority,
            srm_vault: serum_gov.srm_vault,
            claim_ticket,
            clock: sysvar::clock::id(),
            token_program: spl_token::id(),
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = serum_gov::instruction::DepositSrm {
            amount: 100_000_000,
        }
        .data();

        let instruction = Instruction {
            program_id: serum_gov::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&user.owner.pubkey()),
            &[&user.owner],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        match err {
            BanksClientError::TransactionError(_) => assert!(true),
            _ => assert!(false),
        }
    }
}
